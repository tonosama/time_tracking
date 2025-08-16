use crate::domain::entities::Task;
use crate::domain::value_objects::{ProjectId, TaskId, Status};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// タスクリポジトリトレイト
#[async_trait]
pub trait TaskRepository: Send + Sync {
    /// タスクを保存（新規作成またはバージョン追加）
    async fn save(&self, task: &Task) -> anyhow::Result<()>;

    /// タスクIDで検索
    async fn find_by_id(&self, id: TaskId) -> anyhow::Result<Option<Task>>;

    /// プロジェクトIDでタスク一覧を取得
    async fn find_by_project_id(&self, project_id: ProjectId) -> anyhow::Result<Vec<Task>>;

    /// プロジェクトIDでアクティブなタスク一覧を取得
    async fn find_active_by_project_id(&self, project_id: ProjectId) -> anyhow::Result<Vec<Task>>;

    /// 全てのアクティブなタスクを取得
    async fn find_all_active(&self) -> anyhow::Result<Vec<Task>>;

    /// 全てのタスクを取得（ステータス関係なし）
    async fn find_all(&self) -> anyhow::Result<Vec<Task>>;

    /// 指定したステータスのタスクを取得
    async fn find_by_status(&self, status: &Status) -> anyhow::Result<Vec<Task>>;

    /// タスク名で検索（前方一致）
    async fn find_by_name_prefix(&self, prefix: &str) -> anyhow::Result<Vec<Task>>;

    /// 次に使用可能なタスクIDを生成
    async fn next_id(&self) -> anyhow::Result<TaskId>;

    /// タスクの存在確認
    async fn exists(&self, id: TaskId) -> anyhow::Result<bool>;

    /// タスクの履歴を取得
    async fn find_history(&self, id: TaskId) -> anyhow::Result<Vec<Task>>;

    /// 指定した時刻でのタスク状態を取得
    async fn find_at_time(&self, id: TaskId, at: DateTime<Utc>) -> anyhow::Result<Option<Task>>;

    /// 指定したプロジェクトに属するタスクの数を取得
    async fn count_by_project_id(&self, project_id: ProjectId) -> anyhow::Result<usize>;

    /// 階層構造のため、プロジェクト配下のタスクを階層順で取得
    async fn find_by_project_id_ordered(&self, project_id: ProjectId) -> anyhow::Result<Vec<Task>>;
}

#[cfg(test)]
#[allow(non_snake_case)]
pub mod tests {
    use super::*;
    use crate::domain::entities::Task;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // テスト用のインメモリリポジトリ実装
    #[derive(Debug, Default, Clone)]
    pub struct InMemoryTaskRepository {
        tasks: Arc<Mutex<HashMap<TaskId, Vec<Task>>>>,
        next_id: Arc<Mutex<i64>>,
    }

    impl InMemoryTaskRepository {
        pub fn new() -> Self {
            Self {
                tasks: Arc::new(Mutex::new(HashMap::new())),
                next_id: Arc::new(Mutex::new(1)),
            }
        }
    }

    #[async_trait]
    impl TaskRepository for InMemoryTaskRepository {
        async fn save(&self, task: &Task) -> anyhow::Result<()> {
            let mut tasks = self.tasks.lock().await;
            let versions = tasks.entry(task.id()).or_insert_with(Vec::new);
            versions.push(task.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: TaskId) -> anyhow::Result<Option<Task>> {
            let tasks = self.tasks.lock().await;
            if let Some(versions) = tasks.get(&id) {
                let latest = versions
                    .iter()
                    .max_by_key(|t| t.effective_at())
                    .cloned();
                Ok(latest)
            } else {
                Ok(None)
            }
        }

        async fn find_by_project_id(&self, project_id: ProjectId) -> anyhow::Result<Vec<Task>> {
            let tasks = self.tasks.lock().await;
            let mut result = Vec::new();
            
            for versions in tasks.values() {
                if let Some(latest) = versions.iter().max_by_key(|t| t.effective_at()) {
                    if latest.belongs_to_project(project_id) {
                        result.push(latest.clone());
                    }
                }
            }
            
            Ok(result)
        }

        async fn find_active_by_project_id(&self, project_id: ProjectId) -> anyhow::Result<Vec<Task>> {
            let tasks = self.tasks.lock().await;
            let mut result = Vec::new();
            
            for versions in tasks.values() {
                if let Some(latest) = versions.iter().max_by_key(|t| t.effective_at()) {
                    if latest.belongs_to_project(project_id) && latest.is_active() {
                        result.push(latest.clone());
                    }
                }
            }
            
            Ok(result)
        }

        async fn find_all_active(&self) -> anyhow::Result<Vec<Task>> {
            let tasks = self.tasks.lock().await;
            let mut result = Vec::new();
            
            for versions in tasks.values() {
                if let Some(latest) = versions.iter().max_by_key(|t| t.effective_at()) {
                    if latest.is_active() {
                        result.push(latest.clone());
                    }
                }
            }
            
            Ok(result)
        }

        async fn find_all(&self) -> anyhow::Result<Vec<Task>> {
            let tasks = self.tasks.lock().await;
            let mut result = Vec::new();
            
            for versions in tasks.values() {
                if let Some(latest) = versions.iter().max_by_key(|t| t.effective_at()) {
                    result.push(latest.clone());
                }
            }
            
            Ok(result)
        }

        async fn find_by_status(&self, status: &Status) -> anyhow::Result<Vec<Task>> {
            let tasks = self.tasks.lock().await;
            let mut result = Vec::new();
            
            for versions in tasks.values() {
                if let Some(latest) = versions.iter().max_by_key(|t| t.effective_at()) {
                    if latest.status() == status {
                        result.push(latest.clone());
                    }
                }
            }
            
            Ok(result)
        }

        async fn find_by_name_prefix(&self, prefix: &str) -> anyhow::Result<Vec<Task>> {
            let tasks = self.tasks.lock().await;
            let mut result = Vec::new();
            
            for versions in tasks.values() {
                if let Some(latest) = versions.iter().max_by_key(|t| t.effective_at()) {
                    if latest.name().starts_with(prefix) {
                        result.push(latest.clone());
                    }
                }
            }
            
            Ok(result)
        }

        async fn next_id(&self) -> anyhow::Result<TaskId> {
            let mut next_id = self.next_id.lock().await;
            let id = *next_id;
            *next_id += 1;
            TaskId::new(id)
        }

        async fn exists(&self, id: TaskId) -> anyhow::Result<bool> {
            let tasks = self.tasks.lock().await;
            Ok(tasks.contains_key(&id))
        }

        async fn find_history(&self, id: TaskId) -> anyhow::Result<Vec<Task>> {
            let tasks = self.tasks.lock().await;
            if let Some(versions) = tasks.get(&id) {
                let mut history = versions.clone();
                history.sort_by_key(|t| t.effective_at());
                Ok(history)
            } else {
                Ok(Vec::new())
            }
        }

        async fn find_at_time(&self, id: TaskId, at: DateTime<Utc>) -> anyhow::Result<Option<Task>> {
            let tasks = self.tasks.lock().await;
            if let Some(versions) = tasks.get(&id) {
                let task = versions
                    .iter()
                    .filter(|t| t.effective_at() <= at)
                    .max_by_key(|t| t.effective_at())
                    .cloned();
                Ok(task)
            } else {
                Ok(None)
            }
        }

        async fn count_by_project_id(&self, project_id: ProjectId) -> anyhow::Result<usize> {
            let tasks = self.find_by_project_id(project_id).await?;
            Ok(tasks.len())
        }

        async fn find_by_project_id_ordered(&self, project_id: ProjectId) -> anyhow::Result<Vec<Task>> {
            let mut tasks = self.find_by_project_id(project_id).await?;
            tasks.sort_by(|a, b| a.name().cmp(b.name()));
            Ok(tasks)
        }
    }

    // テストケース
    #[test]
    fn タスクの保存と取得ができること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTaskRepository::new();
            let task_id = TaskId::new(1).unwrap();
            let project_id = ProjectId::new(1).unwrap();
            let task = Task::new(task_id, project_id, "テストタスク".to_string()).unwrap();
            
            repository.save(&task).await.unwrap();
            
            let found = repository.find_by_id(task_id).await.unwrap().unwrap();
            assert_eq!(found.id(), task_id);
            assert_eq!(found.project_id(), project_id);
            assert_eq!(found.name(), "テストタスク");
        });
    }

    #[test]
    fn 次のタスクIDが正しく生成されること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTaskRepository::new();
            
            let id1 = repository.next_id().await.unwrap();
            let id2 = repository.next_id().await.unwrap();
            let id3 = repository.next_id().await.unwrap();
            
            assert_eq!(id1, TaskId::new(1).unwrap());
            assert_eq!(id2, TaskId::new(2).unwrap());
            assert_eq!(id3, TaskId::new(3).unwrap());
        });
    }

    #[test]
    fn プロジェクトIDでタスクが取得できること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTaskRepository::new();
            let project1 = ProjectId::new(1).unwrap();
            let project2 = ProjectId::new(2).unwrap();
            
            let task1 = Task::new(TaskId::new(1).unwrap(), project1, "プロジェクト1のタスク".to_string()).unwrap();
            let task2 = Task::new(TaskId::new(2).unwrap(), project1, "プロジェクト1のタスク2".to_string()).unwrap();
            let task3 = Task::new(TaskId::new(3).unwrap(), project2, "プロジェクト2のタスク".to_string()).unwrap();
            
            repository.save(&task1).await.unwrap();
            repository.save(&task2).await.unwrap();
            repository.save(&task3).await.unwrap();
            
            let project1_tasks = repository.find_by_project_id(project1).await.unwrap();
            assert_eq!(project1_tasks.len(), 2);
            
            let project2_tasks = repository.find_by_project_id(project2).await.unwrap();
            assert_eq!(project2_tasks.len(), 1);
        });
    }

    #[test]
    fn プロジェクトIDでアクティブタスクが取得できること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTaskRepository::new();
            let project_id = ProjectId::new(1).unwrap();
            
            let task1 = Task::new(TaskId::new(1).unwrap(), project_id, "アクティブタスク".to_string()).unwrap();
            let mut task2 = Task::new(TaskId::new(2).unwrap(), project_id, "アーカイブタスク".to_string()).unwrap();
            task2 = task2.archive();
            
            repository.save(&task1).await.unwrap();
            repository.save(&task2).await.unwrap();
            
            let active_tasks = repository.find_active_by_project_id(project_id).await.unwrap();
            assert_eq!(active_tasks.len(), 1);
            assert_eq!(active_tasks[0].name(), "アクティブタスク");
        });
    }

    #[test]
    fn タスク履歴が取得できること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTaskRepository::new();
            let task_id = TaskId::new(1).unwrap();
            let project_id = ProjectId::new(1).unwrap();
            
            let task1 = Task::new(task_id, project_id, "初期名".to_string()).unwrap();
            let task2 = task1.change_name("変更後".to_string()).unwrap();
            let task3 = task2.archive();
            
            repository.save(&task1).await.unwrap();
            repository.save(&task2).await.unwrap();
            repository.save(&task3).await.unwrap();
            
            let history = repository.find_history(task_id).await.unwrap();
            assert_eq!(history.len(), 3);
            assert_eq!(history[0].name(), "初期名");
            assert_eq!(history[1].name(), "変更後");
            assert_eq!(history[2].name(), "変更後");
            assert!(history[2].status().is_archived());
        });
    }

    #[test]
    fn プロジェクト内アクティブタスク数が取得できること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTaskRepository::new();
            let project_id = ProjectId::new(1).unwrap();
            
            let task1 = Task::new(TaskId::new(1).unwrap(), project_id, "タスク1".to_string()).unwrap();
            let task2 = Task::new(TaskId::new(2).unwrap(), project_id, "タスク2".to_string()).unwrap();
            let mut task3 = Task::new(TaskId::new(3).unwrap(), project_id, "タスク3".to_string()).unwrap();
            task3 = task3.archive();
            
            repository.save(&task1).await.unwrap();
            repository.save(&task2).await.unwrap();
            repository.save(&task3).await.unwrap();
            
            let count = repository.count_by_project_id(project_id).await.unwrap();
            assert_eq!(count, 3); // 全タスク数（アーカイブ含む）
            
            let active_tasks = repository.find_active_by_project_id(project_id).await.unwrap();
            assert_eq!(active_tasks.len(), 2); // アクティブタスク数のみ
        });
    }

    // 追加のテストケース

    #[test]
    fn タスクの複数回更新ができること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTaskRepository::new();
            let task_id = TaskId::new(1).unwrap();
            let project_id = ProjectId::new(1).unwrap();
            
            let mut task = Task::new(task_id, project_id, "初期名".to_string()).unwrap();
            repository.save(&task).await.unwrap();
            
            task = task.change_name("2回目".to_string()).unwrap();
            repository.save(&task).await.unwrap();
            
            task = task.change_name("3回目".to_string()).unwrap();
            repository.save(&task).await.unwrap();
            
            let history = repository.find_history(task_id).await.unwrap();
            assert_eq!(history.len(), 3);
            assert_eq!(history[0].name(), "初期名");
            assert_eq!(history[1].name(), "2回目");
            assert_eq!(history[2].name(), "3回目");
        });
    }

    #[test]
    fn タスクのアーカイブと復元ができること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTaskRepository::new();
            let task_id = TaskId::new(1).unwrap();
            let project_id = ProjectId::new(1).unwrap();
            
            let mut task = Task::new(task_id, project_id, "復元テスト".to_string()).unwrap();
            repository.save(&task).await.unwrap();
            
            task = task.archive();
            repository.save(&task).await.unwrap();
            
            task = task.restore().unwrap();
            repository.save(&task).await.unwrap();
            
            let found_task = repository.find_by_id(task_id).await.unwrap().unwrap();
            assert!(found_task.status().is_active());
            
            let active_tasks = repository.find_active_by_project_id(project_id).await.unwrap();
            assert_eq!(active_tasks.len(), 1);
            assert_eq!(active_tasks[0].name(), "復元テスト");
        });
    }

    #[test]
    fn 空のプロジェクトでの動作確認() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTaskRepository::new();
            let project_id = ProjectId::new(1).unwrap();
            
            let tasks = repository.find_by_project_id(project_id).await.unwrap();
            assert_eq!(tasks.len(), 0);
            
            let active_tasks = repository.find_active_by_project_id(project_id).await.unwrap();
            assert_eq!(active_tasks.len(), 0);
            
            let count = repository.count_by_project_id(project_id).await.unwrap();
            assert_eq!(count, 0);
        });
    }

    #[test]
    fn タスク名のバリデーションが機能すること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTaskRepository::new();
            let task_id = TaskId::new(1).unwrap();
            let project_id = ProjectId::new(1).unwrap();
            
            // 空文字列の名前でタスクを作成（バリデーションエラー）
            let result = Task::new(task_id, project_id, "".to_string());
            assert!(result.is_err());
            
            // 空白のみの名前でタスクを作成（バリデーションエラー）
            let result = Task::new(task_id, project_id, "   ".to_string());
            assert!(result.is_err());
            
            // 正常な名前でタスクを作成
            let task = Task::new(task_id, project_id, "正常なタスク".to_string()).unwrap();
            repository.save(&task).await.unwrap();
            
            let found_task = repository.find_by_id(task_id).await.unwrap().unwrap();
            assert_eq!(found_task.name(), "正常なタスク");
        });
    }

    #[test]
    fn タスクのバージョン管理が正しく動作すること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTaskRepository::new();
            let task_id = TaskId::new(1).unwrap();
            let project_id = ProjectId::new(1).unwrap();
            
            let mut task = Task::new(task_id, project_id, "バージョンテスト".to_string()).unwrap();
            
            // 初期保存（バージョン1）
            repository.save(&task).await.unwrap();
            
            // 名前変更（バージョン2）
            task = task.change_name("バージョン2".to_string()).unwrap();
            repository.save(&task).await.unwrap();
            
            // アーカイブ（バージョン3）
            task = task.archive();
            repository.save(&task).await.unwrap();
            
            // 復元（バージョン4）
            task = task.restore().unwrap();
            repository.save(&task).await.unwrap();
            
            // 履歴を確認（4つのバージョンが存在することを確認）
            let history = repository.find_history(task_id).await.unwrap();
            assert_eq!(history.len(), 4);
            
            // 最新のタスクがアクティブであることを確認
            let current_task = repository.find_by_id(task_id).await.unwrap().unwrap();
            assert!(current_task.status().is_active());
            assert_eq!(current_task.name(), "バージョン2");
        });
    }

    #[test]
    fn 複数タスクの同時操作ができること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTaskRepository::new();
            let project_id = ProjectId::new(1).unwrap();
            
            // 複数のタスクを作成
            for i in 1..=5 {
                let task_id = TaskId::new(i).unwrap();
                let task = Task::new(task_id, project_id, format!("タスク{}", i)).unwrap();
                repository.save(&task).await.unwrap();
            }
            
            // すべてのタスクが保存されていることを確認
            let tasks = repository.find_by_project_id(project_id).await.unwrap();
            assert_eq!(tasks.len(), 5);
            
            for i in 1..=5 {
                assert!(tasks.iter().any(|t| t.name() == format!("タスク{}", i)));
            }
        });
    }

    #[test]
    fn タスクのプロジェクト移動ができること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTaskRepository::new();
            let task_id = TaskId::new(1).unwrap();
            let project1 = ProjectId::new(1).unwrap();
            let project2 = ProjectId::new(2).unwrap();
            
            let mut task = Task::new(task_id, project1, "移動テスト".to_string()).unwrap();
            repository.save(&task).await.unwrap();
            
            task = task.move_to_project(project2);
            repository.save(&task).await.unwrap();
            
            // 元のプロジェクトには存在しないことを確認
            let project1_tasks = repository.find_by_project_id(project1).await.unwrap();
            assert_eq!(project1_tasks.len(), 0);
            
            // 新しいプロジェクトに存在することを確認
            let project2_tasks = repository.find_by_project_id(project2).await.unwrap();
            assert_eq!(project2_tasks.len(), 1);
            assert_eq!(project2_tasks[0].name(), "移動テスト");
        });
    }
}

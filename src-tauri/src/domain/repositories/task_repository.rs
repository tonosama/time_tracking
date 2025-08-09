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
    #[tokio::test]
    async fn test_save_and_find_task() {
        let repo = InMemoryTaskRepository::new();
        let task_id = TaskId::new(1).unwrap();
        let project_id = ProjectId::new(1).unwrap();
        let task = Task::new(task_id, project_id, "Test Task".to_string()).unwrap();

        repo.save(&task).await.unwrap();
        let found = repo.find_by_id(task_id).await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "Test Task");
    }

    #[tokio::test]
    async fn test_find_by_project_id() {
        let repo = InMemoryTaskRepository::new();
        let project_id1 = ProjectId::new(1).unwrap();
        let project_id2 = ProjectId::new(2).unwrap();
        
        let task1 = Task::new(TaskId::new(1).unwrap(), project_id1, "Task 1".to_string()).unwrap();
        let task2 = Task::new(TaskId::new(2).unwrap(), project_id1, "Task 2".to_string()).unwrap();
        let task3 = Task::new(TaskId::new(3).unwrap(), project_id2, "Task 3".to_string()).unwrap();

        repo.save(&task1).await.unwrap();
        repo.save(&task2).await.unwrap();
        repo.save(&task3).await.unwrap();

        let project1_tasks = repo.find_by_project_id(project_id1).await.unwrap();
        let project2_tasks = repo.find_by_project_id(project_id2).await.unwrap();

        assert_eq!(project1_tasks.len(), 2);
        assert_eq!(project2_tasks.len(), 1);
    }

    #[tokio::test]
    async fn test_find_active_by_project_id() {
        let repo = InMemoryTaskRepository::new();
        let project_id = ProjectId::new(1).unwrap();
        
        let task1 = Task::new(TaskId::new(1).unwrap(), project_id, "Active Task".to_string()).unwrap();
        let task2 = Task::new(TaskId::new(2).unwrap(), project_id, "Archived Task".to_string()).unwrap();
        let archived_task2 = task2.archive();

        repo.save(&task1).await.unwrap();
        repo.save(&archived_task2).await.unwrap();

        let active_tasks = repo.find_active_by_project_id(project_id).await.unwrap();
        assert_eq!(active_tasks.len(), 1);
        assert_eq!(active_tasks[0].name(), "Active Task");
    }

    #[tokio::test]
    async fn test_task_history() {
        let repo = InMemoryTaskRepository::new();
        let task_id = TaskId::new(1).unwrap();
        let project_id = ProjectId::new(1).unwrap();
        
        let task1 = Task::new(task_id, project_id, "Original Name".to_string()).unwrap();
        let task2 = task1.change_name("Updated Name".to_string()).unwrap();
        let task3 = task2.archive();

        repo.save(&task1).await.unwrap();
        repo.save(&task2).await.unwrap();
        repo.save(&task3).await.unwrap();

        let history = repo.find_history(task_id).await.unwrap();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].name(), "Original Name");
        assert_eq!(history[1].name(), "Updated Name");
        assert_eq!(history[2].name(), "Updated Name");
        assert!(history[2].is_archived());
    }

    #[tokio::test]
    async fn test_count_by_project_id() {
        let repo = InMemoryTaskRepository::new();
        let project_id = ProjectId::new(1).unwrap();
        
        let task1 = Task::new(TaskId::new(1).unwrap(), project_id, "Task 1".to_string()).unwrap();
        let task2 = Task::new(TaskId::new(2).unwrap(), project_id, "Task 2".to_string()).unwrap();

        repo.save(&task1).await.unwrap();
        repo.save(&task2).await.unwrap();

        let count = repo.count_by_project_id(project_id).await.unwrap();
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_next_id_generation() {
        let repo = InMemoryTaskRepository::new();
        
        let id1 = repo.next_id().await.unwrap();
        let id2 = repo.next_id().await.unwrap();
        
        assert_eq!(id1.value(), 1);
        assert_eq!(id2.value(), 2);
    }
}

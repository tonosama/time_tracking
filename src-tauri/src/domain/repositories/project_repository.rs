use crate::domain::entities::Project;
use crate::domain::value_objects::{ProjectId, Status};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// プロジェクトリポジトリトレイト
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// プロジェクトを保存（新規作成またはバージョン追加）
    async fn save(&self, project: &Project) -> anyhow::Result<()>;

    /// プロジェクトIDで検索
    async fn find_by_id(&self, id: ProjectId) -> anyhow::Result<Option<Project>>;

    /// 全てのアクティブなプロジェクトを取得
    async fn find_all_active(&self) -> anyhow::Result<Vec<Project>>;

    /// 全てのプロジェクトを取得（ステータス関係なし）
    async fn find_all(&self) -> anyhow::Result<Vec<Project>>;

    /// 指定したステータスのプロジェクトを取得
    async fn find_by_status(&self, status: &Status) -> anyhow::Result<Vec<Project>>;

    /// プロジェクト名で検索（前方一致）
    async fn find_by_name_prefix(&self, prefix: &str) -> anyhow::Result<Vec<Project>>;

    /// 次に使用可能なプロジェクトIDを生成
    async fn next_id(&self) -> anyhow::Result<ProjectId>;

    /// プロジェクトの存在確認
    async fn exists(&self, id: ProjectId) -> anyhow::Result<bool>;

    /// プロジェクトの履歴を取得
    async fn find_history(&self, id: ProjectId) -> anyhow::Result<Vec<Project>>;

    /// 指定した時刻でのプロジェクト状態を取得
    async fn find_at_time(&self, id: ProjectId, at: DateTime<Utc>) -> anyhow::Result<Option<Project>>;
}

#[cfg(test)]
#[allow(non_snake_case)]
pub mod tests {
    use super::*;
    use crate::domain::entities::Project;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // テスト用のインメモリリポジトリ実装
    #[derive(Debug, Default, Clone)]
    pub struct InMemoryProjectRepository {
        projects: Arc<Mutex<HashMap<ProjectId, Vec<Project>>>>,
        next_id: Arc<Mutex<i64>>,
    }

    impl InMemoryProjectRepository {
        pub fn new() -> Self {
            Self {
                projects: Arc::new(Mutex::new(HashMap::new())),
                next_id: Arc::new(Mutex::new(1)),
            }
        }
    }

    #[async_trait]
    impl ProjectRepository for InMemoryProjectRepository {
        async fn save(&self, project: &Project) -> anyhow::Result<()> {
            let mut projects = self.projects.lock().await;
            let versions = projects.entry(project.id()).or_insert_with(Vec::new);
            versions.push(project.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: ProjectId) -> anyhow::Result<Option<Project>> {
            let projects = self.projects.lock().await;
            if let Some(versions) = projects.get(&id) {
                // 最新バージョンを返す（effective_at順）
                let latest = versions
                    .iter()
                    .max_by_key(|p| p.effective_at())
                    .cloned();
                Ok(latest)
            } else {
                Ok(None)
            }
        }

        async fn find_all_active(&self) -> anyhow::Result<Vec<Project>> {
            let projects = self.projects.lock().await;
            let mut result = Vec::new();
            
            for versions in projects.values() {
                if let Some(latest) = versions.iter().max_by_key(|p| p.effective_at()) {
                    if latest.is_active() {
                        result.push(latest.clone());
                    }
                }
            }
            
            Ok(result)
        }

        async fn find_all(&self) -> anyhow::Result<Vec<Project>> {
            let projects = self.projects.lock().await;
            let mut result = Vec::new();
            
            for versions in projects.values() {
                if let Some(latest) = versions.iter().max_by_key(|p| p.effective_at()) {
                    result.push(latest.clone());
                }
            }
            
            Ok(result)
        }

        async fn find_by_status(&self, status: &Status) -> anyhow::Result<Vec<Project>> {
            let projects = self.projects.lock().await;
            let mut result = Vec::new();
            
            for versions in projects.values() {
                if let Some(latest) = versions.iter().max_by_key(|p| p.effective_at()) {
                    if latest.status() == status {
                        result.push(latest.clone());
                    }
                }
            }
            
            Ok(result)
        }

        async fn find_by_name_prefix(&self, prefix: &str) -> anyhow::Result<Vec<Project>> {
            let projects = self.projects.lock().await;
            let mut result = Vec::new();
            
            for versions in projects.values() {
                if let Some(latest) = versions.iter().max_by_key(|p| p.effective_at()) {
                    if latest.name().starts_with(prefix) {
                        result.push(latest.clone());
                    }
                }
            }
            
            Ok(result)
        }

        async fn next_id(&self) -> anyhow::Result<ProjectId> {
            let mut next_id = self.next_id.lock().await;
            let id = *next_id;
            *next_id += 1;
            ProjectId::new(id)
        }

        async fn exists(&self, id: ProjectId) -> anyhow::Result<bool> {
            let projects = self.projects.lock().await;
            Ok(projects.contains_key(&id))
        }

        async fn find_history(&self, id: ProjectId) -> anyhow::Result<Vec<Project>> {
            let projects = self.projects.lock().await;
            if let Some(versions) = projects.get(&id) {
                let mut history = versions.clone();
                history.sort_by_key(|p| p.effective_at());
                Ok(history)
            } else {
                Ok(Vec::new())
            }
        }

        async fn find_at_time(&self, id: ProjectId, at: DateTime<Utc>) -> anyhow::Result<Option<Project>> {
            let projects = self.projects.lock().await;
            if let Some(versions) = projects.get(&id) {
                let project = versions
                    .iter()
                    .filter(|p| p.effective_at() <= at)
                    .max_by_key(|p| p.effective_at())
                    .cloned();
                Ok(project)
            } else {
                Ok(None)
            }
        }
    }

    // テストケース
    #[test]
    fn プロジェクトの保存と取得ができること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryProjectRepository::new();
            let project_id = ProjectId::new(1).unwrap();
            let project = Project::new(project_id, "テストプロジェクト".to_string()).unwrap();
            
            repository.save(&project).await.unwrap();
            
            let found = repository.find_by_id(project_id).await.unwrap().unwrap();
            assert_eq!(found.id(), project_id);
            assert_eq!(found.name(), "テストプロジェクト");
        });
    }

    #[test]
    fn 次のIDが正しく生成されること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryProjectRepository::new();
            
            let id1 = repository.next_id().await.unwrap();
            let id2 = repository.next_id().await.unwrap();
            let id3 = repository.next_id().await.unwrap();
            
            assert_eq!(id1, ProjectId::new(1).unwrap());
            assert_eq!(id2, ProjectId::new(2).unwrap());
            assert_eq!(id3, ProjectId::new(3).unwrap());
        });
    }

    #[test]
    fn アクティブプロジェクト一覧が取得できること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryProjectRepository::new();
            
            let project1 = Project::new(ProjectId::new(1).unwrap(), "プロジェクト1".to_string()).unwrap();
            let mut project2 = Project::new(ProjectId::new(2).unwrap(), "プロジェクト2".to_string()).unwrap();
            project2 = project2.archive();
            
            repository.save(&project1).await.unwrap();
            repository.save(&project2).await.unwrap();
            
            let active_projects = repository.find_all_active().await.unwrap();
            assert_eq!(active_projects.len(), 1);
            assert_eq!(active_projects[0].name(), "プロジェクト1");
        });
    }

    #[test]
    fn プロジェクト履歴が取得できること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryProjectRepository::new();
            let project_id = ProjectId::new(1).unwrap();
            
            let project1 = Project::new(project_id, "初期名".to_string()).unwrap();
            let project2 = project1.change_name("変更後".to_string()).unwrap();
            let project3 = project2.archive();
            
            repository.save(&project1).await.unwrap();
            repository.save(&project2).await.unwrap();
            repository.save(&project3).await.unwrap();
            
            let history = repository.find_history(project_id).await.unwrap();
            assert_eq!(history.len(), 3);
            assert_eq!(history[0].name(), "初期名");
            assert_eq!(history[1].name(), "変更後");
            assert_eq!(history[2].name(), "変更後");
            assert!(history[2].status().is_archived());
        });
    }

    // 追加のテストケース

    #[test]
    fn プロジェクトの複数回更新ができること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryProjectRepository::new();
            let project_id = ProjectId::new(1).unwrap();
            
            let mut project = Project::new(project_id, "初期名".to_string()).unwrap();
            repository.save(&project).await.unwrap();
            
            project = project.change_name("2回目".to_string()).unwrap();
            repository.save(&project).await.unwrap();
            
            project = project.change_name("3回目".to_string()).unwrap();
            repository.save(&project).await.unwrap();
            
            let history = repository.find_history(project_id).await.unwrap();
            assert_eq!(history.len(), 3);
            assert_eq!(history[0].name(), "初期名");
            assert_eq!(history[1].name(), "2回目");
            assert_eq!(history[2].name(), "3回目");
        });
    }

    #[test]
    fn プロジェクトのアーカイブと復元ができること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryProjectRepository::new();
            let project_id = ProjectId::new(1).unwrap();
            
            let mut project = Project::new(project_id, "復元テスト".to_string()).unwrap();
            repository.save(&project).await.unwrap();
            
            project = project.archive();
            repository.save(&project).await.unwrap();
            
            project = project.restore().unwrap();
            repository.save(&project).await.unwrap();
            
            let found_project = repository.find_by_id(project_id).await.unwrap().unwrap();
            assert!(found_project.status().is_active());
            
            let active_projects = repository.find_all_active().await.unwrap();
            assert_eq!(active_projects.len(), 1);
            assert_eq!(active_projects[0].name(), "復元テスト");
        });
    }

    #[test]
    fn 空のリポジトリでの動作確認() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryProjectRepository::new();
            
            let active_projects = repository.find_all_active().await.unwrap();
            assert_eq!(active_projects.len(), 0);
            
            let all_projects = repository.find_all().await.unwrap();
            assert_eq!(all_projects.len(), 0);
            
            let project_id = ProjectId::new(999).unwrap();
            let history = repository.find_history(project_id).await.unwrap();
            assert_eq!(history.len(), 0);
        });
    }

    #[test]
    fn プロジェクト名のバリデーションが機能すること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryProjectRepository::new();
            let project_id = ProjectId::new(1).unwrap();
            
            // 空文字列の名前でプロジェクトを作成（バリデーションエラー）
            let result = Project::new(project_id, "".to_string());
            assert!(result.is_err());
            
            // 空白のみの名前でプロジェクトを作成（バリデーションエラー）
            let result = Project::new(project_id, "   ".to_string());
            assert!(result.is_err());
            
            // 正常な名前でプロジェクトを作成
            let project = Project::new(project_id, "正常なプロジェクト".to_string()).unwrap();
            repository.save(&project).await.unwrap();
            
            let found_project = repository.find_by_id(project_id).await.unwrap().unwrap();
            assert_eq!(found_project.name(), "正常なプロジェクト");
        });
    }

    #[test]
    fn プロジェクトのバージョン管理が正しく動作すること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryProjectRepository::new();
            let project_id = ProjectId::new(1).unwrap();
            
            let mut project = Project::new(project_id, "バージョンテスト".to_string()).unwrap();
            
            // 初期保存（バージョン1）
            repository.save(&project).await.unwrap();
            
            // 名前変更（バージョン2）
            project = project.change_name("バージョン2".to_string()).unwrap();
            repository.save(&project).await.unwrap();
            
            // アーカイブ（バージョン3）
            project = project.archive();
            repository.save(&project).await.unwrap();
            
            // 復元（バージョン4）
            project = project.restore().unwrap();
            repository.save(&project).await.unwrap();
            
            // 履歴を確認（4つのバージョンが存在することを確認）
            let history = repository.find_history(project_id).await.unwrap();
            assert_eq!(history.len(), 4);
            
            // 最新のプロジェクトがアクティブであることを確認
            let current_project = repository.find_by_id(project_id).await.unwrap().unwrap();
            assert!(current_project.status().is_active());
            assert_eq!(current_project.name(), "バージョン2");
        });
    }

    #[test]
    fn 複数プロジェクトの同時操作ができること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryProjectRepository::new();
            
            // 複数のプロジェクトを作成
            for i in 1..=5 {
                let project_id = ProjectId::new(i).unwrap();
                let project = Project::new(project_id, format!("プロジェクト{}", i)).unwrap();
                repository.save(&project).await.unwrap();
            }
            
            // すべてのプロジェクトが保存されていることを確認
            let active_projects = repository.find_all_active().await.unwrap();
            assert_eq!(active_projects.len(), 5);
            
            for i in 1..=5 {
                assert!(active_projects.iter().any(|p| p.name() == format!("プロジェクト{}", i)));
            }
        });
    }
}

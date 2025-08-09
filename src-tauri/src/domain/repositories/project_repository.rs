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
    #[tokio::test]
    async fn test_save_and_find_project() {
        let repo = InMemoryProjectRepository::new();
        let id = ProjectId::new(1).unwrap();
        let project = Project::new(id, "Test Project".to_string()).unwrap();

        repo.save(&project).await.unwrap();
        let found = repo.find_by_id(id).await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "Test Project");
    }

    #[tokio::test]
    async fn test_find_all_active() {
        let repo = InMemoryProjectRepository::new();
        let id1 = ProjectId::new(1).unwrap();
        let id2 = ProjectId::new(2).unwrap();
        
        let project1 = Project::new(id1, "Active Project".to_string()).unwrap();
        let project2 = Project::new(id2, "Archived Project".to_string()).unwrap();
        let archived_project2 = project2.archive();

        repo.save(&project1).await.unwrap();
        repo.save(&archived_project2).await.unwrap();

        let active_projects = repo.find_all_active().await.unwrap();
        assert_eq!(active_projects.len(), 1);
        assert_eq!(active_projects[0].name(), "Active Project");
    }

    #[tokio::test]
    async fn test_next_id_generation() {
        let repo = InMemoryProjectRepository::new();
        
        let id1 = repo.next_id().await.unwrap();
        let id2 = repo.next_id().await.unwrap();
        
        assert_eq!(id1.value(), 1);
        assert_eq!(id2.value(), 2);
    }

    #[tokio::test]
    async fn test_project_history() {
        let repo = InMemoryProjectRepository::new();
        let id = ProjectId::new(1).unwrap();
        
        let project1 = Project::new(id, "Original Name".to_string()).unwrap();
        let project2 = project1.change_name("Updated Name".to_string()).unwrap();
        let project3 = project2.archive();

        repo.save(&project1).await.unwrap();
        repo.save(&project2).await.unwrap();
        repo.save(&project3).await.unwrap();

        let history = repo.find_history(id).await.unwrap();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].name(), "Original Name");
        assert_eq!(history[1].name(), "Updated Name");
        assert_eq!(history[2].name(), "Updated Name");
        assert!(history[2].is_archived());
    }
}

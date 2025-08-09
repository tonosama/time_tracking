use crate::domain::entities::Project;
use crate::domain::repositories::{ProjectRepository, TaskRepository};
use crate::domain::value_objects::ProjectId;
use async_trait::async_trait;

/// プロジェクト管理ドメインサービス
#[async_trait]
pub trait ProjectManagementService: Send + Sync {
    /// プロジェクトが削除可能かどうかを判定
    async fn can_archive_project(&self, project_id: ProjectId) -> anyhow::Result<bool>;
    
    /// プロジェクトの階層構造の整合性を検証
    async fn validate_project_hierarchy(&self, project: &Project) -> anyhow::Result<()>;
    
    /// プロジェクト名の重複をチェック
    async fn is_project_name_unique(&self, name: &str, exclude_id: Option<ProjectId>) -> anyhow::Result<bool>;
    
    /// プロジェクトアーカイブ時の関連タスクの処理
    async fn archive_project_with_tasks(&self, project_id: ProjectId) -> anyhow::Result<()>;
}

/// プロジェクト管理サービスの実装
pub struct ProjectManagementServiceImpl<P: ProjectRepository, T: TaskRepository> {
    project_repo: P,
    task_repo: T,
}

impl<P: ProjectRepository, T: TaskRepository> ProjectManagementServiceImpl<P, T> {
    pub fn new(project_repo: P, task_repo: T) -> Self {
        Self {
            project_repo,
            task_repo,
        }
    }
}

#[async_trait]
impl<P: ProjectRepository, T: TaskRepository> ProjectManagementService for ProjectManagementServiceImpl<P, T> {
    async fn can_archive_project(&self, project_id: ProjectId) -> anyhow::Result<bool> {
        // アクティブなタスクが存在しないかチェック
        let active_tasks = self.task_repo.find_active_by_project_id(project_id).await?;
        Ok(active_tasks.is_empty())
    }

    async fn validate_project_hierarchy(&self, project: &Project) -> anyhow::Result<()> {
        // プロジェクトの存在確認
        if !self.project_repo.exists(project.id()).await? {
            return Err(anyhow::anyhow!("Project does not exist"));
        }

        // プロジェクトがアーカイブ済みの場合、関連するタスクもアーカイブされているべき
        if project.is_archived() {
            let active_tasks = self.task_repo.find_active_by_project_id(project.id()).await?;
            if !active_tasks.is_empty() {
                return Err(anyhow::anyhow!(
                    "Archived project cannot have active tasks. Found {} active tasks",
                    active_tasks.len()
                ));
            }
        }

        Ok(())
    }

    async fn is_project_name_unique(&self, name: &str, exclude_id: Option<ProjectId>) -> anyhow::Result<bool> {
        let all_projects = self.project_repo.find_all().await?;
        
        for project in all_projects {
            if project.name() == name {
                if let Some(exclude) = exclude_id {
                    if project.id() != exclude {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
        }
        
        Ok(true)
    }

    async fn archive_project_with_tasks(&self, project_id: ProjectId) -> anyhow::Result<()> {
        // プロジェクトの存在確認
        let project = self.project_repo.find_by_id(project_id).await?
            .ok_or_else(|| anyhow::anyhow!("Project not found"))?;

        // 既にアーカイブ済みの場合はエラー
        if project.is_archived() {
            return Err(anyhow::anyhow!("Project is already archived"));
        }

        // 関連するすべてのアクティブタスクを取得してアーカイブ
        let active_tasks = self.task_repo.find_active_by_project_id(project_id).await?;
        for task in active_tasks {
            let archived_task = task.archive();
            self.task_repo.save(&archived_task).await?;
        }

        // プロジェクトをアーカイブ
        let archived_project = project.archive();
        self.project_repo.save(&archived_project).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{Project, Task};
    use crate::domain::value_objects::TaskId;
    use crate::domain::repositories::tests::InMemoryProjectRepository;
    use crate::domain::repositories::task_tests::InMemoryTaskRepository;

    async fn setup_service() -> ProjectManagementServiceImpl<InMemoryProjectRepository, InMemoryTaskRepository> {
        let project_repo = InMemoryProjectRepository::new();
        let task_repo = InMemoryTaskRepository::new();
        ProjectManagementServiceImpl::new(project_repo, task_repo)
    }

    #[tokio::test]
    async fn test_can_archive_project_with_no_active_tasks() {
        let service = setup_service().await;
        let project_id = ProjectId::new(1).unwrap();
        let project = Project::new(project_id, "Test Project".to_string()).unwrap();

        service.project_repo.save(&project).await.unwrap();

        let can_archive = service.can_archive_project(project_id).await.unwrap();
        assert!(can_archive);
    }

    #[tokio::test]
    async fn test_cannot_archive_project_with_active_tasks() {
        let service = setup_service().await;
        let project_id = ProjectId::new(1).unwrap();
        let task_id = TaskId::new(1).unwrap();
        
        let project = Project::new(project_id, "Test Project".to_string()).unwrap();
        let task = Task::new(task_id, project_id, "Active Task".to_string()).unwrap();

        service.project_repo.save(&project).await.unwrap();
        service.task_repo.save(&task).await.unwrap();

        let can_archive = service.can_archive_project(project_id).await.unwrap();
        assert!(!can_archive);
    }

    #[tokio::test]
    async fn test_can_archive_project_with_only_archived_tasks() {
        let service = setup_service().await;
        let project_id = ProjectId::new(1).unwrap();
        let task_id = TaskId::new(1).unwrap();
        
        let project = Project::new(project_id, "Test Project".to_string()).unwrap();
        let task = Task::new(task_id, project_id, "Test Task".to_string()).unwrap();
        let archived_task = task.archive();

        service.project_repo.save(&project).await.unwrap();
        service.task_repo.save(&archived_task).await.unwrap();

        let can_archive = service.can_archive_project(project_id).await.unwrap();
        assert!(can_archive);
    }

    #[tokio::test]
    async fn test_validate_project_hierarchy_success() {
        let service = setup_service().await;
        let project_id = ProjectId::new(1).unwrap();
        let project = Project::new(project_id, "Test Project".to_string()).unwrap();

        service.project_repo.save(&project).await.unwrap();

        let result = service.validate_project_hierarchy(&project).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_project_hierarchy_archived_with_active_tasks() {
        let service = setup_service().await;
        let project_id = ProjectId::new(1).unwrap();
        let task_id = TaskId::new(1).unwrap();
        
        let project = Project::new(project_id, "Test Project".to_string()).unwrap();
        let archived_project = project.archive();
        let task = Task::new(task_id, project_id, "Active Task".to_string()).unwrap();

        service.project_repo.save(&archived_project).await.unwrap();
        service.task_repo.save(&task).await.unwrap();

        let result = service.validate_project_hierarchy(&archived_project).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_is_project_name_unique() {
        let service = setup_service().await;
        let project_id = ProjectId::new(1).unwrap();
        let project = Project::new(project_id, "Existing Project".to_string()).unwrap();

        service.project_repo.save(&project).await.unwrap();

        // 新しい名前は一意
        let is_unique = service.is_project_name_unique("New Project", None).await.unwrap();
        assert!(is_unique);

        // 既存の名前は一意ではない
        let is_unique = service.is_project_name_unique("Existing Project", None).await.unwrap();
        assert!(!is_unique);

        // 同じIDを除外する場合は一意
        let is_unique = service.is_project_name_unique("Existing Project", Some(project_id)).await.unwrap();
        assert!(is_unique);
    }

    #[tokio::test]
    async fn test_archive_project_with_tasks() {
        let service = setup_service().await;
        let project_id = ProjectId::new(1).unwrap();
        let task_id = TaskId::new(1).unwrap();
        
        let project = Project::new(project_id, "Test Project".to_string()).unwrap();
        let task = Task::new(task_id, project_id, "Test Task".to_string()).unwrap();

        service.project_repo.save(&project).await.unwrap();
        service.task_repo.save(&task).await.unwrap();

        service.archive_project_with_tasks(project_id).await.unwrap();

        // プロジェクトがアーカイブされていることを確認
        let archived_project = service.project_repo.find_by_id(project_id).await.unwrap().unwrap();
        assert!(archived_project.is_archived());

        // タスクもアーカイブされていることを確認
        let archived_task = service.task_repo.find_by_id(task_id).await.unwrap().unwrap();
        assert!(archived_task.is_archived());
    }

    #[tokio::test]
    async fn test_archive_already_archived_project() {
        let service = setup_service().await;
        let project_id = ProjectId::new(1).unwrap();
        let project = Project::new(project_id, "Test Project".to_string()).unwrap();
        let archived_project = project.archive();

        service.project_repo.save(&archived_project).await.unwrap();

        let result = service.archive_project_with_tasks(project_id).await;
        assert!(result.is_err());
    }
}

use crate::domain::entities::Project;
use crate::domain::repositories::ProjectRepository;
use crate::domain::services::ProjectManagementService;
use crate::domain::value_objects::ProjectId;
use async_trait::async_trait;

/// プロジェクト作成コマンド
#[derive(Debug, Clone)]
pub struct CreateProjectCommand {
    pub name: String,
}

/// プロジェクト更新コマンド
#[derive(Debug, Clone)]
pub struct UpdateProjectCommand {
    pub id: ProjectId,
    pub name: String,
}

/// プロジェクトアーカイブコマンド
#[derive(Debug, Clone)]
pub struct ArchiveProjectCommand {
    pub id: ProjectId,
    pub force: bool, // 関連タスクも一緒にアーカイブするかどうか
}

/// プロジェクト復元コマンド
#[derive(Debug, Clone)]
pub struct RestoreProjectCommand {
    pub id: ProjectId,
}

/// プロジェクトユースケーストレイト
#[async_trait]
pub trait ProjectUseCases: Send + Sync {
    /// プロジェクトを作成する
    async fn create_project(&self, command: CreateProjectCommand) -> anyhow::Result<Project>;
    
    /// プロジェクトを更新する
    async fn update_project(&self, command: UpdateProjectCommand) -> anyhow::Result<Project>;
    
    /// プロジェクトをアーカイブする
    async fn archive_project(&self, command: ArchiveProjectCommand) -> anyhow::Result<()>;
    
    /// プロジェクトを復元する
    async fn restore_project(&self, command: RestoreProjectCommand) -> anyhow::Result<Project>;
    
    /// プロジェクトを取得する
    async fn get_project(&self, id: ProjectId) -> anyhow::Result<Option<Project>>;
    
    /// 全てのアクティブプロジェクトを取得する
    async fn get_all_active_projects(&self) -> anyhow::Result<Vec<Project>>;
    
    /// 全てのプロジェクトを取得する
    async fn get_all_projects(&self) -> anyhow::Result<Vec<Project>>;
    
    /// プロジェクトの履歴を取得する
    async fn get_project_history(&self, id: ProjectId) -> anyhow::Result<Vec<Project>>;
}

/// プロジェクトユースケース実装
pub struct ProjectUseCasesImpl<R: ProjectRepository, S: ProjectManagementService> {
    repository: R,
    service: S,
}

impl<R: ProjectRepository, S: ProjectManagementService> ProjectUseCasesImpl<R, S> {
    pub fn new(repository: R, service: S) -> Self {
        Self { repository, service }
    }
}

#[async_trait]
impl<R: ProjectRepository, S: ProjectManagementService> ProjectUseCases for ProjectUseCasesImpl<R, S> {
    async fn create_project(&self, command: CreateProjectCommand) -> anyhow::Result<Project> {
        // プロジェクト名の一意性をチェック
        if !self.service.is_project_name_unique(&command.name, None).await? {
            return Err(anyhow::anyhow!("Project name '{}' already exists", command.name));
        }

        // 新しいIDを生成
        let id = self.repository.next_id().await?;
        
        // プロジェクトを作成
        let project = Project::new(id, command.name)?;
        
        // 保存
        self.repository.save(&project).await?;
        
        Ok(project)
    }

    async fn update_project(&self, command: UpdateProjectCommand) -> anyhow::Result<Project> {
        // 既存のプロジェクトを取得
        let existing_project = self.repository.find_by_id(command.id).await?
            .ok_or_else(|| anyhow::anyhow!("Project not found"))?;

        // アーカイブ済みのプロジェクトは更新できない
        if existing_project.is_archived() {
            return Err(anyhow::anyhow!("Cannot update archived project"));
        }

        // 名前が変更される場合は一意性をチェック
        if existing_project.name() != command.name {
            if !self.service.is_project_name_unique(&command.name, Some(command.id)).await? {
                return Err(anyhow::anyhow!("Project name '{}' already exists", command.name));
            }
        }

        // プロジェクト名を変更
        let updated_project = existing_project.change_name(command.name)?;
        
        // 保存
        self.repository.save(&updated_project).await?;
        
        Ok(updated_project)
    }

    async fn archive_project(&self, command: ArchiveProjectCommand) -> anyhow::Result<()> {
        // 既存のプロジェクトを取得
        let existing_project = self.repository.find_by_id(command.id).await?
            .ok_or_else(|| anyhow::anyhow!("Project not found"))?;

        // 既にアーカイブ済みの場合はエラー
        if existing_project.is_archived() {
            return Err(anyhow::anyhow!("Project is already archived"));
        }

        if command.force {
            // 関連タスクも一緒にアーカイブ
            self.service.archive_project_with_tasks(command.id).await?;
        } else {
            // アクティブなタスクがないかチェック
            if !self.service.can_archive_project(command.id).await? {
                return Err(anyhow::anyhow!(
                    "Cannot archive project with active tasks. Use force=true to archive with tasks."
                ));
            }

            // プロジェクトのみアーカイブ
            let archived_project = existing_project.archive();
            self.repository.save(&archived_project).await?;
        }

        Ok(())
    }

    async fn restore_project(&self, command: RestoreProjectCommand) -> anyhow::Result<Project> {
        // 既存のプロジェクトを取得
        let existing_project = self.repository.find_by_id(command.id).await?
            .ok_or_else(|| anyhow::anyhow!("Project not found"))?;

        // プロジェクトを復元
        let restored_project = existing_project.restore()?;
        
        // 保存
        self.repository.save(&restored_project).await?;
        
        Ok(restored_project)
    }

    async fn get_project(&self, id: ProjectId) -> anyhow::Result<Option<Project>> {
        self.repository.find_by_id(id).await
    }

    async fn get_all_active_projects(&self) -> anyhow::Result<Vec<Project>> {
        self.repository.find_all_active().await
    }

    async fn get_all_projects(&self) -> anyhow::Result<Vec<Project>> {
        self.repository.find_all().await
    }

    async fn get_project_history(&self, id: ProjectId) -> anyhow::Result<Vec<Project>> {
        self.repository.find_history(id).await
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::domain::repositories::tests::InMemoryProjectRepository;
    use crate::domain::repositories::task_tests::InMemoryTaskRepository;
    use crate::domain::services::ProjectManagementServiceImpl;

    async fn setup_use_cases() -> ProjectUseCasesImpl<InMemoryProjectRepository, ProjectManagementServiceImpl<InMemoryProjectRepository, InMemoryTaskRepository>> {
        let project_repo = InMemoryProjectRepository::new();
        let task_repo = InMemoryTaskRepository::new();
        let service = ProjectManagementServiceImpl::new(project_repo.clone(), task_repo);
        ProjectUseCasesImpl::new(project_repo, service)
    }

    #[tokio::test]
    async fn プロジェクト作成が成功すること() {
        let use_cases = setup_use_cases().await;
        let command = CreateProjectCommand {
            name: "Test Project".to_string(),
        };

        let result = use_cases.create_project(command).await;
        assert!(result.is_ok());

        let project = result.unwrap();
        assert_eq!(project.name(), "Test Project");
        assert!(project.is_active());
    }

    #[tokio::test]
    async fn 重複名プロジェクト作成が失敗すること() {
        let use_cases = setup_use_cases().await;
        let command1 = CreateProjectCommand {
            name: "Test Project".to_string(),
        };
        let command2 = CreateProjectCommand {
            name: "Test Project".to_string(),
        };

        // 最初のプロジェクトは成功
        use_cases.create_project(command1).await.unwrap();

        // 同じ名前の2番目のプロジェクトは失敗
        let result = use_cases.create_project(command2).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn プロジェクト更新が成功すること() {
        let use_cases = setup_use_cases().await;
        
        // プロジェクトを作成
        let create_command = CreateProjectCommand {
            name: "Original Name".to_string(),
        };
        let project = use_cases.create_project(create_command).await.unwrap();

        // プロジェクトを更新
        let update_command = UpdateProjectCommand {
            id: project.id(),
            name: "Updated Name".to_string(),
        };
        let result = use_cases.update_project(update_command).await;
        assert!(result.is_ok());

        let updated_project = result.unwrap();
        assert_eq!(updated_project.name(), "Updated Name");
        assert_eq!(updated_project.id(), project.id());
    }

    #[tokio::test]
    async fn 存在しないプロジェクト更新が失敗すること() {
        let use_cases = setup_use_cases().await;
        let command = UpdateProjectCommand {
            id: ProjectId::new(999).unwrap(),
            name: "New Name".to_string(),
        };

        let result = use_cases.update_project(command).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn 重複名でのプロジェクト更新が失敗すること() {
        let use_cases = setup_use_cases().await;
        
        // 2つのプロジェクトを作成
        let project1 = use_cases.create_project(CreateProjectCommand {
            name: "Project 1".to_string(),
        }).await.unwrap();
        
        use_cases.create_project(CreateProjectCommand {
            name: "Project 2".to_string(),
        }).await.unwrap();

        // プロジェクト1を既存のプロジェクト2の名前に変更しようとする
        let update_command = UpdateProjectCommand {
            id: project1.id(),
            name: "Project 2".to_string(),
        };
        let result = use_cases.update_project(update_command).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn プロジェクトアーカイブが成功すること() {
        let use_cases = setup_use_cases().await;
        
        // プロジェクトを作成
        let project = use_cases.create_project(CreateProjectCommand {
            name: "Test Project".to_string(),
        }).await.unwrap();

        // プロジェクトをアーカイブ
        let command = ArchiveProjectCommand {
            id: project.id(),
            force: false,
        };
        let result = use_cases.archive_project(command).await;
        assert!(result.is_ok());

        // アーカイブされたことを確認
        let archived_project = use_cases.get_project(project.id()).await.unwrap().unwrap();
        assert!(archived_project.is_archived());
    }

    #[tokio::test]
    async fn プロジェクト復元が成功すること() {
        let use_cases = setup_use_cases().await;
        
        // プロジェクトを作成してアーカイブ
        let project = use_cases.create_project(CreateProjectCommand {
            name: "Test Project".to_string(),
        }).await.unwrap();
        
        use_cases.archive_project(ArchiveProjectCommand {
            id: project.id(),
            force: false,
        }).await.unwrap();

        // プロジェクトを復元
        let command = RestoreProjectCommand {
            id: project.id(),
        };
        let result = use_cases.restore_project(command).await;
        assert!(result.is_ok());

        let restored_project = result.unwrap();
        assert!(restored_project.is_active());
    }

    #[tokio::test]
    async fn アクティブプロジェクト一覧取得が成功すること() {
        let use_cases = setup_use_cases().await;
        
        // アクティブなプロジェクトを作成
        let active_project = use_cases.create_project(CreateProjectCommand {
            name: "Active Project".to_string(),
        }).await.unwrap();
        
        // アーカイブ済みプロジェクトを作成
        let archived_project = use_cases.create_project(CreateProjectCommand {
            name: "Archived Project".to_string(),
        }).await.unwrap();
        
        use_cases.archive_project(ArchiveProjectCommand {
            id: archived_project.id(),
            force: false,
        }).await.unwrap();

        // アクティブなプロジェクトのみ取得
        let active_projects = use_cases.get_all_active_projects().await.unwrap();
        assert_eq!(active_projects.len(), 1);
        assert_eq!(active_projects[0].id(), active_project.id());
    }

    #[tokio::test]
    async fn プロジェクト履歴取得が成功すること() {
        let use_cases = setup_use_cases().await;
        
        // プロジェクトを作成
        let project = use_cases.create_project(CreateProjectCommand {
            name: "Original Name".to_string(),
        }).await.unwrap();

        // プロジェクトを更新
        use_cases.update_project(UpdateProjectCommand {
            id: project.id(),
            name: "Updated Name".to_string(),
        }).await.unwrap();

        // 履歴を取得
        let history = use_cases.get_project_history(project.id()).await.unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].name(), "Original Name");
        assert_eq!(history[1].name(), "Updated Name");
    }
}

use crate::domain::entities::Task;
use crate::domain::repositories::{TaskRepository, ProjectRepository};
use crate::domain::value_objects::{ProjectId, TaskId};
use async_trait::async_trait;

/// タスク作成コマンド
#[derive(Debug, Clone)]
pub struct CreateTaskCommand {
    pub project_id: ProjectId,
    pub name: String,
}

/// タスク更新コマンド
#[derive(Debug, Clone)]
pub struct UpdateTaskCommand {
    pub id: TaskId,
    pub name: Option<String>,
    pub project_id: Option<ProjectId>,
}

/// タスクアーカイブコマンド
#[derive(Debug, Clone)]
pub struct ArchiveTaskCommand {
    pub id: TaskId,
}

/// タスク復元コマンド
#[derive(Debug, Clone)]
pub struct RestoreTaskCommand {
    pub id: TaskId,
}

/// タスクユースケーストレイト
#[async_trait]
pub trait TaskUseCases: Send + Sync {
    /// タスクを作成する
    async fn create_task(&self, command: CreateTaskCommand) -> anyhow::Result<Task>;
    
    /// タスクを更新する
    async fn update_task(&self, command: UpdateTaskCommand) -> anyhow::Result<Task>;
    
    /// タスクをアーカイブする
    async fn archive_task(&self, command: ArchiveTaskCommand) -> anyhow::Result<()>;
    
    /// タスクを復元する
    async fn restore_task(&self, command: RestoreTaskCommand) -> anyhow::Result<Task>;
    
    /// タスクを取得する
    async fn get_task(&self, id: TaskId) -> anyhow::Result<Option<Task>>;
    
    /// プロジェクトのタスク一覧を取得する
    async fn get_tasks_by_project(&self, project_id: ProjectId) -> anyhow::Result<Vec<Task>>;
    
    /// プロジェクトのアクティブなタスク一覧を取得する
    async fn get_active_tasks_by_project(&self, project_id: ProjectId) -> anyhow::Result<Vec<Task>>;
    
    /// 全てのアクティブタスクを取得する
    async fn get_all_active_tasks(&self) -> anyhow::Result<Vec<Task>>;
    
    /// タスクの履歴を取得する
    async fn get_task_history(&self, id: TaskId) -> anyhow::Result<Vec<Task>>;
    
    /// タスクを別のプロジェクトに移動する
    async fn move_task_to_project(&self, task_id: TaskId, new_project_id: ProjectId) -> anyhow::Result<Task>;
}

/// タスクユースケース実装
pub struct TaskUseCasesImpl<T: TaskRepository, P: ProjectRepository> {
    task_repository: T,
    project_repository: P,
}

impl<T: TaskRepository, P: ProjectRepository> TaskUseCasesImpl<T, P> {
    pub fn new(task_repository: T, project_repository: P) -> Self {
        Self {
            task_repository,
            project_repository,
        }
    }
}

#[async_trait]
impl<T: TaskRepository, P: ProjectRepository> TaskUseCases for TaskUseCasesImpl<T, P> {
    async fn create_task(&self, command: CreateTaskCommand) -> anyhow::Result<Task> {
        // プロジェクトの存在確認
        let project = self.project_repository.find_by_id(command.project_id).await?
            .ok_or_else(|| anyhow::anyhow!("Project not found"))?;

        // アーカイブ済みプロジェクトにはタスクを作成できない
        if project.is_archived() {
            return Err(anyhow::anyhow!("Cannot create task in archived project"));
        }

        // 新しいIDを生成
        let id = self.task_repository.next_id().await?;
        
        // タスクを作成
        let task = Task::new(id, command.project_id, command.name)?;
        
        // 保存
        self.task_repository.save(&task).await?;
        
        Ok(task)
    }

    async fn update_task(&self, command: UpdateTaskCommand) -> anyhow::Result<Task> {
        // 既存のタスクを取得
        let mut existing_task = self.task_repository.find_by_id(command.id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        // アーカイブ済みのタスクは更新できない
        if existing_task.is_archived() {
            return Err(anyhow::anyhow!("Cannot update archived task"));
        }

        // 名前を更新
        if let Some(name) = command.name {
            existing_task = existing_task.change_name(name)?;
        }

        // プロジェクトを移動
        if let Some(new_project_id) = command.project_id {
            // 移動先プロジェクトの存在確認
            let target_project = self.project_repository.find_by_id(new_project_id).await?
                .ok_or_else(|| anyhow::anyhow!("Target project not found"))?;

            // アーカイブ済みプロジェクトには移動できない
            if target_project.is_archived() {
                return Err(anyhow::anyhow!("Cannot move task to archived project"));
            }

            existing_task = existing_task.move_to_project(new_project_id);
        }

        // 保存
        self.task_repository.save(&existing_task).await?;
        
        Ok(existing_task)
    }

    async fn archive_task(&self, command: ArchiveTaskCommand) -> anyhow::Result<()> {
        // 既存のタスクを取得
        let existing_task = self.task_repository.find_by_id(command.id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        // 既にアーカイブ済みの場合はエラー
        if existing_task.is_archived() {
            return Err(anyhow::anyhow!("Task is already archived"));
        }

        // タスクをアーカイブ
        let archived_task = existing_task.archive();
        self.task_repository.save(&archived_task).await?;

        Ok(())
    }

    async fn restore_task(&self, command: RestoreTaskCommand) -> anyhow::Result<Task> {
        // 既存のタスクを取得
        let existing_task = self.task_repository.find_by_id(command.id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        // 所属するプロジェクトがアクティブか確認
        let project = self.project_repository.find_by_id(existing_task.project_id()).await?
            .ok_or_else(|| anyhow::anyhow!("Project not found"))?;

        if project.is_archived() {
            return Err(anyhow::anyhow!("Cannot restore task in archived project"));
        }

        // タスクを復元
        let restored_task = existing_task.restore()?;
        
        // 保存
        self.task_repository.save(&restored_task).await?;
        
        Ok(restored_task)
    }

    async fn get_task(&self, id: TaskId) -> anyhow::Result<Option<Task>> {
        self.task_repository.find_by_id(id).await
    }

    async fn get_tasks_by_project(&self, project_id: ProjectId) -> anyhow::Result<Vec<Task>> {
        self.task_repository.find_by_project_id_ordered(project_id).await
    }

    async fn get_active_tasks_by_project(&self, project_id: ProjectId) -> anyhow::Result<Vec<Task>> {
        self.task_repository.find_active_by_project_id(project_id).await
    }

    async fn get_all_active_tasks(&self) -> anyhow::Result<Vec<Task>> {
        self.task_repository.find_all_active().await
    }

    async fn get_task_history(&self, id: TaskId) -> anyhow::Result<Vec<Task>> {
        self.task_repository.find_history(id).await
    }

    async fn move_task_to_project(&self, task_id: TaskId, new_project_id: ProjectId) -> anyhow::Result<Task> {
        let command = UpdateTaskCommand {
            id: task_id,
            name: None,
            project_id: Some(new_project_id),
        };
        self.update_task(command).await
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::domain::entities::Project;
    use crate::domain::repositories::tests::InMemoryProjectRepository;
    use crate::domain::repositories::task_tests::InMemoryTaskRepository;

    async fn setup_use_cases() -> (TaskUseCasesImpl<InMemoryTaskRepository, InMemoryProjectRepository>, ProjectId) {
        let task_repo = InMemoryTaskRepository::new();
        let project_repo = InMemoryProjectRepository::new();
        
        // テスト用プロジェクトを作成
        let project_id = ProjectId::new(1).unwrap();
        let project = Project::new(project_id, "Test Project".to_string()).unwrap();
        project_repo.save(&project).await.unwrap();
        
        let use_cases = TaskUseCasesImpl::new(task_repo, project_repo);
        (use_cases, project_id)
    }

    #[tokio::test]
    async fn タスク作成が成功すること() {
        let (use_cases, project_id) = setup_use_cases().await;
        let command = CreateTaskCommand {
            project_id,
            name: "Test Task".to_string(),
        };

        let result = use_cases.create_task(command).await;
        assert!(result.is_ok());

        let task = result.unwrap();
        assert_eq!(task.name(), "Test Task");
        assert_eq!(task.project_id(), project_id);
        assert!(task.is_active());
    }

    #[tokio::test]
    async fn 存在しないプロジェクトでタスク作成が失敗すること() {
        let (use_cases, _) = setup_use_cases().await;
        let command = CreateTaskCommand {
            project_id: ProjectId::new(999).unwrap(),
            name: "Test Task".to_string(),
        };

        let result = use_cases.create_task(command).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn アーカイブプロジェクトでタスク作成が失敗すること() {
        let (use_cases, project_id) = setup_use_cases().await;
        
        // プロジェクトをアーカイブ
        let project = use_cases.project_repository.find_by_id(project_id).await.unwrap().unwrap();
        let archived_project = project.archive();
        use_cases.project_repository.save(&archived_project).await.unwrap();

        let command = CreateTaskCommand {
            project_id,
            name: "Test Task".to_string(),
        };

        let result = use_cases.create_task(command).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn タスク名更新が成功すること() {
        let (use_cases, project_id) = setup_use_cases().await;
        
        // タスクを作成
        let task = use_cases.create_task(CreateTaskCommand {
            project_id,
            name: "Original Name".to_string(),
        }).await.unwrap();

        // タスク名を更新
        let command = UpdateTaskCommand {
            id: task.id(),
            name: Some("Updated Name".to_string()),
            project_id: None,
        };
        let result = use_cases.update_task(command).await;
        assert!(result.is_ok());

        let updated_task = result.unwrap();
        assert_eq!(updated_task.name(), "Updated Name");
        assert_eq!(updated_task.id(), task.id());
    }

    #[tokio::test]
    async fn タスクのプロジェクト移動が成功すること() {
        let (use_cases, project_id) = setup_use_cases().await;
        
        // 新しいプロジェクトを作成
        let new_project_id = ProjectId::new(2).unwrap();
        let new_project = Project::new(new_project_id, "New Project".to_string()).unwrap();
        use_cases.project_repository.save(&new_project).await.unwrap();
        
        // タスクを作成
        let task = use_cases.create_task(CreateTaskCommand {
            project_id,
            name: "Test Task".to_string(),
        }).await.unwrap();

        // タスクを移動
        let command = UpdateTaskCommand {
            id: task.id(),
            name: None,
            project_id: Some(new_project_id),
        };
        let result = use_cases.update_task(command).await;
        assert!(result.is_ok());

        let moved_task = result.unwrap();
        assert_eq!(moved_task.project_id(), new_project_id);
    }

    #[tokio::test]
    async fn タスクアーカイブが成功すること() {
        let (use_cases, project_id) = setup_use_cases().await;
        
        // タスクを作成
        let task = use_cases.create_task(CreateTaskCommand {
            project_id,
            name: "Test Task".to_string(),
        }).await.unwrap();

        // タスクをアーカイブ
        let command = ArchiveTaskCommand {
            id: task.id(),
        };
        let result = use_cases.archive_task(command).await;
        assert!(result.is_ok());

        // アーカイブされたことを確認
        let archived_task = use_cases.get_task(task.id()).await.unwrap().unwrap();
        assert!(archived_task.is_archived());
    }

    #[tokio::test]
    async fn タスク復元が成功すること() {
        let (use_cases, project_id) = setup_use_cases().await;
        
        // タスクを作成してアーカイブ
        let task = use_cases.create_task(CreateTaskCommand {
            project_id,
            name: "Test Task".to_string(),
        }).await.unwrap();
        
        use_cases.archive_task(ArchiveTaskCommand {
            id: task.id(),
        }).await.unwrap();

        // タスクを復元
        let command = RestoreTaskCommand {
            id: task.id(),
        };
        let result = use_cases.restore_task(command).await;
        assert!(result.is_ok());

        let restored_task = result.unwrap();
        assert!(restored_task.is_active());
    }

    #[tokio::test]
    async fn アーカイブプロジェクトでタスク復元が失敗すること() {
        let (use_cases, project_id) = setup_use_cases().await;
        
        // タスクを作成してアーカイブ
        let task = use_cases.create_task(CreateTaskCommand {
            project_id,
            name: "Test Task".to_string(),
        }).await.unwrap();
        
        use_cases.archive_task(ArchiveTaskCommand {
            id: task.id(),
        }).await.unwrap();

        // プロジェクトをアーカイブ
        let project = use_cases.project_repository.find_by_id(project_id).await.unwrap().unwrap();
        let archived_project = project.archive();
        use_cases.project_repository.save(&archived_project).await.unwrap();

        // アーカイブ済みプロジェクトのタスクは復元できない
        let command = RestoreTaskCommand {
            id: task.id(),
        };
        let result = use_cases.restore_task(command).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn プロジェクトのアクティブタスク一覧取得が成功すること() {
        let (use_cases, project_id) = setup_use_cases().await;
        
        // アクティブなタスクを作成
        let active_task = use_cases.create_task(CreateTaskCommand {
            project_id,
            name: "Active Task".to_string(),
        }).await.unwrap();
        
        // アーカイブ済みタスクを作成
        let archived_task = use_cases.create_task(CreateTaskCommand {
            project_id,
            name: "Archived Task".to_string(),
        }).await.unwrap();
        
        use_cases.archive_task(ArchiveTaskCommand {
            id: archived_task.id(),
        }).await.unwrap();

        // アクティブなタスクのみ取得
        let active_tasks = use_cases.get_active_tasks_by_project(project_id).await.unwrap();
        assert_eq!(active_tasks.len(), 1);
        assert_eq!(active_tasks[0].id(), active_task.id());
    }

    #[tokio::test]
    async fn タスク履歴取得が成功すること() {
        let (use_cases, project_id) = setup_use_cases().await;
        
        // タスクを作成
        let task = use_cases.create_task(CreateTaskCommand {
            project_id,
            name: "Original Name".to_string(),
        }).await.unwrap();

        // タスクを更新
        use_cases.update_task(UpdateTaskCommand {
            id: task.id(),
            name: Some("Updated Name".to_string()),
            project_id: None,
        }).await.unwrap();

        // 履歴を取得
        let history = use_cases.get_task_history(task.id()).await.unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].name(), "Original Name");
        assert_eq!(history[1].name(), "Updated Name");
    }
}

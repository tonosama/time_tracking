use crate::application::use_cases::{ProjectUseCases, TaskUseCases};
use crate::infrastructure::database::DatabaseConnection;
use crate::infrastructure::repositories::{SqliteProjectRepository, SqliteTaskRepository};
use std::sync::Arc;
use tokio::sync::Mutex;

/// アプリケーションサービス - 依存性の注入とライフサイクル管理
pub struct ApplicationService {
    db: Arc<Mutex<DatabaseConnection>>,
}

impl ApplicationService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
        }
    }

    /// プロジェクトユースケースを作成
    pub fn create_project_use_cases(&self) -> impl ProjectUseCases {
        let project_repo = SqliteProjectRepository::new(self.db.clone());
        let task_repo = SqliteTaskRepository::new(self.db.clone());
        let project_service = crate::domain::services::ProjectManagementServiceImpl::new(
            project_repo.clone(),
            task_repo,
        );
        
        crate::application::use_cases::ProjectUseCasesImpl::new(project_repo, project_service)
    }

    /// タスクユースケースを作成
    pub fn create_task_use_cases(&self) -> impl TaskUseCases {
        let project_repo = SqliteProjectRepository::new(self.db.clone());
        let task_repo = SqliteTaskRepository::new(self.db.clone());
        
        crate::application::use_cases::TaskUseCasesImpl::new(task_repo, project_repo)
    }

    /// データベース接続を取得
    pub fn database(&self) -> Arc<Mutex<DatabaseConnection>> {
        self.db.clone()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn アプリケーションサービス作成ができること() {
        let db = DatabaseConnection::new_in_memory().unwrap();
        db.run_migrations().unwrap();
        
        let app_service = ApplicationService::new(db);
        
        // ユースケースが作成できることを確認
        let _project_use_cases = app_service.create_project_use_cases();
        let _task_use_cases = app_service.create_task_use_cases();
    }
}

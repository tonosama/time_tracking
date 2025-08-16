use crate::application::use_cases::{ProjectUseCases, TaskUseCases, TimeTrackingUseCases};
use crate::infrastructure::config::Config;
use crate::infrastructure::database::DatabaseConnection;
use crate::infrastructure::repositories::{SqliteProjectRepository, SqliteTaskRepository, SqliteTimeEntryRepository};
use std::sync::Arc;
use tokio::sync::Mutex;

/// アプリケーションサービス - 依存性の注入とライフサイクル管理
pub struct ApplicationService {
    db: Arc<Mutex<DatabaseConnection>>,
    project_use_cases: Box<dyn ProjectUseCases>,
    task_use_cases: Box<dyn TaskUseCases>,
    time_tracking_use_cases: Box<dyn TimeTrackingUseCases>,
}

impl ApplicationService {
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        tracing::info!("ApplicationService::new: Starting application service initialization");
        tracing::info!("ApplicationService::new: Config - database_path: {:?}", config.database_path);
        
        tracing::debug!("ApplicationService::new: Creating database connection");
        let db = match DatabaseConnection::new(&config.database_path) {
            Ok(db) => {
                tracing::info!("ApplicationService::new: Database connection created successfully");
                db
            },
            Err(e) => {
                tracing::error!("ApplicationService::new: Failed to create database connection: {}", e);
                return Err(e);
            }
        };
        
        tracing::debug!("ApplicationService::new: Running database migrations");
        if let Err(e) = db.run_migrations() {
            tracing::error!("ApplicationService::new: Failed to run database migrations: {}", e);
            return Err(e);
        }
        tracing::info!("ApplicationService::new: Database migrations completed successfully");
        
        tracing::debug!("ApplicationService::new: Creating Arc<Mutex<DatabaseConnection>>");
        let db_arc = Arc::new(Mutex::new(db));
        tracing::debug!("ApplicationService::new: Database connection wrapped in Arc<Mutex>");
        
        // リポジトリを作成
        tracing::debug!("ApplicationService::new: Creating repositories");
        let project_repo = SqliteProjectRepository::new(db_arc.clone());
        tracing::debug!("ApplicationService::new: Project repository created");
        
        let task_repo = SqliteTaskRepository::new(db_arc.clone());
        tracing::debug!("ApplicationService::new: Task repository created");
        
        let time_entry_repo = SqliteTimeEntryRepository::new(db_arc.clone());
        tracing::debug!("ApplicationService::new: Time entry repository created");
        
        // ドメインサービスを作成
        tracing::debug!("ApplicationService::new: Creating domain services");
        let project_service = crate::domain::services::ProjectManagementServiceImpl::new(
            project_repo.clone(),
            task_repo.clone(),
        );
        tracing::debug!("ApplicationService::new: Project management service created");
        
        let time_tracking_service = crate::domain::services::TimeTrackingServiceImpl::new(
            time_entry_repo.clone(),
        );
        tracing::debug!("ApplicationService::new: Time tracking service created");
        
        // ユースケースを作成
        tracing::debug!("ApplicationService::new: Creating use cases");
        let project_use_cases = Box::new(
            crate::application::use_cases::ProjectUseCasesImpl::new(project_repo.clone(), project_service)
        ) as Box<dyn ProjectUseCases>;
        tracing::debug!("ApplicationService::new: Project use cases created");
        
        let task_use_cases = Box::new(
            crate::application::use_cases::TaskUseCasesImpl::new(task_repo.clone(), project_repo.clone())
        ) as Box<dyn TaskUseCases>;
        tracing::debug!("ApplicationService::new: Task use cases created");
        
        let time_tracking_use_cases = Box::new(
            crate::application::use_cases::TimeTrackingUseCasesImpl::new(
                time_entry_repo,
                task_repo,
                time_tracking_service,
            )
        ) as Box<dyn TimeTrackingUseCases>;
        tracing::debug!("ApplicationService::new: Time tracking use cases created");

        tracing::info!("ApplicationService::new: All components created successfully, creating ApplicationService instance");
        
        let service = Self {
            db: db_arc,
            project_use_cases,
            task_use_cases,
            time_tracking_use_cases,
        };
        
        tracing::info!("ApplicationService::new: Application service initialization completed successfully");
        Ok(service)
    }

    /// プロジェクトユースケースを取得
    pub fn project_use_cases(&self) -> &dyn ProjectUseCases {
        self.project_use_cases.as_ref()
    }

    /// タスクユースケースを取得
    pub fn task_use_cases(&self) -> &dyn TaskUseCases {
        self.task_use_cases.as_ref()
    }

    /// タイムトラッキングユースケースを取得
    pub fn time_tracking_use_cases(&self) -> &dyn TimeTrackingUseCases {
        self.time_tracking_use_cases.as_ref()
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
        let config = Config::default();
        let app_service = ApplicationService::new(config).await.unwrap();
        
        // ユースケースが取得できることを確認
        let _project_use_cases = app_service.project_use_cases();
        let _task_use_cases = app_service.task_use_cases();
        let _time_tracking_use_cases = app_service.time_tracking_use_cases();
    }
}

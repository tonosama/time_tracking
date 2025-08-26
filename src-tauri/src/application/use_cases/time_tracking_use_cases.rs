use crate::domain::entities::time_entry::{TimeEntry, TimeEntryEvent};
use crate::domain::repositories::{TaskRepository, TimeEntryRepository};
use crate::domain::services::TimeTrackingService;
use crate::domain::value_objects::TaskId;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// タイマー開始コマンド
#[derive(Debug, Clone)]
pub struct StartTimerCommand {
    pub task_id: TaskId,
}

/// タイマー停止コマンド
#[derive(Debug, Clone)]
pub struct StopTimerCommand {
    pub task_id: TaskId,
}

/// 手動時間エントリ追加コマンド
#[derive(Debug, Clone)]
pub struct AddManualEntryCommand {
    pub task_id: TaskId,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub note: Option<String>,
}

/// タイムトラッキングユースケーストレイト
#[async_trait]
pub trait TimeTrackingUseCases: Send + Sync {
    /// タイマーを開始する
    async fn start_timer(&self, command: StartTimerCommand) -> anyhow::Result<TimeEntryEvent>;

    /// タイマーを停止する
    async fn stop_timer(&self, command: StopTimerCommand) -> anyhow::Result<Option<TimeEntryEvent>>;

    /// 現在実行中のタスクを取得
    async fn get_current_timer(&self) -> anyhow::Result<Option<TaskId>>;

    /// 指定タスクのタイマー状態を取得
    async fn get_timer_status(&self, task_id: TaskId) -> anyhow::Result<TimerStatus>;

    /// 手動で時間エントリを追加
    async fn add_manual_entry(&self, command: AddManualEntryCommand) -> anyhow::Result<()>;

    /// 指定タスクの時間エントリ一覧を取得
    async fn get_task_entries(&self, task_id: TaskId) -> anyhow::Result<Vec<TimeEntry>>;

    /// 指定タスクの最近のエントリを取得
    async fn get_recent_task_entries(&self, task_id: TaskId, limit: usize) -> anyhow::Result<Vec<TimeEntry>>;

    /// 指定プロジェクトの時間エントリ一覧を取得
    async fn get_project_entries(&self, project_id: i64) -> anyhow::Result<Vec<TimeEntry>>;

    /// 最近の時間エントリを取得
    async fn get_recent_entries(&self, limit: usize) -> anyhow::Result<Vec<TimeEntry>>;

    /// 指定タスクの合計作業時間を取得
    async fn get_task_total_duration(&self, task_id: TaskId) -> anyhow::Result<i64>;

    /// 指定プロジェクトの合計作業時間を取得
    async fn get_project_total_duration(&self, project_id: i64) -> anyhow::Result<i64>;

    /// 全ての実行中タイマーを停止
    async fn stop_all_timers(&self) -> anyhow::Result<Vec<TimeEntryEvent>>;
}

/// タイマーの状態情報
#[derive(Debug, Clone)]
pub struct TimerStatus {
    pub is_running: bool,
    pub current_entry: Option<TimeEntry>,
    pub elapsed_seconds: Option<i64>,
}

/// タイムトラッキングユースケース実装
pub struct TimeTrackingUseCasesImpl<T: TimeEntryRepository, K: TaskRepository, S: TimeTrackingService> {
    time_entry_repository: T,
    task_repository: K,
    time_tracking_service: S,
}

impl<T: TimeEntryRepository, K: TaskRepository, S: TimeTrackingService> 
    TimeTrackingUseCasesImpl<T, K, S> 
{
    pub fn new(time_entry_repository: T, task_repository: K, time_tracking_service: S) -> Self {
        Self {
            time_entry_repository,
            task_repository,
            time_tracking_service,
        }
    }
}

#[async_trait]
impl<T: TimeEntryRepository, K: TaskRepository, S: TimeTrackingService> 
    TimeTrackingUseCases for TimeTrackingUseCasesImpl<T, K, S> 
{
    async fn start_timer(&self, command: StartTimerCommand) -> anyhow::Result<TimeEntryEvent> {
        // タスクの存在確認
        let task = self.task_repository.find_by_id(command.task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        // アーカイブ済みタスクではタイマーを開始できない
        if task.is_archived() {
            return Err(anyhow::anyhow!("Cannot start timer for archived task"));
        }

        // タイマーを開始
        let event = self.time_tracking_service.start_timer(command.task_id).await?;
        
        Ok(event)
    }

    async fn stop_timer(&self, command: StopTimerCommand) -> anyhow::Result<Option<TimeEntryEvent>> {
        // タスクの存在確認
        let _task = self.task_repository.find_by_id(command.task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        // タイマーを停止
        let event = self.time_tracking_service.stop_timer(command.task_id).await?;
        
        Ok(event)
    }

    async fn get_current_timer(&self) -> anyhow::Result<Option<TaskId>> {
        self.time_tracking_service.get_running_task().await
    }

    async fn get_timer_status(&self, task_id: TaskId) -> anyhow::Result<TimerStatus> {
        let is_running = self.time_tracking_service.is_task_running(task_id).await?;
        
        if is_running {
            let current_entry = self.time_entry_repository
                .find_running_entry_by_task(task_id)
                .await?;
            
            let elapsed_seconds = current_entry.as_ref().map(|e| e.elapsed_seconds());
            
            Ok(TimerStatus {
                is_running: true,
                current_entry,
                elapsed_seconds,
            })
        } else {
            Ok(TimerStatus {
                is_running: false,
                current_entry: None,
                elapsed_seconds: None,
            })
        }
    }

    async fn add_manual_entry(&self, command: AddManualEntryCommand) -> anyhow::Result<()> {
        // タスクの存在確認
        let task = self.task_repository.find_by_id(command.task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        // アーカイブ済みタスクには手動エントリを追加できない
        if task.is_archived() {
            return Err(anyhow::anyhow!("Cannot add entry for archived task"));
        }

        // 手動エントリを追加
        self.time_tracking_service.add_manual_entry(
            command.task_id,
            command.start_time,
            command.end_time,
            command.note,
        ).await?;

        Ok(())
    }

    async fn get_task_entries(&self, task_id: TaskId) -> anyhow::Result<Vec<TimeEntry>> {
        // タスクの存在確認
        let _task = self.task_repository.find_by_id(task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        self.time_entry_repository.find_entries_by_task(task_id).await
    }

    async fn get_recent_task_entries(&self, task_id: TaskId, limit: usize) -> anyhow::Result<Vec<TimeEntry>> {
        // タスクの存在確認
        let _task = self.task_repository.find_by_id(task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        self.time_entry_repository
            .find_recent_entries_by_task(task_id, limit)
            .await
    }

    async fn get_project_entries(&self, project_id: i64) -> anyhow::Result<Vec<TimeEntry>> {
        self.time_entry_repository
            .find_entries_by_project(project_id)
            .await
    }

    async fn get_recent_entries(&self, limit: usize) -> anyhow::Result<Vec<TimeEntry>> {
        println!("[USECASE] get_recent_entries called with limit: {}", limit);
        eprintln!("[USECASE] get_recent_entries called with limit: {}", limit);
        
        tracing::info!("TimeTrackingUseCasesImpl::get_recent_entries: Starting with limit: {}", limit);
        
        tracing::debug!("TimeTrackingUseCasesImpl::get_recent_entries: Calling time_entry_repository.find_recent_entries");
        
        match self.time_entry_repository.find_recent_entries(limit).await {
            Ok(entries) => {
                println!("[USECASE] get_recent_entries success - {} entries returned", entries.len());
                eprintln!("[USECASE] get_recent_entries success - {} entries returned", entries.len());
                
                tracing::info!("TimeTrackingUseCasesImpl::get_recent_entries: Repository call successful - {} entries returned", entries.len());
                
                // エントリーの詳細をログ出力
                for (i, entry) in entries.iter().enumerate() {
                    tracing::debug!("TimeTrackingUseCasesImpl::get_recent_entries: Entry {} - task_id: {}, start_time: {}, end_time: {:?}, duration: {:?}", 
                        i + 1, 
                        entry.task_id(), 
                        entry.start_time(), 
                        entry.end_time(),
                        entry.duration_in_seconds()
                    );
                }
                
                tracing::debug!("TimeTrackingUseCasesImpl::get_recent_entries: Successfully returning {} entries", entries.len());
                Ok(entries)
            },
            Err(e) => {
                println!("[USECASE] get_recent_entries failed: {}", e);
                eprintln!("[USECASE] get_recent_entries failed: {}", e);
                
                tracing::error!("TimeTrackingUseCasesImpl::get_recent_entries: Repository call failed: {}", e);
                tracing::error!("TimeTrackingUseCasesImpl::get_recent_entries: Error type: {}", std::any::type_name_of_val(&*e));
                tracing::error!("TimeTrackingUseCasesImpl::get_recent_entries: Error details: {:?}", e);
                
                // エラーの種類に応じた詳細情報
                if let Some(db_error) = e.downcast_ref::<rusqlite::Error>() {
                    println!("[USECASE] get_recent_entries SQLite error: {:?}", db_error);
                    eprintln!("[USECASE] get_recent_entries SQLite error: {:?}", db_error);
                    
                    tracing::error!("TimeTrackingUseCasesImpl::get_recent_entries: SQLite error: {:?}", db_error);
                }
                
                if let Some(anyhow_error) = e.downcast_ref::<anyhow::Error>() {
                    println!("[USECASE] get_recent_entries Anyhow error: {:?}", anyhow_error);
                    eprintln!("[USECASE] get_recent_entries Anyhow error: {:?}", anyhow_error);
                    
                    tracing::error!("TimeTrackingUseCasesImpl::get_recent_entries: Anyhow error: {:?}", anyhow_error);
                }
                
                Err(e)
            }
        }
    }

    async fn get_task_total_duration(&self, task_id: TaskId) -> anyhow::Result<i64> {
        // タスクの存在確認
        let _task = self.task_repository.find_by_id(task_id).await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        self.time_entry_repository.sum_duration_by_task(task_id).await
    }

    async fn get_project_total_duration(&self, project_id: i64) -> anyhow::Result<i64> {
        self.time_entry_repository
            .sum_duration_by_project(project_id)
            .await
    }

    async fn stop_all_timers(&self) -> anyhow::Result<Vec<TimeEntryEvent>> {
        self.time_tracking_service.stop_all_timers().await
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::domain::entities::{Project, Task};
    use crate::domain::repositories::{time_entry_tests::InMemoryTimeEntryRepository, task_tests::InMemoryTaskRepository};
    use crate::domain::services::TimeTrackingServiceImpl;
    use crate::domain::value_objects::ProjectId;

    async fn setup_use_cases() -> (
        TimeTrackingUseCasesImpl<
            InMemoryTimeEntryRepository,
            InMemoryTaskRepository,
            TimeTrackingServiceImpl<InMemoryTimeEntryRepository>
        >,
        TaskId,
    ) {
        let time_entry_repo = InMemoryTimeEntryRepository::new();
        let task_repo = InMemoryTaskRepository::new();
        let time_tracking_service = TimeTrackingServiceImpl::new(time_entry_repo.clone());

        // テスト用プロジェクトとタスクを作成
        let project_id = ProjectId::new(1).unwrap();
        let _project = Project::new(project_id, "Test Project".to_string()).unwrap();
        // Note: このテストではプロジェクトリポジトリは使用しない

        let task_id = TaskId::new(1).unwrap();
        let task = Task::new(task_id, project_id, "Test Task".to_string()).unwrap();
        task_repo.save(&task).await.unwrap();

        let use_cases = TimeTrackingUseCasesImpl::new(
            time_entry_repo,
            task_repo,
            time_tracking_service,
        );

        (use_cases, task_id)
    }

    #[tokio::test]
    async fn タイマー開始が正しく動作すること() {
        let (use_cases, task_id) = setup_use_cases().await;

        let command = StartTimerCommand { task_id };
        let event = use_cases.start_timer(command).await.unwrap();

        assert_eq!(event.task_id(), task_id);
        assert!(event.is_start());

        // タイマー状態を確認
        let status = use_cases.get_timer_status(task_id).await.unwrap();
        assert!(status.is_running);
        assert!(status.current_entry.is_some());
        assert!(status.elapsed_seconds.is_some());
    }

    #[tokio::test]
    async fn タイマー停止が正しく動作すること() {
        let (use_cases, task_id) = setup_use_cases().await;

        // タイマーを開始
        let start_command = StartTimerCommand { task_id };
        use_cases.start_timer(start_command).await.unwrap();

        // タイマーを停止
        let stop_command = StopTimerCommand { task_id };
        let stop_event = use_cases.stop_timer(stop_command).await.unwrap();

        assert!(stop_event.is_some());
        let stop_event = stop_event.unwrap();
        assert_eq!(stop_event.task_id(), task_id);
        assert!(stop_event.is_stop());

        // タイマー状態を確認
        let status = use_cases.get_timer_status(task_id).await.unwrap();
        assert!(!status.is_running);
        assert!(status.current_entry.is_none());
        assert!(status.elapsed_seconds.is_none());
    }

    #[tokio::test]
    async fn 存在しないタスクでタイマー開始が失敗すること() {
        let (use_cases, _) = setup_use_cases().await;
        let invalid_task_id = TaskId::new(999).unwrap();

        let command = StartTimerCommand { task_id: invalid_task_id };
        let result = use_cases.start_timer(command).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Task not found"));
    }

    #[tokio::test]
    async fn アーカイブタスクでタイマー開始が失敗すること() {
        let (use_cases, task_id) = setup_use_cases().await;

        // タスクをアーカイブ
        let task = use_cases.task_repository.find_by_id(task_id).await.unwrap().unwrap();
        let archived_task = task.archive();
        use_cases.task_repository.save(&archived_task).await.unwrap();

        let command = StartTimerCommand { task_id };
        let result = use_cases.start_timer(command).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot start timer for archived task"));
    }

    #[tokio::test]
    async fn 手動エントリ追加が正しく動作すること() {
        let (use_cases, task_id) = setup_use_cases().await;

        let start_time = chrono::Utc::now() - chrono::Duration::hours(2);
        let end_time = chrono::Utc::now() - chrono::Duration::hours(1);
        let note = Some("手動追加のテスト".to_string());

        let command = AddManualEntryCommand {
            task_id,
            start_time,
            end_time,
            note,
        };

        let result = use_cases.add_manual_entry(command).await;
        assert!(result.is_ok());

        // エントリが作成されたことを確認
        let entries = use_cases.get_task_entries(task_id).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].is_completed());
        assert_eq!(entries[0].duration_in_seconds(), Some(3600)); // 1時間
    }

    #[tokio::test]
    async fn タスクの時間エントリ一覧取得が正しく動作すること() {
        let (use_cases, task_id) = setup_use_cases().await;

        // タイマーで1つのエントリを作成
        let start_command = StartTimerCommand { task_id };
        use_cases.start_timer(start_command).await.unwrap();

        let stop_command = StopTimerCommand { task_id };
        use_cases.stop_timer(stop_command).await.unwrap();

        // エントリを取得
        let entries = use_cases.get_task_entries(task_id).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].task_id(), task_id);
        assert!(entries[0].is_completed());
    }

    #[tokio::test]
    async fn 合計作業時間取得が正しく動作すること() {
        let (use_cases, task_id) = setup_use_cases().await;

        // 手動で1時間のエントリを追加
        let start_time = chrono::Utc::now() - chrono::Duration::hours(2);
        let end_time = chrono::Utc::now() - chrono::Duration::hours(1);

        let command = AddManualEntryCommand {
            task_id,
            start_time,
            end_time,
            note: None,
        };
        use_cases.add_manual_entry(command).await.unwrap();

        // 合計時間を取得
        let total_duration = use_cases.get_task_total_duration(task_id).await.unwrap();
        assert_eq!(total_duration, 3600); // 1時間 = 3600秒
    }

    #[tokio::test]
    async fn 現在のタイマー取得が正しく動作すること() {
        let (use_cases, task_id) = setup_use_cases().await;

        // 初期状態では実行中のタイマーなし
        let current = use_cases.get_current_timer().await.unwrap();
        assert!(current.is_none());

        // タイマーを開始
        let start_command = StartTimerCommand { task_id };
        use_cases.start_timer(start_command).await.unwrap();

        // 現在のタイマーを確認
        let current = use_cases.get_current_timer().await.unwrap();
        assert_eq!(current, Some(task_id));

        // タイマーを停止
        let stop_command = StopTimerCommand { task_id };
        use_cases.stop_timer(stop_command).await.unwrap();

        // 再び実行中のタイマーなし
        let current = use_cases.get_current_timer().await.unwrap();
        assert!(current.is_none());
    }

    #[tokio::test]
    async fn 全タイマー停止が正しく動作すること() {
        let (use_cases, task_id) = setup_use_cases().await;

        // タイマーを開始
        let start_command = StartTimerCommand { task_id };
        use_cases.start_timer(start_command).await.unwrap();

        // 全タイマーを停止
        let stop_events = use_cases.stop_all_timers().await.unwrap();
        assert_eq!(stop_events.len(), 1);
        assert_eq!(stop_events[0].task_id(), task_id);

        // 実行中のタイマーがないことを確認
        let current = use_cases.get_current_timer().await.unwrap();
        assert!(current.is_none());
    }
}

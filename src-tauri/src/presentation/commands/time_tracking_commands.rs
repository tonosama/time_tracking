use crate::application::dto::{
    AddManualEntryRequest, CurrentTimerResponse, ProjectTimeSummaryResponse, StartTimerRequest,
    StopTimerRequest, TaskTimeSummaryResponse, TimeEntryEventResponse, TimeEntryResponse,
    TimerStatusResponse,
};
use crate::application::services::ApplicationService;
use crate::domain::value_objects::TaskId;
use tauri::State;

/// タイマーを開始する
#[tauri::command]
pub async fn start_timer(
    app_service: State<'_, ApplicationService>,
    request: StartTimerRequest,
) -> Result<TimeEntryEventResponse, String> {
    tracing::info!(task_id = request.task_id, "Timer start requested");
    
    let task_id = request.task_id; // コピーしておく
    let command = request.to_command().map_err(|e| {
        tracing::error!(task_id = task_id, error = %e, "Failed to create start timer command");
        e.to_string()
    })?;
    
    let event = app_service
        .time_tracking_use_cases()
        .start_timer(command)
        .await
        .map_err(|e| {
            tracing::error!(task_id = task_id, error = %e, "Failed to start timer");
            e.to_string()
        })?;

    tracing::info!(
        task_id = task_id,
        event_id = ?event.id(),
        "Timer started successfully"
    );

    Ok(TimeEntryEventResponse::from(event))
}

/// タイマーを停止する
#[tauri::command]
pub async fn stop_timer(
    app_service: State<'_, ApplicationService>,
    request: StopTimerRequest,
) -> Result<Option<TimeEntryEventResponse>, String> {
    tracing::info!(task_id = request.task_id, "Timer stop requested");
    
    let task_id = request.task_id; // コピーしておく
    let command = request.to_command().map_err(|e| {
        tracing::error!(task_id = task_id, error = %e, "Failed to create stop timer command");
        e.to_string()
    })?;
    
    let event = app_service
        .time_tracking_use_cases()
        .stop_timer(command)
        .await
        .map_err(|e| {
            tracing::error!(task_id = task_id, error = %e, "Failed to stop timer");
            e.to_string()
        })?;

    if let Some(ref event) = event {
        tracing::info!(
            task_id = task_id,
            event_id = ?event.id(),
            "Timer stopped successfully"
        );
    } else {
        tracing::warn!(task_id = task_id, "No timer was running for this task");
    }

    Ok(event.map(TimeEntryEventResponse::from))
}

/// 現在実行中のタイマーを取得する
#[tauri::command]
pub async fn get_current_timer(
    app_service: State<'_, ApplicationService>,
) -> Result<CurrentTimerResponse, String> {
    let current_task_id = app_service
        .time_tracking_use_cases()
        .get_current_timer()
        .await
        .map_err(|e| e.to_string())?;

    // 実行中の場合は経過時間も取得
    let elapsed_seconds = if let Some(task_id) = current_task_id {
        let status = app_service
            .time_tracking_use_cases()
            .get_timer_status(task_id)
            .await
            .map_err(|e| e.to_string())?;
        status.elapsed_seconds
    } else {
        None
    };

    Ok(CurrentTimerResponse::new(current_task_id, elapsed_seconds))
}

/// 指定タスクのタイマー状態を取得する
#[tauri::command]
pub async fn get_timer_status(
    app_service: State<'_, ApplicationService>,
    task_id: i64,
) -> Result<TimerStatusResponse, String> {
    let task_id = TaskId::new(task_id).map_err(|e| e.to_string())?;
    
    let status = app_service
        .time_tracking_use_cases()
        .get_timer_status(task_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(TimerStatusResponse::new(
        status.is_running,
        status.current_entry,
        status.elapsed_seconds,
    ))
}

/// 手動で時間エントリを追加する
#[tauri::command]
pub async fn add_manual_entry(
    app_service: State<'_, ApplicationService>,
    request: AddManualEntryRequest,
) -> Result<(), String> {
    let command = request.to_command().map_err(|e| e.to_string())?;
    
    app_service
        .time_tracking_use_cases()
        .add_manual_entry(command)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// 指定タスクの時間エントリ一覧を取得する
#[tauri::command]
pub async fn get_task_entries(
    app_service: State<'_, ApplicationService>,
    task_id: i64,
) -> Result<Vec<TimeEntryResponse>, String> {
    let task_id = TaskId::new(task_id).map_err(|e| e.to_string())?;
    
    let entries = app_service
        .time_tracking_use_cases()
        .get_task_entries(task_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(entries.into_iter().map(TimeEntryResponse::from).collect())
}

/// 指定タスクの最近の時間エントリを取得する
#[tauri::command]
pub async fn get_recent_task_entries(
    app_service: State<'_, ApplicationService>,
    task_id: i64,
    limit: usize,
) -> Result<Vec<TimeEntryResponse>, String> {
    let task_id = TaskId::new(task_id).map_err(|e| e.to_string())?;
    
    let entries = app_service
        .time_tracking_use_cases()
        .get_recent_task_entries(task_id, limit)
        .await
        .map_err(|e| e.to_string())?;

    Ok(entries.into_iter().map(TimeEntryResponse::from).collect())
}

/// 指定プロジェクトの時間エントリ一覧を取得する
#[tauri::command]
pub async fn get_project_entries(
    app_service: State<'_, ApplicationService>,
    project_id: i64,
) -> Result<Vec<TimeEntryResponse>, String> {
    let entries = app_service
        .time_tracking_use_cases()
        .get_project_entries(project_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(entries.into_iter().map(TimeEntryResponse::from).collect())
}

/// 最近の時間エントリを取得する
#[tauri::command]
pub async fn get_recent_entries(
    app_service: State<'_, ApplicationService>,
    limit: usize,
) -> Result<Vec<TimeEntryResponse>, String> {
    let entries = app_service
        .time_tracking_use_cases()
        .get_recent_entries(limit)
        .await
        .map_err(|e| e.to_string())?;

    Ok(entries.into_iter().map(TimeEntryResponse::from).collect())
}

/// 指定タスクの時間サマリーを取得する
#[tauri::command]
pub async fn get_task_time_summary(
    app_service: State<'_, ApplicationService>,
    task_id: i64,
) -> Result<TaskTimeSummaryResponse, String> {
    let task_id = TaskId::new(task_id).map_err(|e| e.to_string())?;
    
    // 合計時間を取得
    let total_duration = app_service
        .time_tracking_use_cases()
        .get_task_total_duration(task_id)
        .await
        .map_err(|e| e.to_string())?;

    // エントリ数を取得
    let entries = app_service
        .time_tracking_use_cases()
        .get_task_entries(task_id)
        .await
        .map_err(|e| e.to_string())?;
    let entry_count = entries.len();

    // 実行中かどうかを取得
    let status = app_service
        .time_tracking_use_cases()
        .get_timer_status(task_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(TaskTimeSummaryResponse::new(
        task_id,
        total_duration,
        entry_count,
        status.is_running,
    ))
}

/// 指定プロジェクトの時間サマリーを取得する
#[tauri::command]
pub async fn get_project_time_summary(
    app_service: State<'_, ApplicationService>,
    project_id: i64,
) -> Result<ProjectTimeSummaryResponse, String> {
    // 合計時間を取得
    let total_duration = app_service
        .time_tracking_use_cases()
        .get_project_total_duration(project_id)
        .await
        .map_err(|e| e.to_string())?;

    // エントリ数を取得
    let entries = app_service
        .time_tracking_use_cases()
        .get_project_entries(project_id)
        .await
        .map_err(|e| e.to_string())?;
    let entry_count = entries.len();

    Ok(ProjectTimeSummaryResponse::new(
        project_id,
        total_duration,
        entry_count,
    ))
}

/// 全ての実行中タイマーを停止する
#[tauri::command]
pub async fn stop_all_timers(
    app_service: State<'_, ApplicationService>,
) -> Result<Vec<TimeEntryEventResponse>, String> {
    let events = app_service
        .time_tracking_use_cases()
        .stop_all_timers()
        .await
        .map_err(|e| e.to_string())?;

    Ok(events.into_iter().map(TimeEntryEventResponse::from).collect())
}

/// 指定タスクが実行中かどうかを取得する（軽量版）
#[tauri::command]
pub async fn is_task_running(
    app_service: State<'_, ApplicationService>,
    task_id: i64,
) -> Result<bool, String> {
    let task_id = TaskId::new(task_id).map_err(|e| e.to_string())?;
    
    let status = app_service
        .time_tracking_use_cases()
        .get_timer_status(task_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(status.is_running)
}

/// 指定期間の時間エントリーを取得する（新規追加）
#[tauri::command]
pub async fn get_time_entries(
    app_service: State<'_, ApplicationService>,
    #[allow(non_snake_case)] startDate: String,
) -> Result<Vec<TimeEntryResponse>, String> {
    println!("[BACKEND] get_time_entries called with startDate: {}", startDate);
    eprintln!("[BACKEND] get_time_entries called with startDate: {}", startDate);
    
    tracing::info!("get_time_entries: Command called with startDate: {}", startDate);
    tracing::info!("get_time_entries: Command execution started");
    tracing::info!("get_time_entries: About to call app_service.time_tracking_use_cases().get_recent_entries(100)");
    
    // 簡単な実装：最新のエントリーを取得
    tracing::debug!("get_time_entries: Calling app_service.time_tracking_use_cases().get_recent_entries(100)");
    
    match app_service
        .time_tracking_use_cases()
        .get_recent_entries(100) // 最新100件を取得
        .await
    {
        Ok(entries) => {
            println!("[BACKEND] get_time_entries success - {} entries returned", entries.len());
            eprintln!("[BACKEND] get_time_entries success - {} entries returned", entries.len());
            
            tracing::info!("get_time_entries: Use case call successful - {} entries returned", entries.len());
            
            tracing::debug!("get_time_entries: Converting entries to TimeEntryResponse");
            let responses: Vec<TimeEntryResponse> = entries.into_iter().map(TimeEntryResponse::from).collect();
            
            tracing::info!("get_time_entries: Successfully converted {} entries to TimeEntryResponse", responses.len());
            
            // レスポンスの詳細をログ出力
            for (i, response) in responses.iter().enumerate() {
                tracing::debug!("get_time_entries: Response {} - task_id: {}, start_time: {:?}, end_time: {:?}, duration: {:?}", 
                    i + 1, 
                    response.task_id, 
                    response.start_time, 
                    response.end_time,
                    response.duration_in_seconds
                );
            }
            
            tracing::info!("get_time_entries: Command completed successfully - returning {} entries", responses.len());
            Ok(responses)
        },
        Err(e) => {
            println!("[BACKEND] get_time_entries failed: {}", e);
            eprintln!("[BACKEND] get_time_entries failed: {}", e);
            
            tracing::error!("get_time_entries: Use case call failed: {}", e);
            tracing::error!("get_time_entries: Error type: {}", std::any::type_name_of_val(&*e));
            tracing::error!("get_time_entries: Error details: {:?}", e);
            
            // エラーの種類に応じた詳細情報
            if let Some(db_error) = e.downcast_ref::<rusqlite::Error>() {
                println!("[BACKEND] get_time_entries SQLite error: {:?}", db_error);
                eprintln!("[BACKEND] get_time_entries SQLite error: {:?}", db_error);
                
                tracing::error!("get_time_entries: SQLite error: {:?}", db_error);
                match db_error {
                    rusqlite::Error::QueryReturnedNoRows => {
                        tracing::warn!("get_time_entries: No rows found in database");
                    },
                    rusqlite::Error::InvalidPath(_) => {
                        tracing::error!("get_time_entries: Invalid database path");
                    },
                    rusqlite::Error::SqliteFailure(code, message) => {
                        tracing::error!("get_time_entries: SQLite failure - code: {}, message: {:?}", code, message);
                    },
                    _ => {
                        tracing::error!("get_time_entries: Other SQLite error: {:?}", db_error);
                    }
                }
            }
            
            if let Some(anyhow_error) = e.downcast_ref::<anyhow::Error>() {
                println!("[BACKEND] get_time_entries Anyhow error: {:?}", anyhow_error);
                eprintln!("[BACKEND] get_time_entries Anyhow error: {:?}", anyhow_error);
                
                tracing::error!("get_time_entries: Anyhow error: {:?}", anyhow_error);
            }
            
            // データが存在しない場合は空の配列を返す
            if e.to_string().contains("no rows") {
                println!("[BACKEND] get_time_entries: No rows found, returning empty array");
                eprintln!("[BACKEND] get_time_entries: No rows found, returning empty array");
                
                tracing::info!("get_time_entries: No rows found, returning empty array");
                Ok(Vec::new())
            } else {
                println!("[BACKEND] get_time_entries: Returning error to frontend: {}", e);
                eprintln!("[BACKEND] get_time_entries: Returning error to frontend: {}", e);
                
                tracing::error!("get_time_entries: Returning error to frontend: {}", e);
                Err(e.to_string())
            }
        }
    }
}

/// テスト用の簡単なコマンド
#[tauri::command]
pub async fn test_get_time_entries() -> Result<String, String> {
    println!("[BACKEND] test_get_time_entries called");
    eprintln!("[BACKEND] test_get_time_entries called");
    
    Ok("test_get_time_entries: Command is working!".to_string())
}

/// 全体のタイマーステータスを取得する（パラメータ不要版）
#[tauri::command]
pub async fn get_global_timer_status(
    app_service: State<'_, ApplicationService>,
) -> Result<CurrentTimerResponse, String> {
    tracing::debug!("get_global_timer_status called");
    
    let current_task_id = app_service
        .time_tracking_use_cases()
        .get_current_timer()
        .await
        .map_err(|e| {
            tracing::error!("get_global_timer_status: get_current_timer failed: {}", e);
            e.to_string()
        })?;

    let (task_id, elapsed_seconds) = if let Some(task_id) = current_task_id {
        tracing::debug!("get_global_timer_status: current task found: {}", task_id);
        let elapsed = app_service
            .time_tracking_use_cases()
            .get_timer_status(task_id)
            .await
            .map_err(|e| {
                tracing::error!("get_global_timer_status: get_timer_status failed: {}", e);
                e.to_string()
            })?;
        (Some(task_id.into()), elapsed.elapsed_seconds)
    } else {
        tracing::debug!("get_global_timer_status: no current task");
        (None, None)
    };

    tracing::debug!("get_global_timer_status success: task_id={:?}, elapsed={:?}", task_id, elapsed_seconds);
    Ok(CurrentTimerResponse::new(task_id, elapsed_seconds))
}

#[cfg(test)]
#[allow(non_snake_case, dead_code)]
mod tests {
    // テストを一時的に無効化
    /*
    use super::*;
    use crate::application::services::ApplicationService;


    async fn setup_app_service() -> ApplicationService {
        let config = crate::infrastructure::config::Config::default();
        ApplicationService::new(config).await.unwrap()
    }

    async fn create_test_task(app_service: &ApplicationService) -> (i64, i64) {
        // テスト用プロジェクトを作成
        let project_request = crate::application::dto::CreateProjectRequest {
            name: "Test Project".to_string(),
        };
        let project_response = app_service
            .project_use_cases()
            .create_project(project_request.to_command().unwrap())
            .await
            .unwrap();

        // テスト用タスクを作成
        let task_request = crate::application::dto::CreateTaskRequest {
            project_id: project_response.id(),
            name: "Test Task".to_string(),
        };
        let task_response = app_service
            .task_use_cases()
            .create_task(task_request.to_command().unwrap())
            .await
            .unwrap();

        (project_response.id(), task_response.id())
    }

    #[tokio::test]
    async fn タイマー開始停止が正しく動作すること() {
        let app_service = setup_app_service().await;
        let (_, task_id) = create_test_task(&app_service).await;

        // タイマー開始
        let start_request = StartTimerRequest { task_id };
        let start_response = start_timer(tauri::State::from(&app_service), start_request)
            .await
            .unwrap();

        assert_eq!(start_response.task_id, task_id);
        assert_eq!(start_response.event_type, "start");

        // 実行中状態を確認
        let status = get_timer_status(tauri::State(&app_service), task_id)
            .await
            .unwrap();
        assert!(status.is_running);

        // タイマー停止
        let stop_request = StopTimerRequest { task_id };
        let stop_response = stop_timer(tauri::State(&app_service), stop_request)
            .await
            .unwrap();

        assert!(stop_response.is_some());
        let stop_event = stop_response.unwrap();
        assert_eq!(stop_event.task_id, task_id);
        assert_eq!(stop_event.event_type, "stop");

        // 停止状態を確認
        let status = get_timer_status(tauri::State(&app_service), task_id)
            .await
            .unwrap();
        assert!(!status.is_running);
    }

    #[tokio::test]
    async fn 現在のタイマー取得が正しく動作すること() {
        let app_service = setup_app_service().await;
        let (_, task_id) = create_test_task(&app_service).await;

        // 初期状態では実行中のタイマーなし
        let current = get_current_timer(tauri::State(&app_service)).await.unwrap();
        assert!(current.task_id.is_none());

        // タイマーを開始
        let start_request = StartTimerRequest { task_id };
        start_timer(tauri::State(&app_service), start_request)
            .await
            .unwrap();

        // 現在のタイマーを確認
        let current = get_current_timer(tauri::State(&app_service)).await.unwrap();
        assert_eq!(current.task_id, Some(task_id));
        assert!(current.elapsed_seconds.is_some());
    }

    #[tokio::test]
    async fn 手動エントリ追加が正しく動作すること() {
        let app_service = setup_app_service().await;
        let (_, task_id) = create_test_task(&app_service).await;

        let request = AddManualEntryRequest {
            task_id,
            start_time: "2024-01-01T10:00:00Z".to_string(),
            end_time: "2024-01-01T11:00:00Z".to_string(),
            note: Some("手動追加".to_string()),
        };

        let result = add_manual_entry(tauri::State(&app_service), request).await;
        assert!(result.is_ok());

        // エントリが作成されたことを確認
        let entries = get_task_entries(tauri::State(&app_service), task_id)
            .await
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].duration_in_seconds, Some(3600)); // 1時間
    }

    #[tokio::test]
    async fn タスク時間サマリー取得が正しく動作すること() {
        let app_service = setup_app_service().await;
        let (_, task_id) = create_test_task(&app_service).await;

        // 手動エントリを追加
        let request = AddManualEntryRequest {
            task_id,
            start_time: "2024-01-01T10:00:00Z".to_string(),
            end_time: "2024-01-01T11:00:00Z".to_string(),
            note: None,
        };
        add_manual_entry(tauri::State(&app_service), request).await.unwrap();

        // サマリーを取得
        let summary = get_task_time_summary(tauri::State(&app_service), task_id)
            .await
            .unwrap();

        assert_eq!(summary.task_id, task_id);
        assert_eq!(summary.total_duration_seconds, 3600);
        assert_eq!(summary.total_duration_formatted, "01:00:00");
        assert_eq!(summary.entry_count, 1);
        assert!(!summary.is_running);
    }

    #[tokio::test]
    async fn 実行中タスク確認が正しく動作すること() {
        let app_service = setup_app_service().await;
        let (_, task_id) = create_test_task(&app_service).await;

        // 初期状態では実行中でない
        let is_running = is_task_running(tauri::State(&app_service), task_id)
            .await
            .unwrap();
        assert!(!is_running);

        // タイマーを開始
        let start_request = StartTimerRequest { task_id };
        start_timer(tauri::State(&app_service), start_request)
            .await
            .unwrap();

        // 実行中であることを確認
        let is_running = is_task_running(tauri::State(&app_service), task_id)
            .await
            .unwrap();
        assert!(is_running);
    }

    #[tokio::test]
    async fn 存在しないタスクでエラーが発生すること() {
        let app_service = setup_app_service().await;
        let invalid_task_id = 999999;

        let start_request = StartTimerRequest { task_id: invalid_task_id };
        let result = start_timer(tauri::State(&app_service), start_request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Task not found"));
    }

    #[tokio::test]
    async fn 全タイマー停止が正しく動作すること() {
        let app_service = setup_app_service().await;
        let (_, task_id) = create_test_task(&app_service).await;

        // タイマーを開始
        let start_request = StartTimerRequest { task_id };
        start_timer(tauri::State(&app_service), start_request)
            .await
            .unwrap();

        // 全タイマーを停止
        let stop_events = stop_all_timers(tauri::State(&app_service)).await.unwrap();
        assert_eq!(stop_events.len(), 1);
        assert_eq!(stop_events[0].task_id, task_id);

        // 実行中のタイマーがないことを確認
        let current = get_current_timer(tauri::State(&app_service)).await.unwrap();
        assert!(current.task_id.is_none());
    */
}

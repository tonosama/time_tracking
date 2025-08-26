// Time Tracker Go - Main Entry Point

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use time_tracker_go::application::services::ApplicationService;
use time_tracker_go::infrastructure::config::Config;
use time_tracker_go::presentation::commands::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::rolling;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    // ログ出力を初期化
    tracing::info!("main: Starting Time Tracker application initialization");
    init_logging();
    tracing::info!("main: Logging system initialized");

    tracing::info!("main: Loading application configuration");
    // 設定を読み込み
    let config = Config::default();
    tracing::info!("main: Configuration loaded: {:?}", config);

    // アプリケーションサービスを初期化
    tracing::info!("main: Initializing application service");
    let app_service = match ApplicationService::new(config).await {
        Ok(service) => {
            tracing::info!("main: Application service initialized successfully");
            service
        },
        Err(e) => {
            tracing::error!("main: Failed to initialize application service: {}", e);
            tracing::error!("main: Application startup failed");
            std::process::exit(1);
        }
    };

    tracing::info!("main: Setting up Tauri application");
    tauri::Builder::default()
        .manage(app_service)
        .invoke_handler(tauri::generate_handler![
            // プロジェクト管理コマンド
            create_project,
            update_project,
            archive_project,
            restore_project,
            get_project,
            get_all_active_projects,
            get_all_projects,
            get_project_history,
            // タスク管理コマンド
            create_task,
            update_task,
            archive_task,
            restore_task,
            get_task,
            get_tasks_by_project,
            get_active_tasks_by_project,
            get_all_active_tasks,
            get_task_history,
            move_task_to_project,
            // タイムトラッキング管理コマンド
            start_timer,
            stop_timer,
            get_current_timer,
            get_timer_status,
            add_manual_entry,
            get_task_entries,
            get_recent_task_entries,
            get_project_entries,
            get_recent_entries,
            get_time_entries,
            test_get_time_entries,
            get_global_timer_status,
            get_task_time_summary,
            get_project_time_summary,
            stop_all_timers,
            is_task_running,
            // ログ出力コマンド
            log_to_file,
            // デバッグ用コマンド
            debug_database_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// ログ出力を初期化
fn init_logging() {
    // ログディレクトリを取得
    let log_dir = get_log_directory();
    
    // ログディレクトリが存在しない場合は作成
    if !log_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&log_dir) {
            eprintln!("Failed to create log directory: {}", e);
            return;
        }
    }

    // ファイルローテーション設定（日次）
    let file_appender = rolling::daily(&log_dir, "time-tracker.log");

    // 環境変数からログレベルを取得（デフォルト: debug）
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "time_tracker_go=debug,debug".into());

    // ログサブスクライバーを設定
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file_appender)
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(false) // ファイル出力ではANSIエスケープコードを無効化
        )
        .init();

    tracing::info!("Logging initialized. Log files will be written to: {:?}", log_dir);
}

/// プラットフォーム固有のログディレクトリを取得
fn get_log_directory() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Library")
            .join("Logs")
            .join("time-tracker-go")
    }
    
    #[cfg(target_os = "windows")]
    {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("time-tracker-go")
            .join("logs")
    }
    
    #[cfg(target_os = "linux")]
    {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("time-tracker-go")
            .join("logs")
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        PathBuf::from("./logs")
    }
}

// Time Tracker Go - Main Entry Point

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use time_tracker_go::application::services::ApplicationService;
use time_tracker_go::infrastructure::database::DatabaseConnection;
use time_tracker_go::presentation::commands::*;

fn main() {
    // データベース接続を初期化
    let db = DatabaseConnection::new_in_memory()
        .expect("Failed to create database connection");
    
    db.run_migrations()
        .expect("Failed to run database migrations");

    // アプリケーションサービスを初期化
    let app_service = ApplicationService::new(db);
    let project_use_cases = Box::new(app_service.create_project_use_cases()) as Box<dyn time_tracker_go::application::use_cases::ProjectUseCases>;
    let task_use_cases = Box::new(app_service.create_task_use_cases()) as Box<dyn time_tracker_go::application::use_cases::TaskUseCases>;

    tauri::Builder::default()
        .manage(project_use_cases)
        .manage(task_use_cases)
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use time_tracker_go::domain::entities::Project;
use time_tracker_go::domain::value_objects::{ProjectId, Status};
use time_tracker_go::infrastructure::database::DatabaseConnection;
use time_tracker_go::infrastructure::repositories::SqliteProjectRepository;
use chrono::{TimeZone, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_project_repository_save_and_find() {
    // データベース接続をセットアップ
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteProjectRepository::new(db.clone());
    
    // テスト用プロジェクトを作成
    let project_id = ProjectId::new(1).unwrap();
    let project = Project::new(project_id, "テストプロジェクト".to_string()).unwrap();
    
    // プロジェクトを保存
    repository.save(&project).await.unwrap();
    
    // プロジェクトを取得して検証
    let found_project = repository.find_by_id(project_id).await.unwrap().unwrap();
    assert_eq!(found_project.id(), project_id);
    assert_eq!(found_project.name(), "テストプロジェクト");
    assert!(found_project.status().is_active());
}

#[tokio::test]
async fn test_project_repository_find_by_id_not_found() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteProjectRepository::new(db.clone());
    
    // 存在しないプロジェクトIDで検索
    let project_id = ProjectId::new(999).unwrap();
    let result = repository.find_by_id(project_id).await.unwrap();
    
    assert!(result.is_none());
}

#[tokio::test]
async fn test_project_repository_update() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteProjectRepository::new(db.clone());
    
    // 初期プロジェクトを作成
    let project_id = ProjectId::new(1).unwrap();
    let mut project = Project::new(project_id, "初期プロジェクト".to_string()).unwrap();
    
    // 保存
    repository.save(&project).await.unwrap();
    
    // プロジェクト名を変更
    project = project.change_name("更新されたプロジェクト".to_string()).unwrap();
    repository.save(&project).await.unwrap();
    
    // 更新されたプロジェクトを取得して検証
    let found_project = repository.find_by_id(project_id).await.unwrap().unwrap();
    assert_eq!(found_project.name(), "更新されたプロジェクト");
}

#[tokio::test]
async fn test_project_repository_archive() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteProjectRepository::new(db.clone());
    
    // プロジェクトを作成
    let project_id = ProjectId::new(1).unwrap();
    let mut project = Project::new(project_id, "アーカイブテスト".to_string()).unwrap();
    
    // 保存
    repository.save(&project).await.unwrap();
    
    // プロジェクトをアーカイブ
    project = project.archive();
    repository.save(&project).await.unwrap();
    
    // アーカイブされたプロジェクトを取得して検証
    let found_project = repository.find_by_id(project_id).await.unwrap().unwrap();
    assert!(found_project.status().is_archived());
}

#[tokio::test]
async fn test_project_repository_find_all_active() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteProjectRepository::new(db.clone());
    
    // 複数のプロジェクトを作成
    let project1 = Project::new(ProjectId::new(1).unwrap(), "プロジェクト1".to_string()).unwrap();
    let project2 = Project::new(ProjectId::new(2).unwrap(), "プロジェクト2".to_string()).unwrap();
    let mut project3 = Project::new(ProjectId::new(3).unwrap(), "プロジェクト3".to_string()).unwrap();
    
    // 保存
    repository.save(&project1).await.unwrap();
    repository.save(&project2).await.unwrap();
    repository.save(&project3).await.unwrap();
    
    // プロジェクト3をアーカイブ
    project3 = project3.archive();
    repository.save(&project3).await.unwrap();
    
    // アクティブなプロジェクトのみを取得
    let active_projects = repository.find_all_active().await.unwrap();
    
    assert_eq!(active_projects.len(), 2);
    assert!(active_projects.iter().any(|p| p.name() == "プロジェクト1"));
    assert!(active_projects.iter().any(|p| p.name() == "プロジェクト2"));
    assert!(!active_projects.iter().any(|p| p.name() == "プロジェクト3"));
}

#[tokio::test]
async fn test_project_repository_find_history() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteProjectRepository::new(db.clone());
    
    // プロジェクトを作成
    let project_id = ProjectId::new(1).unwrap();
    let mut project = Project::new(project_id, "履歴テスト".to_string()).unwrap();
    
    // 初期保存
    repository.save(&project).await.unwrap();
    
    // 名前変更
    project = project.change_name("変更後".to_string()).unwrap();
    repository.save(&project).await.unwrap();
    
    // アーカイブ
    project = project.archive();
    repository.save(&project).await.unwrap();
    
    // 履歴を取得
    let history = repository.find_history(project_id).await.unwrap();
    
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].name(), "履歴テスト");
    assert!(history[0].status().is_active());
    assert_eq!(history[1].name(), "変更後");
    assert!(history[1].status().is_active());
    assert_eq!(history[2].name(), "変更後");
    assert!(history[2].status().is_archived());
}

#[tokio::test]
async fn test_project_repository_with_custom_timestamp() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteProjectRepository::new(db.clone());
    
    // カスタムタイムスタンプでプロジェクトを作成
    let project_id = ProjectId::new(1).unwrap();
    let custom_time = Utc.ymd(2024, 1, 1).and_hms(12, 0, 0);
    let project = Project::new_with_time(project_id, "カスタム時間".to_string(), custom_time).unwrap();
    
    // 保存
    repository.save(&project).await.unwrap();
    
    // 取得して検証
    let found_project = repository.find_by_id(project_id).await.unwrap().unwrap();
    assert_eq!(found_project.effective_at(), custom_time);
}

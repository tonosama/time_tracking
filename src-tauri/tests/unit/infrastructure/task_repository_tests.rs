use time_tracker_go::domain::entities::Task;
use time_tracker_go::domain::value_objects::{ProjectId, TaskId, Status};
use time_tracker_go::infrastructure::database::DatabaseConnection;
use time_tracker_go::infrastructure::repositories::SqliteTaskRepository;
use chrono::{TimeZone, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_task_repository_save_and_find() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteTaskRepository::new(db.clone());
    
    // テスト用タスクを作成
    let task_id = TaskId::new(1).unwrap();
    let project_id = ProjectId::new(1).unwrap();
    let task = Task::new(task_id, project_id, "テストタスク".to_string()).unwrap();
    
    // タスクを保存
    repository.save(&task).await.unwrap();
    
    // タスクを取得して検証
    let found_task = repository.find_by_id(task_id).await.unwrap().unwrap();
    assert_eq!(found_task.id(), task_id);
    assert_eq!(found_task.project_id(), project_id);
    assert_eq!(found_task.name(), "テストタスク");
    assert!(found_task.status().is_active());
}

#[tokio::test]
async fn test_task_repository_find_by_id_not_found() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteTaskRepository::new(db.clone());
    
    // 存在しないタスクIDで検索
    let task_id = TaskId::new(999).unwrap();
    let result = repository.find_by_id(task_id).await.unwrap();
    
    assert!(result.is_none());
}

#[tokio::test]
async fn test_task_repository_update() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteTaskRepository::new(db.clone());
    
    // 初期タスクを作成
    let task_id = TaskId::new(1).unwrap();
    let project_id = ProjectId::new(1).unwrap();
    let mut task = Task::new(task_id, project_id, "初期タスク".to_string()).unwrap();
    
    // 保存
    repository.save(&task).await.unwrap();
    
    // タスク名を変更
    task = task.change_name("更新されたタスク".to_string()).unwrap();
    repository.save(&task).await.unwrap();
    
    // 更新されたタスクを取得して検証
    let found_task = repository.find_by_id(task_id).await.unwrap().unwrap();
    assert_eq!(found_task.name(), "更新されたタスク");
}

#[tokio::test]
async fn test_task_repository_archive() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteTaskRepository::new(db.clone());
    
    // タスクを作成
    let task_id = TaskId::new(1).unwrap();
    let project_id = ProjectId::new(1).unwrap();
    let mut task = Task::new(task_id, project_id, "アーカイブテスト".to_string()).unwrap();
    
    // 保存
    repository.save(&task).await.unwrap();
    
    // タスクをアーカイブ
    task = task.archive();
    repository.save(&task).await.unwrap();
    
    // アーカイブされたタスクを取得して検証
    let found_task = repository.find_by_id(task_id).await.unwrap().unwrap();
    assert!(found_task.status().is_archived());
}

#[tokio::test]
async fn test_task_repository_find_by_project() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteTaskRepository::new(db.clone());
    
    // 複数のプロジェクトとタスクを作成
    let project1 = ProjectId::new(1).unwrap();
    let project2 = ProjectId::new(2).unwrap();
    
    let task1 = Task::new(TaskId::new(1).unwrap(), project1, "プロジェクト1のタスク1".to_string()).unwrap();
    let task2 = Task::new(TaskId::new(2).unwrap(), project1, "プロジェクト1のタスク2".to_string()).unwrap();
    let task3 = Task::new(TaskId::new(3).unwrap(), project2, "プロジェクト2のタスク1".to_string()).unwrap();
    
    // 保存
    repository.save(&task1).await.unwrap();
    repository.save(&task2).await.unwrap();
    repository.save(&task3).await.unwrap();
    
    // プロジェクト1のタスクを取得
    let project1_tasks = repository.find_by_project(project1).await.unwrap();
    assert_eq!(project1_tasks.len(), 2);
    assert!(project1_tasks.iter().any(|t| t.name() == "プロジェクト1のタスク1"));
    assert!(project1_tasks.iter().any(|t| t.name() == "プロジェクト1のタスク2"));
    
    // プロジェクト2のタスクを取得
    let project2_tasks = repository.find_by_project(project2).await.unwrap();
    assert_eq!(project2_tasks.len(), 1);
    assert_eq!(project2_tasks[0].name(), "プロジェクト2のタスク1");
}

#[tokio::test]
async fn test_task_repository_find_active_by_project() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteTaskRepository::new(db.clone());
    
    let project_id = ProjectId::new(1).unwrap();
    
    // アクティブなタスク
    let task1 = Task::new(TaskId::new(1).unwrap(), project_id, "アクティブタスク1".to_string()).unwrap();
    let task2 = Task::new(TaskId::new(2).unwrap(), project_id, "アクティブタスク2".to_string()).unwrap();
    
    // アーカイブされたタスク
    let mut task3 = Task::new(TaskId::new(3).unwrap(), project_id, "アーカイブタスク".to_string()).unwrap();
    task3 = task3.archive();
    
    // 保存
    repository.save(&task1).await.unwrap();
    repository.save(&task2).await.unwrap();
    repository.save(&task3).await.unwrap();
    
    // アクティブなタスクのみを取得
    let active_tasks = repository.find_active_by_project(project_id).await.unwrap();
    
    assert_eq!(active_tasks.len(), 2);
    assert!(active_tasks.iter().any(|t| t.name() == "アクティブタスク1"));
    assert!(active_tasks.iter().any(|t| t.name() == "アクティブタスク2"));
    assert!(!active_tasks.iter().any(|t| t.name() == "アーカイブタスク"));
}

#[tokio::test]
async fn test_task_repository_find_all_active() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteTaskRepository::new(db.clone());
    
    // 複数のタスクを作成
    let project1 = ProjectId::new(1).unwrap();
    let project2 = ProjectId::new(2).unwrap();
    
    let task1 = Task::new(TaskId::new(1).unwrap(), project1, "タスク1".to_string()).unwrap();
    let task2 = Task::new(TaskId::new(2).unwrap(), project2, "タスク2".to_string()).unwrap();
    let mut task3 = Task::new(TaskId::new(3).unwrap(), project1, "タスク3".to_string()).unwrap();
    
    // 保存
    repository.save(&task1).await.unwrap();
    repository.save(&task2).await.unwrap();
    repository.save(&task3).await.unwrap();
    
    // タスク3をアーカイブ
    task3 = task3.archive();
    repository.save(&task3).await.unwrap();
    
    // すべてのアクティブなタスクを取得
    let active_tasks = repository.find_all_active().await.unwrap();
    
    assert_eq!(active_tasks.len(), 2);
    assert!(active_tasks.iter().any(|t| t.name() == "タスク1"));
    assert!(active_tasks.iter().any(|t| t.name() == "タスク2"));
    assert!(!active_tasks.iter().any(|t| t.name() == "タスク3"));
}

#[tokio::test]
async fn test_task_repository_move_to_project() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteTaskRepository::new(db.clone());
    
    // タスクを作成
    let task_id = TaskId::new(1).unwrap();
    let original_project = ProjectId::new(1).unwrap();
    let new_project = ProjectId::new(2).unwrap();
    let mut task = Task::new(task_id, original_project, "移動テスト".to_string()).unwrap();
    
    // 保存
    repository.save(&task).await.unwrap();
    
    // 別のプロジェクトに移動
    task = task.move_to_project(new_project).unwrap();
    repository.save(&task).await.unwrap();
    
    // 移動されたタスクを取得して検証
    let found_task = repository.find_by_id(task_id).await.unwrap().unwrap();
    assert_eq!(found_task.project_id(), new_project);
    
    // 元のプロジェクトには存在しないことを確認
    let original_project_tasks = repository.find_by_project(original_project).await.unwrap();
    assert_eq!(original_project_tasks.len(), 0);
    
    // 新しいプロジェクトに存在することを確認
    let new_project_tasks = repository.find_by_project(new_project).await.unwrap();
    assert_eq!(new_project_tasks.len(), 1);
    assert_eq!(new_project_tasks[0].name(), "移動テスト");
}

#[tokio::test]
async fn test_task_repository_find_history() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteTaskRepository::new(db.clone());
    
    // タスクを作成
    let task_id = TaskId::new(1).unwrap();
    let project_id = ProjectId::new(1).unwrap();
    let mut task = Task::new(task_id, project_id, "履歴テスト".to_string()).unwrap();
    
    // 初期保存
    repository.save(&task).await.unwrap();
    
    // 名前変更
    task = task.change_name("変更後".to_string()).unwrap();
    repository.save(&task).await.unwrap();
    
    // アーカイブ
    task = task.archive();
    repository.save(&task).await.unwrap();
    
    // 履歴を取得
    let history = repository.find_history(task_id).await.unwrap();
    
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].name(), "履歴テスト");
    assert!(history[0].status().is_active());
    assert_eq!(history[1].name(), "変更後");
    assert!(history[1].status().is_active());
    assert_eq!(history[2].name(), "変更後");
    assert!(history[2].status().is_archived());
}

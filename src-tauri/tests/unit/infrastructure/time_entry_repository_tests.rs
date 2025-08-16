use time_tracker_go::domain::entities::time_entry::{TimeEntry, TimeEntryEvent, TimeEntryEventType};
use time_tracker_go::domain::value_objects::TaskId;
use time_tracker_go::infrastructure::database::DatabaseConnection;
use time_tracker_go::infrastructure::repositories::SqliteTimeEntryRepository;
use chrono::{TimeZone, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_time_entry_repository_save_event() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteTimeEntryRepository::new(db.clone());
    
    // テスト用イベントを作成
    let task_id = TaskId::new(1).unwrap();
    let event = TimeEntryEvent::start(task_id, Utc::now()).unwrap();
    
    // イベントを保存
    let saved_event = repository.save_event(&event).await.unwrap();
    
    // 保存されたイベントを検証
    assert_eq!(saved_event.task_id(), task_id);
    assert_eq!(saved_event.event_type(), TimeEntryEventType::Start);
    assert!(saved_event.id().is_some());
}

#[tokio::test]
async fn test_time_entry_repository_find_running_entries() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteTimeEntryRepository::new(db.clone());
    
    let task1 = TaskId::new(1).unwrap();
    let task2 = TaskId::new(2).unwrap();
    
    // 実行中のエントリを作成
    let start_time1 = Utc.ymd(2024, 1, 1).and_hms(9, 0, 0);
    let start_time2 = Utc.ymd(2024, 1, 1).and_hms(10, 0, 0);
    
    let event1 = TimeEntryEvent::start_with_time(task1, start_time1).unwrap();
    let event2 = TimeEntryEvent::start_with_time(task2, start_time2).unwrap();
    
    // イベントを保存
    repository.save_event(&event1).await.unwrap();
    repository.save_event(&event2).await.unwrap();
    
    // 実行中のエントリを取得
    let running_entries = repository.find_running_entries().await.unwrap();
    
    assert_eq!(running_entries.len(), 2);
    assert!(running_entries.iter().any(|e| e.task_id() == task1));
    assert!(running_entries.iter().any(|e| e.task_id() == task2));
}

#[tokio::test]
async fn test_time_entry_repository_find_running_entry_by_task() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteTimeEntryRepository::new(db.clone());
    
    let task_id = TaskId::new(1).unwrap();
    let start_time = Utc.ymd(2024, 1, 1).and_hms(9, 0, 0);
    
    // 実行中のエントリを作成
    let event = TimeEntryEvent::start_with_time(task_id, start_time).unwrap();
    repository.save_event(&event).await.unwrap();
    
    // 特定のタスクの実行中エントリを取得
    let running_entry = repository.find_running_entry_by_task(task_id).await.unwrap().unwrap();
    
    assert_eq!(running_entry.task_id(), task_id);
    assert_eq!(running_entry.start_time(), start_time);
    assert!(running_entry.end_time().is_none());
}

#[tokio::test]
async fn test_time_entry_repository_find_running_entry_by_task_not_found() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteTimeEntryRepository::new(db.clone());
    
    let task_id = TaskId::new(999).unwrap();
    
    // 存在しないタスクの実行中エントリを取得
    let result = repository.find_running_entry_by_task(task_id).await.unwrap();
    
    assert!(result.is_none());
}

#[tokio::test]
async fn test_time_entry_repository_stop_running_entry() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteTimeEntryRepository::new(db.clone());
    
    let task_id = TaskId::new(1).unwrap();
    let start_time = Utc.ymd(2024, 1, 1).and_hms(9, 0, 0);
    let end_time = Utc.ymd(2024, 1, 1).and_hms(10, 0, 0);
    
    // 開始イベントを作成
    let start_event = TimeEntryEvent::start_with_time(task_id, start_time).unwrap();
    let saved_start_event = repository.save_event(&start_event).await.unwrap();
    
    // 停止イベントを作成
    let stop_event = TimeEntryEvent::stop_with_time(
        task_id, 
        end_time, 
        saved_start_event.id().unwrap()
    ).unwrap();
    repository.save_event(&stop_event).await.unwrap();
    
    // 実行中のエントリが存在しないことを確認
    let running_entries = repository.find_running_entries().await.unwrap();
    assert_eq!(running_entries.len(), 0);
    
    // 特定のタスクの実行中エントリも存在しないことを確認
    let running_entry = repository.find_running_entry_by_task(task_id).await.unwrap();
    assert!(running_entry.is_none());
}

#[tokio::test]
async fn test_time_entry_repository_find_recent_entries() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteTimeEntryRepository::new(db.clone());
    
    let task1 = TaskId::new(1).unwrap();
    let task2 = TaskId::new(2).unwrap();
    
    // 複数のエントリを作成
    let start_time1 = Utc.ymd(2024, 1, 1).and_hms(9, 0, 0);
    let end_time1 = Utc.ymd(2024, 1, 1).and_hms(10, 0, 0);
    let start_time2 = Utc.ymd(2024, 1, 1).and_hms(11, 0, 0);
    let end_time2 = Utc.ymd(2024, 1, 1).and_hms(12, 0, 0);
    
    // タスク1のエントリ
    let start_event1 = TimeEntryEvent::start_with_time(task1, start_time1).unwrap();
    let saved_start_event1 = repository.save_event(&start_event1).await.unwrap();
    let stop_event1 = TimeEntryEvent::stop_with_time(task1, end_time1, saved_start_event1.id().unwrap()).unwrap();
    repository.save_event(&stop_event1).await.unwrap();
    
    // タスク2のエントリ
    let start_event2 = TimeEntryEvent::start_with_time(task2, start_time2).unwrap();
    let saved_start_event2 = repository.save_event(&start_event2).await.unwrap();
    let stop_event2 = TimeEntryEvent::stop_with_time(task2, end_time2, saved_start_event2.id().unwrap()).unwrap();
    repository.save_event(&stop_event2).await.unwrap();
    
    // 最近のエントリを取得
    let recent_entries = repository.find_recent_entries(10).await.unwrap();
    
    assert_eq!(recent_entries.len(), 2);
    
    // 最新のエントリが最初に来ることを確認
    assert_eq!(recent_entries[0].task_id(), task2);
    assert_eq!(recent_entries[1].task_id(), task1);
}

#[tokio::test]
async fn test_time_entry_repository_find_entries_by_date_range() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteTimeEntryRepository::new(db.clone());
    
    let task_id = TaskId::new(1).unwrap();
    
    // 異なる日付のエントリを作成
    let date1_start = Utc.ymd(2024, 1, 1).and_hms(9, 0, 0);
    let date1_end = Utc.ymd(2024, 1, 1).and_hms(10, 0, 0);
    let date2_start = Utc.ymd(2024, 1, 2).and_hms(9, 0, 0);
    let date2_end = Utc.ymd(2024, 1, 2).and_hms(10, 0, 0);
    
    // 1月1日のエントリ
    let start_event1 = TimeEntryEvent::start_with_time(task_id, date1_start).unwrap();
    let saved_start_event1 = repository.save_event(&start_event1).await.unwrap();
    let stop_event1 = TimeEntryEvent::stop_with_time(task_id, date1_end, saved_start_event1.id().unwrap()).unwrap();
    repository.save_event(&stop_event1).await.unwrap();
    
    // 1月2日のエントリ
    let start_event2 = TimeEntryEvent::start_with_time(task_id, date2_start).unwrap();
    let saved_start_event2 = repository.save_event(&start_event2).await.unwrap();
    let stop_event2 = TimeEntryEvent::stop_with_time(task_id, date2_end, saved_start_event2.id().unwrap()).unwrap();
    repository.save_event(&stop_event2).await.unwrap();
    
    // 1月1日のエントリのみを取得
    let start_date = Utc.ymd(2024, 1, 1).and_hms(0, 0, 0);
    let end_date = Utc.ymd(2024, 1, 1).and_hms(23, 59, 59);
    
    let entries = repository.find_entries_by_date_range(start_date, end_date).await.unwrap();
    
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].start_time(), date1_start);
    assert_eq!(entries[0].end_time().unwrap(), date1_end);
}

#[tokio::test]
async fn test_time_entry_repository_add_annotation() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteTimeEntryRepository::new(db.clone());
    
    let task_id = TaskId::new(1).unwrap();
    let start_time = Utc.ymd(2024, 1, 1).and_hms(9, 0, 0);
    let annotation_time = Utc.ymd(2024, 1, 1).and_hms(9, 30, 0);
    
    // 開始イベントを作成
    let start_event = TimeEntryEvent::start_with_time(task_id, start_time).unwrap();
    let saved_start_event = repository.save_event(&start_event).await.unwrap();
    
    // 注釈イベントを作成
    let annotation_event = TimeEntryEvent::annotation_with_time(
        task_id,
        annotation_time,
        "テスト注釈".to_string(),
        saved_start_event.id().unwrap()
    ).unwrap();
    let saved_annotation_event = repository.save_event(&annotation_event).await.unwrap();
    
    // 注釈イベントを検証
    assert_eq!(saved_annotation_event.event_type(), TimeEntryEventType::Annotation);
    assert_eq!(saved_annotation_event.payload(), Some("テスト注釈".to_string()));
    assert_eq!(saved_annotation_event.start_event_id(), Some(saved_start_event.id().unwrap()));
}

#[tokio::test]
async fn test_time_entry_repository_find_events_by_task() {
    let db = Arc::new(Mutex::new(DatabaseConnection::new_in_memory().unwrap()));
    db.lock().await.run_migrations().unwrap();
    
    let repository = SqliteTimeEntryRepository::new(db.clone());
    
    let task_id = TaskId::new(1).unwrap();
    let start_time = Utc.ymd(2024, 1, 1).and_hms(9, 0, 0);
    let end_time = Utc.ymd(2024, 1, 1).and_hms(10, 0, 0);
    let annotation_time = Utc.ymd(2024, 1, 1).and_hms(9, 30, 0);
    
    // 開始イベント
    let start_event = TimeEntryEvent::start_with_time(task_id, start_time).unwrap();
    let saved_start_event = repository.save_event(&start_event).await.unwrap();
    
    // 注釈イベント
    let annotation_event = TimeEntryEvent::annotation_with_time(
        task_id,
        annotation_time,
        "テスト注釈".to_string(),
        saved_start_event.id().unwrap()
    ).unwrap();
    repository.save_event(&annotation_event).await.unwrap();
    
    // 停止イベント
    let stop_event = TimeEntryEvent::stop_with_time(
        task_id,
        end_time,
        saved_start_event.id().unwrap()
    ).unwrap;
    repository.save_event(&stop_event).await.unwrap();
    
    // タスクのイベントを取得
    let events = repository.find_events_by_task(task_id).await.unwrap();
    
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].event_type(), TimeEntryEventType::Start);
    assert_eq!(events[1].event_type(), TimeEntryEventType::Annotation);
    assert_eq!(events[2].event_type(), TimeEntryEventType::Stop);
}

use crate::domain::entities::time_entry::{TimeEntry, TimeEntryEvent};
use crate::domain::value_objects::TaskId;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// タイムエントリリポジトリトレイト
#[async_trait]
pub trait TimeEntryRepository: Send + Sync + Clone {
    /// イベントを保存
    async fn save_event(&self, event: &TimeEntryEvent) -> anyhow::Result<TimeEntryEvent>;

    /// 実行中の時間区間を全て取得
    async fn find_running_entries(&self) -> anyhow::Result<Vec<TimeEntry>>;

    /// 指定タスクの実行中時間区間を取得
    async fn find_running_entry_by_task(&self, task_id: TaskId) -> anyhow::Result<Option<TimeEntry>>;

    /// 指定タスクの全時間区間を取得
    async fn find_entries_by_task(&self, task_id: TaskId) -> anyhow::Result<Vec<TimeEntry>>;

    /// 指定期間のタスクの時間区間を取得
    async fn find_entries_by_task_and_period(
        &self,
        task_id: TaskId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<TimeEntry>>;

    /// 指定期間に重複する時間区間を取得
    async fn find_overlapping_entries(
        &self,
        task_id: TaskId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<TimeEntry>>;

    /// 指定プロジェクトの全時間区間を取得
    async fn find_entries_by_project(&self, project_id: i64) -> anyhow::Result<Vec<TimeEntry>>;

    /// 指定期間の全時間区間を取得
    async fn find_entries_by_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<TimeEntry>>;

    /// 指定タスクの時間区間数を取得
    async fn count_entries_by_task(&self, task_id: TaskId) -> anyhow::Result<usize>;

    /// 指定タスクの合計作業時間を取得（秒）
    async fn sum_duration_by_task(&self, task_id: TaskId) -> anyhow::Result<i64>;

    /// 指定プロジェクトの合計作業時間を取得（秒）
    async fn sum_duration_by_project(&self, project_id: i64) -> anyhow::Result<i64>;

    /// イベントIDで時間区間を取得
    async fn find_entry_by_start_event_id(&self, start_event_id: i64) -> anyhow::Result<Option<TimeEntry>>;

    /// 最新のN件の時間区間を取得
    async fn find_recent_entries(&self, limit: usize) -> anyhow::Result<Vec<TimeEntry>>;

    /// 指定タスクの最新のN件の時間区間を取得
    async fn find_recent_entries_by_task(&self, task_id: TaskId, limit: usize) -> anyhow::Result<Vec<TimeEntry>>;
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// テスト用のインメモリTimeEntryRepository実装
    #[derive(Clone)]
    pub struct InMemoryTimeEntryRepository {
        events: Arc<Mutex<Vec<TimeEntryEvent>>>,
        next_id: Arc<Mutex<i64>>,
    }

    impl InMemoryTimeEntryRepository {
        pub fn new() -> Self {
            Self {
                events: Arc::new(Mutex::new(Vec::new())),
                next_id: Arc::new(Mutex::new(1)),
            }
        }

        fn generate_id(&self) -> i64 {
            let mut id = self.next_id.lock().unwrap();
            let current = *id;
            *id += 1;
            current
        }

        fn build_time_entries(&self) -> Vec<TimeEntry> {
            let events = self.events.lock().unwrap();
            let mut entries = Vec::new();
            let mut start_events: HashMap<i64, &TimeEntryEvent> = HashMap::new();

            // 開始イベントを収集
            for event in events.iter() {
                if event.is_start() && event.id().is_some() {
                    start_events.insert(event.id().unwrap(), event);
                }
            }

            // 各開始イベントに対応する時間区間を構築
            for (start_id, start_event) in start_events {
                // 対応する停止イベントを探す
                let stop_event = events.iter().find(|e| {
                    e.is_stop() && e.start_event_id() == Some(start_id)
                });

                let end_time = stop_event.map(|e| e.at());
                let entry = TimeEntry::new(
                    start_event.task_id(),
                    start_id,
                    start_event.at(),
                    end_time,
                );
                entries.push(entry);
            }

            entries.sort_by(|a, b| b.start_time().cmp(&a.start_time())); // 新しい順
            entries
        }
    }

    #[async_trait]
    impl TimeEntryRepository for InMemoryTimeEntryRepository {
        async fn save_event(&self, event: &TimeEntryEvent) -> anyhow::Result<TimeEntryEvent> {
            let id = self.generate_id();
            let saved_event = event.clone().with_id(id);
            self.events.lock().unwrap().push(saved_event.clone());
            Ok(saved_event)
        }

        async fn find_running_entries(&self) -> anyhow::Result<Vec<TimeEntry>> {
            let entries = self.build_time_entries();
            Ok(entries.into_iter().filter(|e| e.is_running()).collect())
        }

        async fn find_running_entry_by_task(&self, task_id: TaskId) -> anyhow::Result<Option<TimeEntry>> {
            let entries = self.build_time_entries();
            Ok(entries
                .into_iter()
                .find(|e| e.task_id() == task_id && e.is_running()))
        }

        async fn find_entries_by_task(&self, task_id: TaskId) -> anyhow::Result<Vec<TimeEntry>> {
            let entries = self.build_time_entries();
            Ok(entries
                .into_iter()
                .filter(|e| e.task_id() == task_id)
                .collect())
        }

        async fn find_entries_by_task_and_period(
            &self,
            task_id: TaskId,
            start: DateTime<Utc>,
            end: DateTime<Utc>,
        ) -> anyhow::Result<Vec<TimeEntry>> {
            let entries = self.build_time_entries();
            Ok(entries
                .into_iter()
                .filter(|e| {
                    e.task_id() == task_id &&
                    e.start_time() >= start &&
                    e.start_time() <= end
                })
                .collect())
        }

        async fn find_overlapping_entries(
            &self,
            task_id: TaskId,
            start: DateTime<Utc>,
            end: DateTime<Utc>,
        ) -> anyhow::Result<Vec<TimeEntry>> {
            let entries = self.build_time_entries();
            Ok(entries
                .into_iter()
                .filter(|e| {
                    e.task_id() == task_id &&
                    e.start_time() < end &&
                    e.end_time().map_or(true, |et| et > start)
                })
                .collect())
        }

        async fn find_entries_by_project(&self, _project_id: i64) -> anyhow::Result<Vec<TimeEntry>> {
            // TODO: プロジェクトIDとタスクの関係を実装
            Ok(Vec::new())
        }

        async fn find_entries_by_period(
            &self,
            start: DateTime<Utc>,
            end: DateTime<Utc>,
        ) -> anyhow::Result<Vec<TimeEntry>> {
            let entries = self.build_time_entries();
            Ok(entries
                .into_iter()
                .filter(|e| e.start_time() >= start && e.start_time() <= end)
                .collect())
        }

        async fn count_entries_by_task(&self, task_id: TaskId) -> anyhow::Result<usize> {
            let entries = self.find_entries_by_task(task_id).await?;
            Ok(entries.len())
        }

        async fn sum_duration_by_task(&self, task_id: TaskId) -> anyhow::Result<i64> {
            let entries = self.find_entries_by_task(task_id).await?;
            let total = entries
                .iter()
                .filter_map(|e| e.duration_in_seconds())
                .sum();
            Ok(total)
        }

        async fn sum_duration_by_project(&self, project_id: i64) -> anyhow::Result<i64> {
            let entries = self.find_entries_by_project(project_id).await?;
            let total = entries
                .iter()
                .filter_map(|e| e.duration_in_seconds())
                .sum();
            Ok(total)
        }

        async fn find_entry_by_start_event_id(&self, start_event_id: i64) -> anyhow::Result<Option<TimeEntry>> {
            let entries = self.build_time_entries();
            Ok(entries
                .into_iter()
                .find(|e| e.start_event_id() == start_event_id))
        }

        async fn find_recent_entries(&self, limit: usize) -> anyhow::Result<Vec<TimeEntry>> {
            let mut entries = self.build_time_entries();
            entries.truncate(limit);
            Ok(entries)
        }

        async fn find_recent_entries_by_task(&self, task_id: TaskId, limit: usize) -> anyhow::Result<Vec<TimeEntry>> {
            let mut entries = self.find_entries_by_task(task_id).await?;
            entries.truncate(limit);
            Ok(entries)
        }
    }

    #[tokio::test]
    async fn インメモリリポジトリが正しく動作すること() {
        let repo = InMemoryTimeEntryRepository::new();
        let task_id = TaskId::new(1).unwrap();

        // 開始イベント保存
        let start_event = TimeEntryEvent::start(task_id);
        let saved_start = repo.save_event(&start_event).await.unwrap();
        assert!(saved_start.id().is_some());

        // 実行中エントリ確認
        let running = repo.find_running_entries().await.unwrap();
        assert_eq!(running.len(), 1);
        assert_eq!(running[0].task_id(), task_id);
        assert!(running[0].is_running());

        // 停止イベント保存
        let stop_event = TimeEntryEvent::stop(task_id, saved_start.id().unwrap());
        repo.save_event(&stop_event).await.unwrap();

        // 完了エントリ確認
        let entries = repo.find_entries_by_task(task_id).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].is_completed());

        // 実行中エントリが空になることを確認
        let running_after = repo.find_running_entries().await.unwrap();
        assert_eq!(running_after.len(), 0);
    }

    #[test]
    fn タイムエントリイベントの保存ができること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTimeEntryRepository::new();
            let task_id = TaskId::new(1).unwrap();
            let event = TimeEntryEvent::start(task_id);
            
            let saved_event = repository.save_event(&event).await.unwrap();
            
            assert_eq!(saved_event.task_id(), task_id);
            assert_eq!(saved_event.event_type(), &crate::domain::entities::time_entry::TimeEntryEventType::Start);
            assert!(saved_event.id().is_some());
        });
    }

    #[test]
    fn 実行中のエントリが取得できること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTimeEntryRepository::new();
            let task1 = TaskId::new(1).unwrap();
            let task2 = TaskId::new(2).unwrap();
            
            let event1 = TimeEntryEvent::start(task1);
            let event2 = TimeEntryEvent::start(task2);
            
            repository.save_event(&event1).await.unwrap();
            repository.save_event(&event2).await.unwrap();
            
            let running_entries = repository.find_running_entries().await.unwrap();
            assert_eq!(running_entries.len(), 2);
            assert!(running_entries.iter().any(|e| e.task_id() == task1));
            assert!(running_entries.iter().any(|e| e.task_id() == task2));
        });
    }

    #[test]
    fn 特定タスクの実行中エントリが取得できること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTimeEntryRepository::new();
            let task_id = TaskId::new(1).unwrap();
            
            let event = TimeEntryEvent::start(task_id);
            repository.save_event(&event).await.unwrap();
            
            let running_entry = repository.find_running_entry_by_task(task_id).await.unwrap().unwrap();
            assert_eq!(running_entry.task_id(), task_id);
            assert!(running_entry.end_time().is_none());
        });
    }

    #[test]
    fn 実行中エントリの停止ができること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTimeEntryRepository::new();
            let task_id = TaskId::new(1).unwrap();
            
            let start_event = TimeEntryEvent::start(task_id);
            let saved_start_event = repository.save_event(&start_event).await.unwrap();
            
            let stop_event = TimeEntryEvent::stop(task_id, saved_start_event.id().unwrap());
            repository.save_event(&stop_event).await.unwrap();
            
            let running_entries = repository.find_running_entries().await.unwrap();
            assert_eq!(running_entries.len(), 0);
            
            let running_entry = repository.find_running_entry_by_task(task_id).await.unwrap();
            assert!(running_entry.is_none());
        });
    }

    #[test]
    fn 最近のエントリが取得できること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTimeEntryRepository::new();
            let task1 = TaskId::new(1).unwrap();
            let task2 = TaskId::new(2).unwrap();
            
            let start_event1 = TimeEntryEvent::start(task1);
            let saved_start_event1 = repository.save_event(&start_event1).await.unwrap();
            let stop_event1 = TimeEntryEvent::stop(task1, saved_start_event1.id().unwrap());
            repository.save_event(&stop_event1).await.unwrap();
            
            let start_event2 = TimeEntryEvent::start(task2);
            let saved_start_event2 = repository.save_event(&start_event2).await.unwrap();
            let stop_event2 = TimeEntryEvent::stop(task2, saved_start_event2.id().unwrap());
            repository.save_event(&stop_event2).await.unwrap();
            
            let recent_entries = repository.find_recent_entries(10).await.unwrap();
            assert_eq!(recent_entries.len(), 2);
            
            // 最新のエントリが最初に来ることを確認
            assert_eq!(recent_entries[0].task_id(), task2);
            assert_eq!(recent_entries[1].task_id(), task1);
        });
    }

    #[test]
    fn タスクのエントリが取得できること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTimeEntryRepository::new();
            let task_id = TaskId::new(1).unwrap();
            
            let start_event = TimeEntryEvent::start(task_id);
            let saved_start_event = repository.save_event(&start_event).await.unwrap();
            
            let stop_event = TimeEntryEvent::stop(task_id, saved_start_event.id().unwrap());
            repository.save_event(&stop_event).await.unwrap();
            
            let entries = repository.find_entries_by_task(task_id).await.unwrap();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].task_id(), task_id);
        });
    }

    #[test]
    fn タスクの最近のエントリが取得できること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTimeEntryRepository::new();
            let task_id = TaskId::new(1).unwrap();
            
            let start_event = TimeEntryEvent::start(task_id);
            let saved_start_event = repository.save_event(&start_event).await.unwrap();
            
            let stop_event = TimeEntryEvent::stop(task_id, saved_start_event.id().unwrap());
            repository.save_event(&stop_event).await.unwrap();
            
            let recent_entries = repository.find_recent_entries_by_task(task_id, 5).await.unwrap();
            assert_eq!(recent_entries.len(), 1);
            assert_eq!(recent_entries[0].task_id(), task_id);
        });
    }

    // 追加のテストケース

    #[test]
    fn 複数タスクのエントリが正しく管理されること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTimeEntryRepository::new();
            let task1 = TaskId::new(1).unwrap();
            let task2 = TaskId::new(2).unwrap();
            
            // タスク1のエントリ
            let start_event1 = TimeEntryEvent::start(task1);
            let saved_start_event1 = repository.save_event(&start_event1).await.unwrap();
            let stop_event1 = TimeEntryEvent::stop(task1, saved_start_event1.id().unwrap());
            repository.save_event(&stop_event1).await.unwrap();
            
            // タスク2のエントリ
            let start_event2 = TimeEntryEvent::start(task2);
            let saved_start_event2 = repository.save_event(&start_event2).await.unwrap();
            let stop_event2 = TimeEntryEvent::stop(task2, saved_start_event2.id().unwrap());
            repository.save_event(&stop_event2).await.unwrap();
            
            // 最近のエントリを取得
            let recent_entries = repository.find_recent_entries(10).await.unwrap();
            assert_eq!(recent_entries.len(), 2);
            
            // タスク1のエントリを取得
            let task1_entries = repository.find_entries_by_task(task1).await.unwrap();
            assert_eq!(task1_entries.len(), 1);
            
            // タスク2のエントリを取得
            let task2_entries = repository.find_entries_by_task(task2).await.unwrap();
            assert_eq!(task2_entries.len(), 1);
        });
    }

    #[test]
    fn 実行中と完了したエントリが正しく区別されること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTimeEntryRepository::new();
            let task1 = TaskId::new(1).unwrap();
            let task2 = TaskId::new(2).unwrap();
            
            // 完了したエントリ
            let start_event1 = TimeEntryEvent::start(task1);
            let saved_start_event1 = repository.save_event(&start_event1).await.unwrap();
            let stop_event1 = TimeEntryEvent::stop(task1, saved_start_event1.id().unwrap());
            repository.save_event(&stop_event1).await.unwrap();
            
            // 実行中のエントリ
            let start_event2 = TimeEntryEvent::start(task2);
            repository.save_event(&start_event2).await.unwrap();
            
            // 実行中のエントリを確認
            let running_entries = repository.find_running_entries().await.unwrap();
            assert_eq!(running_entries.len(), 1);
            assert_eq!(running_entries[0].task_id(), task2);
            
            // 最近のエントリを確認
            let recent_entries = repository.find_recent_entries(10).await.unwrap();
            assert_eq!(recent_entries.len(), 2);
        });
    }

    #[test]
    fn 空のリポジトリでの動作確認() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTimeEntryRepository::new();
            
            // 空のリポジトリで各種クエリを実行
            let running_entries = repository.find_running_entries().await.unwrap();
            assert_eq!(running_entries.len(), 0);
            
            let recent_entries = repository.find_recent_entries(10).await.unwrap();
            assert_eq!(recent_entries.len(), 0);
            
            let task_id = TaskId::new(999).unwrap();
            let running_entry = repository.find_running_entry_by_task(task_id).await.unwrap();
            assert!(running_entry.is_none());
            
            let entries = repository.find_entries_by_task(task_id).await.unwrap();
            assert_eq!(entries.len(), 0);
        });
    }



    #[test]
    fn イベントのペイロードが正しく保存されること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTimeEntryRepository::new();
            let task_id = TaskId::new(1).unwrap();
            
            // 開始イベント（ペイロードなし）
            let start_event = TimeEntryEvent::start(task_id);
            let saved_start_event = repository.save_event(&start_event).await.unwrap();
            assert!(saved_start_event.payload().is_none());
            
            // 注釈イベント（ペイロードあり）
            let annotation_event = TimeEntryEvent::annotate(task_id, saved_start_event.id().unwrap(), "テスト注釈".to_string());
            let saved_annotation_event = repository.save_event(&annotation_event).await.unwrap();
            assert_eq!(saved_annotation_event.payload(), Some("テスト注釈"));
            
            // 停止イベント（ペイロードなし）
            let stop_event = TimeEntryEvent::stop(task_id, saved_start_event.id().unwrap());
            let saved_stop_event = repository.save_event(&stop_event).await.unwrap();
            assert!(saved_stop_event.payload().is_none());
        });
    }

    #[test]
    fn 複数イベントの同時操作ができること() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let repository = InMemoryTimeEntryRepository::new();
            let task_id = TaskId::new(1).unwrap();
            
            // 複数のイベントを同時に作成
            let mut handles = vec![];
            
            for _i in 1..=5 {
                let repository_clone = repository.clone();
                let handle = tokio::spawn(async move {
                    let event = TimeEntryEvent::start(task_id);
                    repository_clone.save_event(&event).await.unwrap();
                });
                handles.push(handle);
            }
            
            // すべてのタスクが完了するまで待機
            for handle in handles {
                handle.await.unwrap();
            }
            
            // すべてのイベントが保存されていることを確認
            let entries = repository.find_entries_by_task(task_id).await.unwrap();
            assert_eq!(entries.len(), 5);
            
            let running_entries = repository.find_running_entries().await.unwrap();
            assert_eq!(running_entries.len(), 5);
        });
    }
}

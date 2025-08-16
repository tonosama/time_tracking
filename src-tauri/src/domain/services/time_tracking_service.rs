use crate::domain::entities::time_entry::{TimeEntryEvent};
use crate::domain::repositories::TimeEntryRepository;
use crate::domain::value_objects::TaskId;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// タイムトラッキングサービス
#[async_trait]
pub trait TimeTrackingService: Send + Sync {
    /// タイマーを開始する
    /// 同一タスクで実行中の区間がある場合は、自動的に停止してから新しい区間を開始
    async fn start_timer(&self, task_id: TaskId) -> anyhow::Result<TimeEntryEvent>;

    /// タイマーを停止する
    async fn stop_timer(&self, task_id: TaskId) -> anyhow::Result<Option<TimeEntryEvent>>;

    /// 現在実行中のタスクを取得
    async fn get_running_task(&self) -> anyhow::Result<Option<TaskId>>;

    /// 指定タスクが実行中かどうか
    async fn is_task_running(&self, task_id: TaskId) -> anyhow::Result<bool>;

    /// 手動で時間区間を追加
    async fn add_manual_entry(
        &self,
        task_id: TaskId,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        note: Option<String>,
    ) -> anyhow::Result<(TimeEntryEvent, TimeEntryEvent)>;

    /// 全ての実行中タイマーを停止
    async fn stop_all_timers(&self) -> anyhow::Result<Vec<TimeEntryEvent>>;
}

/// タイムトラッキングサービス実装
pub struct TimeTrackingServiceImpl<R: TimeEntryRepository> {
    repository: R,
}

impl<R: TimeEntryRepository> TimeTrackingServiceImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: TimeEntryRepository> TimeTrackingService for TimeTrackingServiceImpl<R> {
    async fn start_timer(&self, task_id: TaskId) -> anyhow::Result<TimeEntryEvent> {
        // Step 1: 他のタスクで実行中のタイマーがあれば停止
        let running_entries = self.repository.find_running_entries().await?;
        for entry in running_entries {
            if entry.task_id() != task_id {
                // 他のタスクのタイマーを停止
                let stop_event = TimeEntryEvent::stop(entry.task_id(), entry.start_event_id());
                self.repository.save_event(&stop_event).await?;
            }
        }

        // Step 2: 同一タスクで実行中の区間があれば停止（暗黙stop）
        let running_for_task = self.repository.find_running_entry_by_task(task_id).await?;
        if let Some(running_entry) = running_for_task {
            let stop_event = TimeEntryEvent::stop(task_id, running_entry.start_event_id());
            self.repository.save_event(&stop_event).await?;
        }

        // Step 3: 新しい開始イベントを作成・保存
        let start_event = TimeEntryEvent::start(task_id);
        let saved_event = self.repository.save_event(&start_event).await?;
        
        Ok(saved_event)
    }

    async fn stop_timer(&self, task_id: TaskId) -> anyhow::Result<Option<TimeEntryEvent>> {
        // 実行中の区間を取得
        let running_entry = self.repository.find_running_entry_by_task(task_id).await?;
        
        match running_entry {
            Some(entry) => {
                let stop_event = TimeEntryEvent::stop(task_id, entry.start_event_id());
                let saved_event = self.repository.save_event(&stop_event).await?;
                Ok(Some(saved_event))
            }
            None => Ok(None), // 実行中のタイマーがない
        }
    }

    async fn get_running_task(&self) -> anyhow::Result<Option<TaskId>> {
        let running_entries = self.repository.find_running_entries().await?;
        Ok(running_entries.first().map(|entry| entry.task_id()))
    }

    async fn is_task_running(&self, task_id: TaskId) -> anyhow::Result<bool> {
        let running_entry = self.repository.find_running_entry_by_task(task_id).await?;
        Ok(running_entry.is_some())
    }

    async fn add_manual_entry(
        &self,
        task_id: TaskId,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        note: Option<String>,
    ) -> anyhow::Result<(TimeEntryEvent, TimeEntryEvent)> {
        // バリデーション
        if start_time >= end_time {
            return Err(anyhow::anyhow!("Start time must be before end time"));
        }

        // 重複チェック（既存の区間と重複しないか）
        let overlapping = self
            .repository
            .find_overlapping_entries(task_id, start_time, end_time)
            .await?;
        
        if !overlapping.is_empty() {
            return Err(anyhow::anyhow!("Time entry overlaps with existing entries"));
        }

        // 手動開始イベントを作成・保存
        let start_event = TimeEntryEvent::start_at(task_id, start_time);
        let saved_start_event = self.repository.save_event(&start_event).await?;

        // 手動停止イベントを作成・保存
        let stop_event = TimeEntryEvent::stop_at(
            task_id,
            saved_start_event.id().unwrap(),
            end_time,
        );
        let saved_stop_event = self.repository.save_event(&stop_event).await?;

        // 注釈がある場合は注釈イベントも保存
        if let Some(note_text) = note {
            let annotate_event = TimeEntryEvent::annotate(
                task_id,
                saved_start_event.id().unwrap(),
                note_text,
            );
            self.repository.save_event(&annotate_event).await?;
        }

        Ok((saved_start_event, saved_stop_event))
    }

    async fn stop_all_timers(&self) -> anyhow::Result<Vec<TimeEntryEvent>> {
        let running_entries = self.repository.find_running_entries().await?;
        let mut stop_events = Vec::new();

        for entry in running_entries {
            let stop_event = TimeEntryEvent::stop(entry.task_id(), entry.start_event_id());
            let saved_event = self.repository.save_event(&stop_event).await?;
            stop_events.push(saved_event);
        }

        Ok(stop_events)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::domain::repositories::time_entry_repository::tests::InMemoryTimeEntryRepository;
    use chrono::TimeZone;

    async fn setup_service() -> TimeTrackingServiceImpl<InMemoryTimeEntryRepository> {
        let repository = InMemoryTimeEntryRepository::new();
        TimeTrackingServiceImpl::new(repository)
    }

    #[tokio::test]
    async fn タイマー開始が正しく動作すること() {
        let service = setup_service().await;
        let task_id = TaskId::new(1).unwrap();

        let start_event = service.start_timer(task_id).await.unwrap();

        assert_eq!(start_event.task_id(), task_id);
        assert!(start_event.is_start());
        assert!(start_event.id().is_some());

        // 実行中状態を確認
        assert!(service.is_task_running(task_id).await.unwrap());
        assert_eq!(service.get_running_task().await.unwrap(), Some(task_id));
    }

    #[tokio::test]
    async fn タイマー停止が正しく動作すること() {
        let service = setup_service().await;
        let task_id = TaskId::new(1).unwrap();

        // 開始
        let start_event = service.start_timer(task_id).await.unwrap();

        // 停止
        let stop_event = service.stop_timer(task_id).await.unwrap();

        assert!(stop_event.is_some());
        let stop_event = stop_event.unwrap();
        assert_eq!(stop_event.task_id(), task_id);
        assert!(stop_event.is_stop());
        assert_eq!(stop_event.start_event_id(), start_event.id());

        // 停止状態を確認
        assert!(!service.is_task_running(task_id).await.unwrap());
        assert_eq!(service.get_running_task().await.unwrap(), None);
    }

    #[tokio::test]
    async fn 同一タスクの連続開始で暗黙停止が動作すること() {
        let service = setup_service().await;
        let task_id = TaskId::new(1).unwrap();

        // 最初の開始
        let first_start = service.start_timer(task_id).await.unwrap();
        assert!(service.is_task_running(task_id).await.unwrap());

        // 同じタスクで再度開始（暗黙停止 + 新規開始）
        let second_start = service.start_timer(task_id).await.unwrap();
        assert!(service.is_task_running(task_id).await.unwrap());

        // 新しい開始イベントであることを確認
        assert_ne!(first_start.id(), second_start.id());

        // 最初の区間は停止されているはず
        let entries = service.repository.find_entries_by_task(task_id).await.unwrap();
        let first_entry = entries.iter().find(|e| e.start_event_id() == first_start.id().unwrap());
        assert!(first_entry.is_some());
        assert!(!first_entry.unwrap().is_running());
    }

    #[tokio::test]
    async fn 複数タスクの排他制御が動作すること() {
        let service = setup_service().await;
        let task1 = TaskId::new(1).unwrap();
        let task2 = TaskId::new(2).unwrap();

        // タスク1で開始
        service.start_timer(task1).await.unwrap();
        assert!(service.is_task_running(task1).await.unwrap());
        assert_eq!(service.get_running_task().await.unwrap(), Some(task1));

        // タスク2で開始（タスク1は自動停止される）
        service.start_timer(task2).await.unwrap();
        assert!(!service.is_task_running(task1).await.unwrap());
        assert!(service.is_task_running(task2).await.unwrap());
        assert_eq!(service.get_running_task().await.unwrap(), Some(task2));
    }

    #[tokio::test]
    async fn 手動エントリ追加が正しく動作すること() {
        let service = setup_service().await;
        let task_id = TaskId::new(1).unwrap();
        let start_time = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
        let end_time = Utc.with_ymd_and_hms(2024, 1, 1, 11, 0, 0).unwrap();
        let note = Some("手動追加".to_string());

        let (start_event, stop_event) = service
            .add_manual_entry(task_id, start_time, end_time, note)
            .await
            .unwrap();

        assert_eq!(start_event.at(), start_time);
        assert_eq!(stop_event.at(), end_time);
        assert_eq!(stop_event.start_event_id(), start_event.id());

        // エントリが作成されていることを確認
        let entries = service.repository.find_entries_by_task(task_id).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].is_completed());
        assert_eq!(entries[0].duration_in_seconds(), Some(3600)); // 1時間
    }

    #[tokio::test]
    async fn 無効な手動エントリ追加が失敗すること() {
        let service = setup_service().await;
        let task_id = TaskId::new(1).unwrap();
        let start_time = Utc.with_ymd_and_hms(2024, 1, 1, 11, 0, 0).unwrap();
        let end_time = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(); // 開始時刻が終了時刻より後

        let result = service
            .add_manual_entry(task_id, start_time, end_time, None)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Start time must be before end time"));
    }

    #[tokio::test]
    async fn 実行中でないタスクの停止で何も起こらないこと() {
        let service = setup_service().await;
        let task_id = TaskId::new(1).unwrap();

        let stop_event = service.stop_timer(task_id).await.unwrap();
        assert!(stop_event.is_none());
    }

    #[tokio::test]
    async fn 全タイマー停止が正しく動作すること() {
        let service = setup_service().await;
        let task1 = TaskId::new(1).unwrap();
        let task2 = TaskId::new(2).unwrap();

        // 複数のタスクでタイマーを開始（これは実際には排他制御により最後のもののみ実行中）
        service.start_timer(task1).await.unwrap();
        service.start_timer(task2).await.unwrap();

        let stop_events = service.stop_all_timers().await.unwrap();
        assert_eq!(stop_events.len(), 1); // 実行中のタスクは1つのみ
        assert_eq!(stop_events[0].task_id(), task2);

        // 全てのタスクが停止していることを確認
        assert!(!service.is_task_running(task1).await.unwrap());
        assert!(!service.is_task_running(task2).await.unwrap());
        assert_eq!(service.get_running_task().await.unwrap(), None);
    }
}

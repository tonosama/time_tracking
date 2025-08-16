use crate::domain::value_objects::TaskId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// タイムエントリイベントの種類
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeEntryEventType {
    Start,
    Stop,
    Annotate,
}

impl TimeEntryEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeEntryEventType::Start => "start",
            TimeEntryEventType::Stop => "stop",
            TimeEntryEventType::Annotate => "annotate",
        }
    }

    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "start" => Ok(TimeEntryEventType::Start),
            "stop" => Ok(TimeEntryEventType::Stop),
            "annotate" => Ok(TimeEntryEventType::Annotate),
            _ => Err(anyhow::anyhow!("Invalid time entry event type: {}", s)),
        }
    }
}

/// タイムエントリイベント
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeEntryEvent {
    id: Option<i64>,
    task_id: TaskId,
    event_type: TimeEntryEventType,
    at: DateTime<Utc>,
    start_event_id: Option<i64>,
    payload: Option<String>,
}

impl TimeEntryEvent {
    /// 開始イベントを作成
    pub fn start(task_id: TaskId) -> Self {
        Self {
            id: None,
            task_id,
            event_type: TimeEntryEventType::Start,
            at: Utc::now(),
            start_event_id: None,
            payload: None,
        }
    }

    /// 指定時刻での開始イベントを作成
    pub fn start_at(task_id: TaskId, at: DateTime<Utc>) -> Self {
        Self {
            id: None,
            task_id,
            event_type: TimeEntryEventType::Start,
            at,
            start_event_id: None,
            payload: None,
        }
    }

    /// 停止イベントを作成
    pub fn stop(task_id: TaskId, start_event_id: i64) -> Self {
        Self {
            id: None,
            task_id,
            event_type: TimeEntryEventType::Stop,
            at: Utc::now(),
            start_event_id: Some(start_event_id),
            payload: None,
        }
    }

    /// 指定時刻での停止イベントを作成
    pub fn stop_at(task_id: TaskId, start_event_id: i64, at: DateTime<Utc>) -> Self {
        Self {
            id: None,
            task_id,
            event_type: TimeEntryEventType::Stop,
            at,
            start_event_id: Some(start_event_id),
            payload: None,
        }
    }

    /// 注釈イベントを作成
    pub fn annotate(task_id: TaskId, start_event_id: i64, note: String) -> Self {
        Self {
            id: None,
            task_id,
            event_type: TimeEntryEventType::Annotate,
            at: Utc::now(),
            start_event_id: Some(start_event_id),
            payload: Some(note),
        }
    }

    /// IDを設定（保存後に使用）
    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    // Getters
    pub fn id(&self) -> Option<i64> {
        self.id
    }

    pub fn task_id(&self) -> TaskId {
        self.task_id
    }

    pub fn event_type(&self) -> &TimeEntryEventType {
        &self.event_type
    }

    pub fn at(&self) -> DateTime<Utc> {
        self.at
    }

    pub fn start_event_id(&self) -> Option<i64> {
        self.start_event_id
    }

    pub fn payload(&self) -> Option<&str> {
        self.payload.as_deref()
    }

    pub fn is_start(&self) -> bool {
        matches!(self.event_type, TimeEntryEventType::Start)
    }

    pub fn is_stop(&self) -> bool {
        matches!(self.event_type, TimeEntryEventType::Stop)
    }

    pub fn is_annotate(&self) -> bool {
        matches!(self.event_type, TimeEntryEventType::Annotate)
    }
}

/// 時間区間（開始と終了の組み合わせ）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeEntry {
    task_id: TaskId,
    start_event_id: i64,
    start_time: DateTime<Utc>,
    end_time: Option<DateTime<Utc>>,
    duration_in_seconds: Option<i64>,
}

impl TimeEntry {
    pub fn new(
        task_id: TaskId,
        start_event_id: i64,
        start_time: DateTime<Utc>,
        end_time: Option<DateTime<Utc>>,
    ) -> Self {
        let duration_in_seconds = end_time.map(|end| (end - start_time).num_seconds());
        
        Self {
            task_id,
            start_event_id,
            start_time,
            end_time,
            duration_in_seconds,
        }
    }

    // Getters
    pub fn task_id(&self) -> TaskId {
        self.task_id
    }

    pub fn start_event_id(&self) -> i64 {
        self.start_event_id
    }

    pub fn start_time(&self) -> DateTime<Utc> {
        self.start_time
    }

    pub fn end_time(&self) -> Option<DateTime<Utc>> {
        self.end_time
    }

    pub fn duration_in_seconds(&self) -> Option<i64> {
        self.duration_in_seconds
    }

    pub fn is_running(&self) -> bool {
        self.end_time.is_none()
    }

    pub fn is_completed(&self) -> bool {
        self.end_time.is_some()
    }

    /// 経過時間を秒単位で取得（実行中の場合は現在時刻まで）
    pub fn elapsed_seconds(&self) -> i64 {
        match self.end_time {
            Some(end) => (end - self.start_time).num_seconds(),
            None => (Utc::now() - self.start_time).num_seconds(),
        }
    }

    /// 経過時間を時:分:秒の形式で取得
    pub fn elapsed_duration(&self) -> String {
        let seconds = self.elapsed_seconds();
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn 開始イベント作成が正しく動作すること() {
        let task_id = TaskId::new(1).unwrap();
        let event = TimeEntryEvent::start(task_id);

        assert_eq!(event.task_id(), task_id);
        assert_eq!(event.event_type(), &TimeEntryEventType::Start);
        assert!(event.is_start());
        assert!(!event.is_stop());
        assert!(!event.is_annotate());
        assert!(event.start_event_id().is_none());
        assert!(event.payload().is_none());
    }

    #[test]
    fn 停止イベント作成が正しく動作すること() {
        let task_id = TaskId::new(1).unwrap();
        let start_event_id = 123;
        let event = TimeEntryEvent::stop(task_id, start_event_id);

        assert_eq!(event.task_id(), task_id);
        assert_eq!(event.event_type(), &TimeEntryEventType::Stop);
        assert!(!event.is_start());
        assert!(event.is_stop());
        assert!(!event.is_annotate());
        assert_eq!(event.start_event_id(), Some(start_event_id));
        assert!(event.payload().is_none());
    }

    #[test]
    fn 注釈イベント作成が正しく動作すること() {
        let task_id = TaskId::new(1).unwrap();
        let start_event_id = 123;
        let note = "重要な作業".to_string();
        let event = TimeEntryEvent::annotate(task_id, start_event_id, note.clone());

        assert_eq!(event.task_id(), task_id);
        assert_eq!(event.event_type(), &TimeEntryEventType::Annotate);
        assert!(!event.is_start());
        assert!(!event.is_stop());
        assert!(event.is_annotate());
        assert_eq!(event.start_event_id(), Some(start_event_id));
        assert_eq!(event.payload(), Some(note.as_str()));
    }

    #[test]
    fn タイムエントリ作成が正しく動作すること() {
        let task_id = TaskId::new(1).unwrap();
        let start_time = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
        let end_time = Some(Utc.with_ymd_and_hms(2024, 1, 1, 11, 30, 0).unwrap());
        
        let entry = TimeEntry::new(task_id, 123, start_time, end_time);

        assert_eq!(entry.task_id(), task_id);
        assert_eq!(entry.start_event_id(), 123);
        assert_eq!(entry.start_time(), start_time);
        assert_eq!(entry.end_time(), end_time);
        assert_eq!(entry.duration_in_seconds(), Some(5400)); // 1.5時間 = 5400秒
        assert!(!entry.is_running());
        assert!(entry.is_completed());
    }

    #[test]
    fn 実行中タイムエントリが正しく動作すること() {
        let task_id = TaskId::new(1).unwrap();
        let start_time = Utc::now() - chrono::Duration::seconds(3661); // 1時間1分1秒前
        
        let entry = TimeEntry::new(task_id, 123, start_time, None);

        assert!(entry.is_running());
        assert!(!entry.is_completed());
        assert!(entry.duration_in_seconds().is_none());
        
        let elapsed = entry.elapsed_seconds();
        assert!(elapsed >= 3661); // 最低でも1時間1分1秒経過
        
        let duration_str = entry.elapsed_duration();
        assert!(duration_str.starts_with("01:01")); // 1時間1分以上
    }

    #[test]
    fn イベントタイプ文字列変換が正しく動作すること() {
        assert_eq!(TimeEntryEventType::Start.as_str(), "start");
        assert_eq!(TimeEntryEventType::Stop.as_str(), "stop");
        assert_eq!(TimeEntryEventType::Annotate.as_str(), "annotate");

        assert_eq!(TimeEntryEventType::from_str("start").unwrap(), TimeEntryEventType::Start);
        assert_eq!(TimeEntryEventType::from_str("stop").unwrap(), TimeEntryEventType::Stop);
        assert_eq!(TimeEntryEventType::from_str("annotate").unwrap(), TimeEntryEventType::Annotate);
        
        assert!(TimeEntryEventType::from_str("invalid").is_err());
    }

    #[test]
    fn 指定時刻でのイベント作成が正しく動作すること() {
        let task_id = TaskId::new(1).unwrap();
        let specific_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();

        let start_event = TimeEntryEvent::start_at(task_id, specific_time);
        assert_eq!(start_event.at(), specific_time);

        let stop_event = TimeEntryEvent::stop_at(task_id, 123, specific_time);
        assert_eq!(stop_event.at(), specific_time);
    }
}

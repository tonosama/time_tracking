use crate::domain::entities::time_entry::{TimeEntry, TimeEntryEvent};
use crate::domain::value_objects::TaskId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// タイマー開始リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartTimerRequest {
    pub task_id: i64,
}

/// タイマー停止リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopTimerRequest {
    pub task_id: i64,
}

/// 手動エントリ追加リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddManualEntryRequest {
    pub task_id: i64,
    pub start_time: String, // ISO 8601形式
    pub end_time: String,   // ISO 8601形式
    pub note: Option<String>,
}

/// タイムエントリイベントレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntryEventResponse {
    pub id: i64,
    pub task_id: i64,
    pub event_type: String,
    pub at: String, // ISO 8601形式
    pub start_event_id: Option<i64>,
    pub payload: Option<String>,
}

/// タイムエントリレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntryResponse {
    pub task_id: i64,
    pub start_event_id: i64,
    pub start_time: String, // ISO 8601形式
    pub end_time: Option<String>, // ISO 8601形式
    pub duration_in_seconds: Option<i64>,
    pub elapsed_duration: String, // HH:MM:SS形式
    pub is_running: bool,
    pub is_completed: bool,
}

/// タイマー状態レスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerStatusResponse {
    pub is_running: bool,
    pub current_entry: Option<TimeEntryResponse>,
    pub elapsed_seconds: Option<i64>,
    pub elapsed_duration: Option<String>, // HH:MM:SS形式
}

/// 現在のタイマーレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentTimerResponse {
    pub task_id: Option<i64>,
    pub elapsed_seconds: Option<i64>,
    pub elapsed_duration: Option<String>,
}

/// タスク作業時間サマリーレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTimeSummaryResponse {
    pub task_id: i64,
    pub total_duration_seconds: i64,
    pub total_duration_formatted: String, // HH:MM:SS形式
    pub entry_count: usize,
    pub is_running: bool,
}

/// プロジェクト作業時間サマリーレスポンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTimeSummaryResponse {
    pub project_id: i64,
    pub total_duration_seconds: i64,
    pub total_duration_formatted: String, // HH:MM:SS形式
    pub entry_count: usize,
}

// 変換実装

impl From<TimeEntryEvent> for TimeEntryEventResponse {
    fn from(event: TimeEntryEvent) -> Self {
        Self {
            id: event.id().unwrap_or(0),
            task_id: i64::from(event.task_id()),
            event_type: event.event_type().as_str().to_string(),
            at: format_datetime(event.at()),
            start_event_id: event.start_event_id(),
            payload: event.payload().map(|s| s.to_string()),
        }
    }
}

impl From<TimeEntry> for TimeEntryResponse {
    fn from(entry: TimeEntry) -> Self {
        Self {
            task_id: i64::from(entry.task_id()),
            start_event_id: entry.start_event_id(),
            start_time: format_datetime(entry.start_time()),
            end_time: entry.end_time().map(format_datetime),
            duration_in_seconds: entry.duration_in_seconds(),
            elapsed_duration: entry.elapsed_duration(),
            is_running: entry.is_running(),
            is_completed: entry.is_completed(),
        }
    }
}

impl StartTimerRequest {
    pub fn to_command(self) -> anyhow::Result<crate::application::use_cases::StartTimerCommand> {
        let task_id = TaskId::new(self.task_id)?;
        Ok(crate::application::use_cases::StartTimerCommand { task_id })
    }
}

impl StopTimerRequest {
    pub fn to_command(self) -> anyhow::Result<crate::application::use_cases::StopTimerCommand> {
        let task_id = TaskId::new(self.task_id)?;
        Ok(crate::application::use_cases::StopTimerCommand { task_id })
    }
}

impl AddManualEntryRequest {
    pub fn to_command(self) -> anyhow::Result<crate::application::use_cases::AddManualEntryCommand> {
        let task_id = TaskId::new(self.task_id)?;
        let start_time = parse_datetime(&self.start_time)?;
        let end_time = parse_datetime(&self.end_time)?;
        
        Ok(crate::application::use_cases::AddManualEntryCommand {
            task_id,
            start_time,
            end_time,
            note: self.note,
        })
    }
}

// ユーティリティ関数

fn format_datetime(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn parse_datetime(s: &str) -> anyhow::Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(s)?.with_timezone(&Utc))
}

fn format_duration_seconds(seconds: i64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

impl TimerStatusResponse {
    pub fn new(
        is_running: bool,
        current_entry: Option<TimeEntry>,
        elapsed_seconds: Option<i64>,
    ) -> Self {
        let elapsed_duration = elapsed_seconds.map(format_duration_seconds);
        
        Self {
            is_running,
            current_entry: current_entry.map(TimeEntryResponse::from),
            elapsed_seconds,
            elapsed_duration,
        }
    }
}

impl CurrentTimerResponse {
    pub fn new(task_id: Option<TaskId>, elapsed_seconds: Option<i64>) -> Self {
        let elapsed_duration = elapsed_seconds.map(format_duration_seconds);
        
        Self {
            task_id: task_id.map(i64::from),
            elapsed_seconds,
            elapsed_duration,
        }
    }
}

impl TaskTimeSummaryResponse {
    pub fn new(
        task_id: TaskId,
        total_duration_seconds: i64,
        entry_count: usize,
        is_running: bool,
    ) -> Self {
        Self {
            task_id: i64::from(task_id),
            total_duration_seconds,
            total_duration_formatted: format_duration_seconds(total_duration_seconds),
            entry_count,
            is_running,
        }
    }
}

impl ProjectTimeSummaryResponse {
    pub fn new(
        project_id: i64,
        total_duration_seconds: i64,
        entry_count: usize,
    ) -> Self {
        Self {
            project_id,
            total_duration_seconds,
            total_duration_formatted: format_duration_seconds(total_duration_seconds),
            entry_count,
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn タイムエントリイベント変換が正しく動作すること() {
        let task_id = TaskId::new(1).unwrap();
        let event = TimeEntryEvent::start(task_id).with_id(123);
        
        let response: TimeEntryEventResponse = event.into();
        
        assert_eq!(response.id, 123);
        assert_eq!(response.task_id, 1);
        assert_eq!(response.event_type, "start");
        assert!(response.start_event_id.is_none());
        assert!(response.payload.is_none());
    }

    #[test]
    fn タイムエントリ変換が正しく動作すること() {
        let task_id = TaskId::new(1).unwrap();
        let start_time = Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap();
        let end_time = Some(Utc.with_ymd_and_hms(2024, 1, 1, 11, 30, 0).unwrap());
        
        let entry = TimeEntry::new(task_id, 123, start_time, end_time);
        let response: TimeEntryResponse = entry.into();
        
        assert_eq!(response.task_id, 1);
        assert_eq!(response.start_event_id, 123);
        assert_eq!(response.start_time, "2024-01-01T10:00:00Z");
        assert_eq!(response.end_time, Some("2024-01-01T11:30:00Z".to_string()));
        assert_eq!(response.duration_in_seconds, Some(5400));
        assert_eq!(response.elapsed_duration, "01:30:00");
        assert!(!response.is_running);
        assert!(response.is_completed);
    }

    #[test]
    fn 実行中タイムエントリ変換が正しく動作すること() {
        let task_id = TaskId::new(1).unwrap();
        let start_time = Utc::now() - chrono::Duration::seconds(3661); // 1時間1分1秒前
        
        let entry = TimeEntry::new(task_id, 123, start_time, None);
        let response: TimeEntryResponse = entry.into();
        
        assert_eq!(response.task_id, 1);
        assert!(response.end_time.is_none());
        assert!(response.duration_in_seconds.is_none());
        assert!(response.is_running);
        assert!(!response.is_completed);
        assert!(response.elapsed_duration.starts_with("01:01"));
    }

    #[test]
    fn リクエスト変換が正しく動作すること() {
        let start_request = StartTimerRequest { task_id: 1 };
        let command = start_request.to_command().unwrap();
        assert_eq!(i64::from(command.task_id), 1);

        let stop_request = StopTimerRequest { task_id: 1 };
        let command = stop_request.to_command().unwrap();
        assert_eq!(i64::from(command.task_id), 1);

        let manual_request = AddManualEntryRequest {
            task_id: 1,
            start_time: "2024-01-01T10:00:00Z".to_string(),
            end_time: "2024-01-01T11:00:00Z".to_string(),
            note: Some("テスト".to_string()),
        };
        let command = manual_request.to_command().unwrap();
        assert_eq!(i64::from(command.task_id), 1);
        assert_eq!(command.note, Some("テスト".to_string()));
    }

    #[test]
    fn 時間フォーマットが正しく動作すること() {
        assert_eq!(format_duration_seconds(0), "00:00:00");
        assert_eq!(format_duration_seconds(3661), "01:01:01");
        assert_eq!(format_duration_seconds(7200), "02:00:00");
        assert_eq!(format_duration_seconds(86400), "24:00:00"); // 24時間
    }

    #[test]
    fn タイマー状態レスポンス作成が正しく動作すること() {
        let response = TimerStatusResponse::new(false, None, None);
        assert!(!response.is_running);
        assert!(response.current_entry.is_none());
        assert!(response.elapsed_seconds.is_none());
        assert!(response.elapsed_duration.is_none());

        let response = TimerStatusResponse::new(true, None, Some(3661));
        assert!(response.is_running);
        assert_eq!(response.elapsed_seconds, Some(3661));
        assert_eq!(response.elapsed_duration, Some("01:01:01".to_string()));
    }

    #[test]
    fn 現在のタイマーレスポンス作成が正しく動作すること() {
        let task_id = TaskId::new(1).unwrap();
        let response = CurrentTimerResponse::new(Some(task_id), Some(3600));
        
        assert_eq!(response.task_id, Some(1));
        assert_eq!(response.elapsed_seconds, Some(3600));
        assert_eq!(response.elapsed_duration, Some("01:00:00".to_string()));
    }

    #[test]
    fn サマリーレスポンス作成が正しく動作すること() {
        let task_id = TaskId::new(1).unwrap();
        let task_summary = TaskTimeSummaryResponse::new(task_id, 7200, 2, true);
        
        assert_eq!(task_summary.task_id, 1);
        assert_eq!(task_summary.total_duration_seconds, 7200);
        assert_eq!(task_summary.total_duration_formatted, "02:00:00");
        assert_eq!(task_summary.entry_count, 2);
        assert!(task_summary.is_running);

        let project_summary = ProjectTimeSummaryResponse::new(1, 14400, 4);
        assert_eq!(project_summary.project_id, 1);
        assert_eq!(project_summary.total_duration_seconds, 14400);
        assert_eq!(project_summary.total_duration_formatted, "04:00:00");
        assert_eq!(project_summary.entry_count, 4);
    }

    #[test]
    fn 無効なリクエスト変換が失敗すること() {
        let invalid_request = StartTimerRequest { task_id: 0 };
        assert!(invalid_request.to_command().is_err());

        let invalid_manual_request = AddManualEntryRequest {
            task_id: 1,
            start_time: "invalid-date".to_string(),
            end_time: "2024-01-01T11:00:00Z".to_string(),
            note: None,
        };
        assert!(invalid_manual_request.to_command().is_err());
    }
}

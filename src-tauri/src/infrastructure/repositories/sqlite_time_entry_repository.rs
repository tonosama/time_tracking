use crate::domain::entities::time_entry::{TimeEntry, TimeEntryEvent};
use crate::domain::value_objects::TaskId;
use crate::domain::repositories::TimeEntryRepository;
use crate::infrastructure::database::DatabaseConnection;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::sync::Arc;
use tokio::sync::Mutex;

/// データベース操作の監視用ヘルパー関数
trait DatabaseOperationLogger {
    fn log_insert(&self, table: &str, data: &str);
    #[allow(dead_code)]
    fn log_update(&self, table: &str, condition: &str, data: &str);
    #[allow(dead_code)]
    fn log_delete(&self, table: &str, condition: &str);
    #[allow(dead_code)]
    fn log_query(&self, sql: &str, params: &str);
}

impl DatabaseOperationLogger for Connection {
    fn log_insert(&self, table: &str, data: &str) {
        tracing::info!("Database INSERT: table={}, data={}", table, data);
    }
    
    #[allow(dead_code)]
    fn log_update(&self, table: &str, condition: &str, data: &str) {
        tracing::info!("Database UPDATE: table={}, condition={}, data={}", table, condition, data);
    }
    
    #[allow(dead_code)]
    fn log_delete(&self, table: &str, condition: &str) {
        tracing::info!("Database DELETE: table={}, condition={}", table, condition);
    }
    
    #[allow(dead_code)]
    fn log_query(&self, sql: &str, params: &str) {
        tracing::debug!("Database QUERY: sql={}, params={}", sql, params);
    }
}

/// SQLiteタイムエントリリポジトリ実装
#[derive(Clone)]
pub struct SqliteTimeEntryRepository {
    db: Arc<Mutex<DatabaseConnection>>,
}

impl SqliteTimeEntryRepository {
    pub fn new(db: Arc<Mutex<DatabaseConnection>>) -> Self {
        Self { db }
    }

    fn format_datetime(dt: DateTime<Utc>) -> String {
        dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
    }

    fn parse_datetime(s: &str) -> anyhow::Result<DateTime<Utc>> {
        Ok(DateTime::parse_from_rfc3339(s)?.with_timezone(&Utc))
    }
}

#[async_trait]
impl TimeEntryRepository for SqliteTimeEntryRepository {
    async fn save_event(&self, event: &TimeEntryEvent) -> anyhow::Result<TimeEntryEvent> {
        tracing::info!("SqliteTimeEntryRepository::save_event: Starting to save event - task_id: {}, event_type: {:?}, at: {}", 
            event.task_id(), event.event_type(), event.at());
        
        let db = self.db.lock().await;
        tracing::debug!("SqliteTimeEntryRepository::save_event: Successfully acquired database lock");
        
        let conn = db.connection();
        tracing::debug!("SqliteTimeEntryRepository::save_event: Got database connection");

        let insert_sql = r#"
            INSERT INTO time_entry_events (task_id, event_type, at, start_event_id, payload)
            VALUES (?1, ?2, ?3, ?4, ?5)
            RETURNING id
        "#;
        
        let params_data = format!(
            "task_id={}, event_type={:?}, at={}, start_event_id={:?}, payload={:?}",
            i64::from(event.task_id()),
            event.event_type().as_str(),
            Self::format_datetime(event.at()),
            event.start_event_id(),
            event.payload()
        );
        
        tracing::debug!("SqliteTimeEntryRepository::save_event: Executing INSERT - sql: {}, params: {}", insert_sql, params_data);
        conn.log_insert("time_entry_events", &params_data);

        let id = match conn.query_row(
            insert_sql,
            params![
                i64::from(event.task_id()),
                event.event_type().as_str(),
                Self::format_datetime(event.at()),
                event.start_event_id(),
                event.payload(),
            ],
            |row| row.get::<_, i64>(0),
        ) {
            Ok(id) => {
                tracing::info!("SqliteTimeEntryRepository::save_event: Successfully inserted event with id: {}", id);
                id
            },
            Err(e) => {
                tracing::error!("SqliteTimeEntryRepository::save_event: Failed to insert event: {}", e);
                tracing::error!("SqliteTimeEntryRepository::save_event: SQL: {}", insert_sql);
                tracing::error!("SqliteTimeEntryRepository::save_event: Parameters: {}", params_data);
                return Err(e.into());
            }
        };

        let result = event.clone().with_id(id);
        tracing::info!("SqliteTimeEntryRepository::save_event: Successfully created TimeEntryEvent with id: {}", id);
        Ok(result)
    }

    async fn find_running_entries(&self) -> anyhow::Result<Vec<TimeEntry>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let mut stmt = conn.prepare(
            r#"
            SELECT task_id, start_event_id, start_time, end_time, duration_in_seconds
            FROM time_entries_view
            WHERE end_time IS NULL OR duration_in_seconds IS NULL
            ORDER BY start_time DESC
            "#,
        )?;

        let entry_iter = stmt.query_map([], |row| {
            let task_id: i64 = row.get(0)?;
            let start_event_id: i64 = row.get(1)?;
            let start_time_str: String = row.get(2)?;
            let end_time_str: Option<String> = row.get(3)?;
            let _duration: Option<i64> = row.get(4)?;

            Ok((task_id, start_event_id, start_time_str, end_time_str))
        })?;

        let mut entries = Vec::new();
        for entry_result in entry_iter {
            let (task_id, start_event_id, start_time_str, end_time_str) = entry_result?;
            let task_id = TaskId::new(task_id)?;
            let start_time = Self::parse_datetime(&start_time_str)?;
            let end_time = end_time_str
                .map(|s| Self::parse_datetime(&s))
                .transpose()?;

            let entry = TimeEntry::new(task_id, start_event_id, start_time, end_time);
            entries.push(entry);
        }

        Ok(entries)
    }

    async fn find_running_entry_by_task(&self, task_id: TaskId) -> anyhow::Result<Option<TimeEntry>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let result = conn.query_row(
            r#"
            SELECT start_event_id, start_time, end_time, duration_in_seconds
            FROM time_entries_view
            WHERE task_id = ?1 AND (end_time IS NULL OR duration_in_seconds IS NULL)
            ORDER BY start_time DESC
            LIMIT 1
            "#,
            params![i64::from(task_id)],
            |row| {
                let start_event_id: i64 = row.get(0)?;
                let start_time_str: String = row.get(1)?;
                let end_time_str: Option<String> = row.get(2)?;
                let _duration: Option<i64> = row.get(3)?;

                Ok((start_event_id, start_time_str, end_time_str))
            },
        );

        match result {
            Ok((start_event_id, start_time_str, end_time_str)) => {
                let start_time = Self::parse_datetime(&start_time_str)?;
                let end_time = end_time_str
                    .map(|s| Self::parse_datetime(&s))
                    .transpose()?;

                let entry = TimeEntry::new(task_id, start_event_id, start_time, end_time);
                Ok(Some(entry))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn find_entries_by_task(&self, task_id: TaskId) -> anyhow::Result<Vec<TimeEntry>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let mut stmt = conn.prepare(
            r#"
            SELECT start_event_id, start_time, end_time, duration_in_seconds
            FROM time_entries_view
            WHERE task_id = ?1
            ORDER BY start_time DESC
            "#,
        )?;

        let entry_iter = stmt.query_map(params![i64::from(task_id)], |row| {
            let start_event_id: i64 = row.get(0)?;
            let start_time_str: String = row.get(1)?;
            let end_time_str: Option<String> = row.get(2)?;
            let _duration: Option<i64> = row.get(3)?;

            Ok((start_event_id, start_time_str, end_time_str))
        })?;

        let mut entries = Vec::new();
        for entry_result in entry_iter {
            let (start_event_id, start_time_str, end_time_str) = entry_result?;
            let start_time = Self::parse_datetime(&start_time_str)?;
            let end_time = end_time_str
                .map(|s| Self::parse_datetime(&s))
                .transpose()?;

            let entry = TimeEntry::new(task_id, start_event_id, start_time, end_time);
            entries.push(entry);
        }

        Ok(entries)
    }

    async fn find_entries_by_task_and_period(
        &self,
        task_id: TaskId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<TimeEntry>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let mut stmt = conn.prepare(
            r#"
            SELECT start_event_id, start_time, end_time, duration_in_seconds
            FROM time_entries_view
            WHERE task_id = ?1 AND start_time >= ?2 AND start_time <= ?3
            ORDER BY start_time DESC
            "#,
        )?;

        let entry_iter = stmt.query_map(
            params![
                i64::from(task_id),
                Self::format_datetime(start),
                Self::format_datetime(end)
            ],
            |row| {
                let start_event_id: i64 = row.get(0)?;
                let start_time_str: String = row.get(1)?;
                let end_time_str: Option<String> = row.get(2)?;
                let _duration: Option<i64> = row.get(3)?;

                Ok((start_event_id, start_time_str, end_time_str))
            },
        )?;

        let mut entries = Vec::new();
        for entry_result in entry_iter {
            let (start_event_id, start_time_str, end_time_str) = entry_result?;
            let start_time = Self::parse_datetime(&start_time_str)?;
            let end_time = end_time_str
                .map(|s| Self::parse_datetime(&s))
                .transpose()?;

            let entry = TimeEntry::new(task_id, start_event_id, start_time, end_time);
            entries.push(entry);
        }

        Ok(entries)
    }

    async fn find_overlapping_entries(
        &self,
        task_id: TaskId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<TimeEntry>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let mut stmt = conn.prepare(
            r#"
            SELECT start_event_id, start_time, end_time, duration_in_seconds
            FROM time_entries_view
            WHERE task_id = ?1 
              AND start_time < ?3 
              AND (end_time IS NULL OR end_time > ?2)
            ORDER BY start_time DESC
            "#,
        )?;

        let entry_iter = stmt.query_map(
            params![
                i64::from(task_id),
                Self::format_datetime(start),
                Self::format_datetime(end)
            ],
            |row| {
                let start_event_id: i64 = row.get(0)?;
                let start_time_str: String = row.get(1)?;
                let end_time_str: Option<String> = row.get(2)?;
                let _duration: Option<i64> = row.get(3)?;

                Ok((start_event_id, start_time_str, end_time_str))
            },
        )?;

        let mut entries = Vec::new();
        for entry_result in entry_iter {
            let (start_event_id, start_time_str, end_time_str) = entry_result?;
            let start_time = Self::parse_datetime(&start_time_str)?;
            let end_time = end_time_str
                .map(|s| Self::parse_datetime(&s))
                .transpose()?;

            let entry = TimeEntry::new(task_id, start_event_id, start_time, end_time);
            entries.push(entry);
        }

        Ok(entries)
    }

    async fn find_entries_by_project(&self, project_id: i64) -> anyhow::Result<Vec<TimeEntry>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let mut stmt = conn.prepare(
            r#"
            SELECT tev.task_id, tev.start_event_id, tev.start_time, tev.end_time, tev.duration_in_seconds
            FROM time_entries_view tev
            JOIN task_current_view tcv ON tev.task_id = tcv.task_id
            WHERE tcv.project_id = ?1
            ORDER BY tev.start_time DESC
            "#,
        )?;

        let entry_iter = stmt.query_map(params![project_id], |row| {
            let task_id: i64 = row.get(0)?;
            let start_event_id: i64 = row.get(1)?;
            let start_time_str: String = row.get(2)?;
            let end_time_str: Option<String> = row.get(3)?;
            let _duration: Option<i64> = row.get(4)?;

            Ok((task_id, start_event_id, start_time_str, end_time_str))
        })?;

        let mut entries = Vec::new();
        for entry_result in entry_iter {
            let (task_id, start_event_id, start_time_str, end_time_str) = entry_result?;
            let task_id = TaskId::new(task_id)?;
            let start_time = Self::parse_datetime(&start_time_str)?;
            let end_time = end_time_str
                .map(|s| Self::parse_datetime(&s))
                .transpose()?;

            let entry = TimeEntry::new(task_id, start_event_id, start_time, end_time);
            entries.push(entry);
        }

        Ok(entries)
    }

    async fn find_entries_by_period(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<TimeEntry>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let mut stmt = conn.prepare(
            r#"
            SELECT task_id, start_event_id, start_time, end_time, duration_in_seconds
            FROM time_entries_view
            WHERE start_time >= ?1 AND start_time <= ?2
            ORDER BY start_time DESC
            "#,
        )?;

        let entry_iter = stmt.query_map(
            params![
                Self::format_datetime(start),
                Self::format_datetime(end)
            ],
            |row| {
                let task_id: i64 = row.get(0)?;
                let start_event_id: i64 = row.get(1)?;
                let start_time_str: String = row.get(2)?;
                let end_time_str: Option<String> = row.get(3)?;
                let _duration: Option<i64> = row.get(4)?;

                Ok((task_id, start_event_id, start_time_str, end_time_str))
            },
        )?;

        let mut entries = Vec::new();
        for entry_result in entry_iter {
            let (task_id, start_event_id, start_time_str, end_time_str) = entry_result?;
            let task_id = TaskId::new(task_id)?;
            let start_time = Self::parse_datetime(&start_time_str)?;
            let end_time = end_time_str
                .map(|s| Self::parse_datetime(&s))
                .transpose()?;

            let entry = TimeEntry::new(task_id, start_event_id, start_time, end_time);
            entries.push(entry);
        }

        Ok(entries)
    }

    async fn count_entries_by_task(&self, task_id: TaskId) -> anyhow::Result<usize> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM time_entries_view WHERE task_id = ?1",
            params![i64::from(task_id)],
            |row| row.get(0),
        )?;

        Ok(count as usize)
    }

    async fn sum_duration_by_task(&self, task_id: TaskId) -> anyhow::Result<i64> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let total: Option<i64> = conn.query_row(
            "SELECT SUM(duration_in_seconds) FROM time_entries_view WHERE task_id = ?1 AND duration_in_seconds IS NOT NULL",
            params![i64::from(task_id)],
            |row| row.get(0),
        )?;

        Ok(total.unwrap_or(0))
    }

    async fn sum_duration_by_project(&self, project_id: i64) -> anyhow::Result<i64> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let total: Option<i64> = conn.query_row(
            r#"
            SELECT SUM(tev.duration_in_seconds)
            FROM time_entries_view tev
            JOIN task_current_view tcv ON tev.task_id = tcv.task_id
            WHERE tcv.project_id = ?1 AND tev.duration_in_seconds IS NOT NULL
            "#,
            params![project_id],
            |row| row.get(0),
        )?;

        Ok(total.unwrap_or(0))
    }

    async fn find_entry_by_start_event_id(&self, start_event_id: i64) -> anyhow::Result<Option<TimeEntry>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let result = conn.query_row(
            r#"
            SELECT task_id, start_time, end_time, duration_in_seconds
            FROM time_entries_view
            WHERE start_event_id = ?1
            "#,
            params![start_event_id],
            |row| {
                let task_id: i64 = row.get(0)?;
                let start_time_str: String = row.get(1)?;
                let end_time_str: Option<String> = row.get(2)?;
                let _duration: Option<i64> = row.get(3)?;

                Ok((task_id, start_time_str, end_time_str))
            },
        );

        match result {
            Ok((task_id, start_time_str, end_time_str)) => {
                let task_id = TaskId::new(task_id)?;
                let start_time = Self::parse_datetime(&start_time_str)?;
                let end_time = end_time_str
                    .map(|s| Self::parse_datetime(&s))
                    .transpose()?;

                let entry = TimeEntry::new(task_id, start_event_id, start_time, end_time);
                Ok(Some(entry))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn find_recent_entries(&self, limit: usize) -> anyhow::Result<Vec<TimeEntry>> {
        tracing::info!("SqliteTimeEntryRepository::find_recent_entries: Starting with limit: {}", limit);
        
        let db = self.db.lock().await;
        tracing::debug!("SqliteTimeEntryRepository::find_recent_entries: Successfully acquired database lock");
        
        let conn = db.connection();
        tracing::debug!("SqliteTimeEntryRepository::find_recent_entries: Got database connection");

        tracing::debug!("SqliteTimeEntryRepository::find_recent_entries: Preparing SQL statement");
        let mut stmt = match conn.prepare(
            r#"
            SELECT task_id, start_event_id, start_time, end_time, duration_in_seconds
            FROM time_entries_view
            ORDER BY start_time DESC
            LIMIT ?1
            "#,
        ) {
            Ok(stmt) => {
                tracing::debug!("SqliteTimeEntryRepository::find_recent_entries: SQL statement prepared successfully");
                stmt
            },
            Err(e) => {
                tracing::error!("SqliteTimeEntryRepository::find_recent_entries: Failed to prepare SQL statement: {}", e);
                tracing::error!("SqliteTimeEntryRepository::find_recent_entries: SQL: SELECT task_id, start_event_id, start_time, end_time, duration_in_seconds FROM time_entries_view ORDER BY start_time DESC LIMIT ?1");
                return Err(e.into());
            }
        };

        tracing::debug!("SqliteTimeEntryRepository::find_recent_entries: Executing query with limit: {}", limit);
        let entry_iter = match stmt.query_map(params![limit], |row| {
            let task_id: i64 = row.get(0)?;
            let start_event_id: i64 = row.get(1)?;
            let start_time_str: String = row.get(2)?;
            let end_time_str: Option<String> = row.get(3)?;
            let _duration: Option<i64> = row.get(4)?;

            Ok((task_id, start_event_id, start_time_str, end_time_str))
        }) {
            Ok(iter) => {
                tracing::debug!("SqliteTimeEntryRepository::find_recent_entries: Query executed successfully");
                iter
            },
            Err(e) => {
                tracing::error!("SqliteTimeEntryRepository::find_recent_entries: Failed to execute query: {}", e);
                tracing::error!("SqliteTimeEntryRepository::find_recent_entries: Query parameters: limit={}", limit);
                return Err(e.into());
            }
        };

        let mut entries = Vec::new();
        let mut row_count = 0;
        
        tracing::debug!("SqliteTimeEntryRepository::find_recent_entries: Starting to process query results");
        
        for entry_result in entry_iter {
            row_count += 1;
            tracing::debug!("SqliteTimeEntryRepository::find_recent_entries: Processing row {}", row_count);
            
            match entry_result {
                Ok((task_id, start_event_id, start_time_str, end_time_str)) => {
                    tracing::debug!("SqliteTimeEntryRepository::find_recent_entries: Row {} raw data - task_id: {}, start_event_id: {}, start_time: {:?}, end_time: {:?}", 
                        row_count, task_id, start_event_id, start_time_str, end_time_str);
                    
                    match TaskId::new(task_id) {
                        Ok(task_id) => {
                            tracing::debug!("SqliteTimeEntryRepository::find_recent_entries: Row {} - TaskId created successfully: {}", row_count, task_id);
                            
                            match Self::parse_datetime(&start_time_str) {
                                Ok(start_time) => {
                                    tracing::debug!("SqliteTimeEntryRepository::find_recent_entries: Row {} - start_time parsed successfully: {}", row_count, start_time);
                                    
                                    let end_time = match &end_time_str {
                                        Some(s) => {
                                            match Self::parse_datetime(s) {
                                                Ok(et) => {
                                                    tracing::debug!("SqliteTimeEntryRepository::find_recent_entries: Row {} - end_time parsed successfully: {}", row_count, et);
                                                    Ok(Some(et))
                                                },
                                                Err(e) => {
                                                    tracing::error!("SqliteTimeEntryRepository::find_recent_entries: Row {} - failed to parse end_time '{:?}': {}", row_count, s, e);
                                                    Err(e)
                                                }
                                            }
                                        },
                                        None => {
                                            tracing::debug!("SqliteTimeEntryRepository::find_recent_entries: Row {} - no end_time (running entry)", row_count);
                                            Ok(None)
                                        }
                                    };
                                    
                                    match end_time {
                                        Ok(end_time) => {
                                            let entry = TimeEntry::new(task_id, start_event_id, start_time, end_time);
                                            entries.push(entry);
                                            tracing::debug!("SqliteTimeEntryRepository::find_recent_entries: Row {} - TimeEntry created successfully", row_count);
                                        },
                                        Err(e) => {
                                            tracing::error!("SqliteTimeEntryRepository::find_recent_entries: Row {} - failed to process end_time: {}", row_count, e);
                                            return Err(e);
                                        }
                                    }
                                },
                                Err(e) => {
                                    tracing::error!("SqliteTimeEntryRepository::find_recent_entries: Row {} - failed to parse start_time '{:?}': {}", row_count, start_time_str, e);
                                    return Err(e);
                                }
                            }
                        },
                        Err(e) => {
                            tracing::error!("SqliteTimeEntryRepository::find_recent_entries: Row {} - failed to create TaskId from {}: {}", row_count, task_id, e);
                            return Err(e.into());
                        }
                    }
                },
                Err(e) => {
                    tracing::error!("SqliteTimeEntryRepository::find_recent_entries: Row {} - failed to read row: {}", row_count, e);
                    return Err(e.into());
                }
            }
        }

        tracing::info!("SqliteTimeEntryRepository::find_recent_entries: Processing completed - rows processed: {}, entries created: {}", 
            row_count, entries.len());
        tracing::debug!("SqliteTimeEntryRepository::find_recent_entries: Successfully returning {} entries", entries.len());
        Ok(entries)
    }

    async fn find_recent_entries_by_task(&self, task_id: TaskId, limit: usize) -> anyhow::Result<Vec<TimeEntry>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let mut stmt = conn.prepare(
            r#"
            SELECT start_event_id, start_time, end_time, duration_in_seconds
            FROM time_entries_view
            WHERE task_id = ?1
            ORDER BY start_time DESC
            LIMIT ?2
            "#,
        )?;

        let entry_iter = stmt.query_map(params![i64::from(task_id), limit], |row| {
            let start_event_id: i64 = row.get(0)?;
            let start_time_str: String = row.get(1)?;
            let end_time_str: Option<String> = row.get(2)?;
            let _duration: Option<i64> = row.get(3)?;

            Ok((start_event_id, start_time_str, end_time_str))
        })?;

        let mut entries = Vec::new();
        for entry_result in entry_iter {
            let (start_event_id, start_time_str, end_time_str) = entry_result?;
            let start_time = Self::parse_datetime(&start_time_str)?;
            let end_time = end_time_str
                .map(|s| Self::parse_datetime(&s))
                .transpose()?;

            let entry = TimeEntry::new(task_id, start_event_id, start_time, end_time);
            entries.push(entry);
        }

        Ok(entries)
    }
}

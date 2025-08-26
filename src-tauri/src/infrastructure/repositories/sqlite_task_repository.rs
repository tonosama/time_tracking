use crate::domain::entities::Task;
use crate::domain::repositories::TaskRepository;
use crate::domain::value_objects::{ProjectId, TaskId, Status};
use crate::infrastructure::database::DatabaseConnection;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::params;
use std::sync::Arc;
use tokio::sync::Mutex;

/// SQLiteタスクリポジトリ実装
#[derive(Clone)]
pub struct SqliteTaskRepository {
    db: Arc<Mutex<DatabaseConnection>>,
}

impl SqliteTaskRepository {
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
impl TaskRepository for SqliteTaskRepository {
    async fn save(&self, task: &Task) -> anyhow::Result<()> {
        let db = self.db.lock().await;
        let conn = db.connection();

        eprintln!("[REPO] Saving task: {:?}", task);
        println!("[REPO] Saving task: {:?}", task);

        // タスク識別子を挿入（存在しない場合）
        eprintln!("[REPO] Inserting task identifier");
        println!("[REPO] Inserting task identifier");
        match conn.execute(
            "INSERT OR IGNORE INTO tasks (id) VALUES (?1)",
            params![i64::from(task.id())],
        ) {
            Ok(_) => {
                eprintln!("[REPO] Task identifier inserted/ignored successfully");
                println!("[REPO] Task identifier inserted/ignored successfully");
            },
            Err(e) => {
                eprintln!("[REPO] Failed to insert task identifier: {}", e);
                println!("[REPO] Failed to insert task identifier: {}", e);
                return Err(e.into());
            }
        }

        // 次のバージョン番号を取得
        eprintln!("[REPO] Getting next version number");
        println!("[REPO] Getting next version number");
        let next_version: i64 = match conn.query_row(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM task_versions WHERE task_id = ?1",
            params![i64::from(task.id())],
            |row| row.get(0),
        ) {
            Ok(version) => {
                eprintln!("[REPO] Next version number: {}", version);
                println!("[REPO] Next version number: {}", version);
                version
            },
            Err(e) => {
                eprintln!("[REPO] Failed to get next version number: {}", e);
                println!("[REPO] Failed to get next version number: {}", e);
                return Err(e.into());
            }
        };

        // タスクバージョンを挿入
        eprintln!("[REPO] Inserting task version");
        println!("[REPO] Inserting task version");
        match conn.execute(
            r#"
            INSERT INTO task_versions (task_id, version, project_id, name, status, effective_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                i64::from(task.id()),
                next_version,
                i64::from(task.project_id()),
                task.name(),
                task.status().as_str(),
                Self::format_datetime(task.effective_at()),
            ],
        ) {
            Ok(_) => {
                eprintln!("[REPO] Task version inserted successfully");
                println!("[REPO] Task version inserted successfully");
                Ok(())
            },
            Err(e) => {
                eprintln!("[REPO] Failed to insert task version: {}", e);
                println!("[REPO] Failed to insert task version: {}", e);
                Err(e.into())
            }
        }
    }

    async fn find_by_id(&self, id: TaskId) -> anyhow::Result<Option<Task>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let result = conn.query_row(
            "SELECT task_id, project_id, name, status, effective_at FROM task_current_view WHERE task_id = ?1",
            params![i64::from(id)],
            |row| {
                let task_id: i64 = row.get(0)?;
                let project_id: i64 = row.get(1)?;
                let name: String = row.get(2)?;
                let status_str: String = row.get(3)?;
                let effective_at_str: String = row.get(4)?;

                Ok((task_id, project_id, name, status_str, effective_at_str))
            },
        );

        match result {
            Ok((task_id, project_id, name, status_str, effective_at_str)) => {
                let task_id = TaskId::new(task_id)?;
                let project_id = ProjectId::new(project_id)?;
                let status = Status::from_str(&status_str)?;
                let effective_at = Self::parse_datetime(&effective_at_str)?;

                let mut task = Task::new_with_time(task_id, project_id, name, effective_at)?;
                if status.is_archived() {
                    task = task.archive();
                }
                Ok(Some(task))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn find_by_project_id(&self, project_id: ProjectId) -> anyhow::Result<Vec<Task>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let mut stmt = conn.prepare(
            "SELECT task_id, project_id, name, status, effective_at FROM task_current_view WHERE project_id = ?1"
        )?;

        let task_iter = stmt.query_map(params![i64::from(project_id)], |row| {
            let task_id: i64 = row.get(0)?;
            let project_id: i64 = row.get(1)?;
            let name: String = row.get(2)?;
            let status_str: String = row.get(3)?;
            let effective_at_str: String = row.get(4)?;

            Ok((task_id, project_id, name, status_str, effective_at_str))
        })?;

        let mut tasks = Vec::new();
        for task_result in task_iter {
            let (task_id, project_id, name, status_str, effective_at_str) = task_result?;
            let task_id = TaskId::new(task_id)?;
            let project_id = ProjectId::new(project_id)?;
            let status = Status::from_str(&status_str)?;
            let effective_at = Self::parse_datetime(&effective_at_str)?;
            
            let mut task = Task::new_with_time(task_id, project_id, name, effective_at)?;
            if status.is_archived() {
                task = task.archive();
            }
            tasks.push(task);
        }

        Ok(tasks)
    }

    async fn find_active_by_project_id(&self, project_id: ProjectId) -> anyhow::Result<Vec<Task>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let mut stmt = conn.prepare(
            "SELECT task_id, project_id, name, status, effective_at FROM task_current_view WHERE project_id = ?1 AND status = 'active'"
        )?;

        let task_iter = stmt.query_map(params![i64::from(project_id)], |row| {
            let task_id: i64 = row.get(0)?;
            let project_id: i64 = row.get(1)?;
            let name: String = row.get(2)?;
            let status_str: String = row.get(3)?;
            let effective_at_str: String = row.get(4)?;

            Ok((task_id, project_id, name, status_str, effective_at_str))
        })?;

        let mut tasks = Vec::new();
        for task_result in task_iter {
            let (task_id, project_id, name, _status_str, effective_at_str) = task_result?;
            let task_id = TaskId::new(task_id)?;
            let project_id = ProjectId::new(project_id)?;
            let effective_at = Self::parse_datetime(&effective_at_str)?;
            
            let task = Task::new_with_time(task_id, project_id, name, effective_at)?;
            tasks.push(task);
        }

        Ok(tasks)
    }

    async fn find_all_active(&self) -> anyhow::Result<Vec<Task>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let mut stmt = conn.prepare(
            "SELECT task_id, project_id, name, status, effective_at FROM task_current_view WHERE status = 'active'"
        )?;

        let task_iter = stmt.query_map([], |row| {
            let task_id: i64 = row.get(0)?;
            let project_id: i64 = row.get(1)?;
            let name: String = row.get(2)?;
            let status_str: String = row.get(3)?;
            let effective_at_str: String = row.get(4)?;

            Ok((task_id, project_id, name, status_str, effective_at_str))
        })?;

        let mut tasks = Vec::new();
        for task_result in task_iter {
            let (task_id, project_id, name, _status_str, effective_at_str) = task_result?;
            let task_id = TaskId::new(task_id)?;
            let project_id = ProjectId::new(project_id)?;
            let effective_at = Self::parse_datetime(&effective_at_str)?;
            
            let task = Task::new_with_time(task_id, project_id, name, effective_at)?;
            tasks.push(task);
        }

        Ok(tasks)
    }

    async fn find_all(&self) -> anyhow::Result<Vec<Task>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let mut stmt = conn.prepare(
            "SELECT task_id, project_id, name, status, effective_at FROM task_current_view"
        )?;

        let task_iter = stmt.query_map([], |row| {
            let task_id: i64 = row.get(0)?;
            let project_id: i64 = row.get(1)?;
            let name: String = row.get(2)?;
            let status_str: String = row.get(3)?;
            let effective_at_str: String = row.get(4)?;

            Ok((task_id, project_id, name, status_str, effective_at_str))
        })?;

        let mut tasks = Vec::new();
        for task_result in task_iter {
            let (task_id, project_id, name, status_str, effective_at_str) = task_result?;
            let task_id = TaskId::new(task_id)?;
            let project_id = ProjectId::new(project_id)?;
            let status = Status::from_str(&status_str)?;
            let effective_at = Self::parse_datetime(&effective_at_str)?;
            
            let mut task = Task::new_with_time(task_id, project_id, name, effective_at)?;
            if status.is_archived() {
                task = task.archive();
            }
            tasks.push(task);
        }

        Ok(tasks)
    }

    async fn find_by_status(&self, status: &Status) -> anyhow::Result<Vec<Task>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let mut stmt = conn.prepare(
            "SELECT task_id, project_id, name, status, effective_at FROM task_current_view WHERE status = ?1"
        )?;

        let task_iter = stmt.query_map(params![status.as_str()], |row| {
            let task_id: i64 = row.get(0)?;
            let project_id: i64 = row.get(1)?;
            let name: String = row.get(2)?;
            let status_str: String = row.get(3)?;
            let effective_at_str: String = row.get(4)?;

            Ok((task_id, project_id, name, status_str, effective_at_str))
        })?;

        let mut tasks = Vec::new();
        for task_result in task_iter {
            let (task_id, project_id, name, status_str, effective_at_str) = task_result?;
            let task_id = TaskId::new(task_id)?;
            let project_id = ProjectId::new(project_id)?;
            let status = Status::from_str(&status_str)?;
            let effective_at = Self::parse_datetime(&effective_at_str)?;
            
            let mut task = Task::new_with_time(task_id, project_id, name, effective_at)?;
            if status.is_archived() {
                task = task.archive();
            }
            tasks.push(task);
        }

        Ok(tasks)
    }

    async fn find_by_name_prefix(&self, prefix: &str) -> anyhow::Result<Vec<Task>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let mut stmt = conn.prepare(
            "SELECT task_id, project_id, name, status, effective_at FROM task_current_view WHERE name LIKE ?1"
        )?;

        let like_pattern = format!("{}%", prefix);
        let task_iter = stmt.query_map(params![like_pattern], |row| {
            let task_id: i64 = row.get(0)?;
            let project_id: i64 = row.get(1)?;
            let name: String = row.get(2)?;
            let status_str: String = row.get(3)?;
            let effective_at_str: String = row.get(4)?;

            Ok((task_id, project_id, name, status_str, effective_at_str))
        })?;

        let mut tasks = Vec::new();
        for task_result in task_iter {
            let (task_id, project_id, name, status_str, effective_at_str) = task_result?;
            let task_id = TaskId::new(task_id)?;
            let project_id = ProjectId::new(project_id)?;
            let status = Status::from_str(&status_str)?;
            let effective_at = Self::parse_datetime(&effective_at_str)?;
            
            let mut task = Task::new_with_time(task_id, project_id, name, effective_at)?;
            if status.is_archived() {
                task = task.archive();
            }
            tasks.push(task);
        }

        Ok(tasks)
    }

    async fn next_id(&self) -> anyhow::Result<TaskId> {
        let db = self.db.lock().await;
        let conn = db.connection();

        eprintln!("[REPO] Generating next task ID");
        println!("[REPO] Generating next task ID");
        
        let next_id: i64 = match conn.query_row(
            "SELECT COALESCE(MAX(id), 0) + 1 FROM tasks",
            [],
            |row| row.get(0),
        ) {
            Ok(id) => {
                eprintln!("[REPO] Next task ID generated: {}", id);
                println!("[REPO] Next task ID generated: {}", id);
                id
            },
            Err(e) => {
                eprintln!("[REPO] Failed to generate next task ID: {}", e);
                println!("[REPO] Failed to generate next task ID: {}", e);
                return Err(e.into());
            }
        };

        match TaskId::new(next_id) {
            Ok(task_id) => {
                eprintln!("[REPO] TaskId created successfully: {:?}", task_id);
                println!("[REPO] TaskId created successfully: {:?}", task_id);
                Ok(task_id)
            },
            Err(e) => {
                eprintln!("[REPO] Failed to create TaskId: {}", e);
                println!("[REPO] Failed to create TaskId: {}", e);
                Err(e)
            }
        }
    }

    async fn exists(&self, id: TaskId) -> anyhow::Result<bool> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE id = ?1",
            params![i64::from(id)],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    async fn find_history(&self, id: TaskId) -> anyhow::Result<Vec<Task>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let mut stmt = conn.prepare(
            "SELECT project_id, name, status, effective_at FROM task_versions WHERE task_id = ?1 ORDER BY effective_at"
        )?;

        let task_iter = stmt.query_map(params![i64::from(id)], |row| {
            let project_id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let status_str: String = row.get(2)?;
            let effective_at_str: String = row.get(3)?;

            Ok((project_id, name, status_str, effective_at_str))
        })?;

        let mut tasks = Vec::new();
        for task_result in task_iter {
            let (project_id, name, status_str, effective_at_str) = task_result?;
            let project_id = ProjectId::new(project_id)?;
            let status = Status::from_str(&status_str)?;
            let effective_at = Self::parse_datetime(&effective_at_str)?;
            
            let mut task = Task::new_with_time(id, project_id, name, effective_at)?;
            if status.is_archived() {
                task = task.archive();
            }
            tasks.push(task);
        }

        Ok(tasks)
    }

    async fn find_at_time(&self, id: TaskId, at: DateTime<Utc>) -> anyhow::Result<Option<Task>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let result = conn.query_row(
            r#"
            SELECT project_id, name, status, effective_at
            FROM task_versions
            WHERE task_id = ?1 AND effective_at <= ?2
            ORDER BY effective_at DESC, version DESC
            LIMIT 1
            "#,
            params![i64::from(id), Self::format_datetime(at)],
            |row| {
                let project_id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                let status_str: String = row.get(2)?;
                let effective_at_str: String = row.get(3)?;

                Ok((project_id, name, status_str, effective_at_str))
            },
        );

        match result {
            Ok((project_id, name, status_str, effective_at_str)) => {
                let project_id = ProjectId::new(project_id)?;
                let status = Status::from_str(&status_str)?;
                let effective_at = Self::parse_datetime(&effective_at_str)?;
                
                let mut task = Task::new_with_time(id, project_id, name, effective_at)?;
                if status.is_archived() {
                    task = task.archive();
                }
                Ok(Some(task))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn count_by_project_id(&self, project_id: ProjectId) -> anyhow::Result<usize> {
        let tasks = self.find_by_project_id(project_id).await?;
        Ok(tasks.len())
    }

    async fn find_by_project_id_ordered(&self, project_id: ProjectId) -> anyhow::Result<Vec<Task>> {
        let mut tasks = self.find_by_project_id(project_id).await?;
        tasks.sort_by(|a, b| a.name().cmp(b.name()));
        Ok(tasks)
    }
}

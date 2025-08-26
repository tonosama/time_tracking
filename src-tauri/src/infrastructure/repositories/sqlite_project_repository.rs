use crate::domain::entities::Project;
use crate::domain::repositories::ProjectRepository;
use crate::domain::value_objects::{ProjectId, Status};
use crate::infrastructure::database::DatabaseConnection;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::params;
use std::sync::Arc;
use tokio::sync::Mutex;

/// SQLiteプロジェクトリポジトリ実装
#[derive(Clone)]
pub struct SqliteProjectRepository {
    db: Arc<Mutex<DatabaseConnection>>,
}

impl SqliteProjectRepository {
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
impl ProjectRepository for SqliteProjectRepository {
    async fn save(&self, project: &Project) -> anyhow::Result<()> {
        let db = self.db.lock().await;
        let conn = db.connection();

        // プロジェクト識別子を挿入（存在しない場合）
        conn.execute(
            "INSERT OR IGNORE INTO projects (id) VALUES (?1)",
            params![i64::from(project.id())],
        )?;

        // 次のバージョン番号を取得
        let next_version: i64 = conn.query_row(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM project_versions WHERE project_id = ?1",
            params![i64::from(project.id())],
            |row| row.get(0),
        )?;

        // プロジェクトバージョンを挿入
        conn.execute(
            r#"
            INSERT INTO project_versions (project_id, version, name, status, effective_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                i64::from(project.id()),
                next_version,
                project.name(),
                project.status().as_str(),
                Self::format_datetime(project.effective_at()),
            ],
        )?;

        Ok(())
    }

    async fn find_by_id(&self, id: ProjectId) -> anyhow::Result<Option<Project>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let result = conn.query_row(
            "SELECT project_id, name, status, effective_at FROM project_current_view WHERE project_id = ?1",
            params![i64::from(id)],
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

                let mut project = Project::new_with_time(project_id, name, effective_at)?;
                if status.is_archived() {
                    project = project.archive();
                }
                Ok(Some(project))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn find_all_active(&self) -> anyhow::Result<Vec<Project>> {
        println!("[REPO] find_all_active called");
        eprintln!("[REPO] find_all_active called");
        
        let db = self.db.lock().await;
        let conn = db.connection();

        println!("[REPO] Executing SQL: SELECT project_id, name, status, effective_at FROM project_current_view WHERE status = 'active'");
        eprintln!("[REPO] Executing SQL: SELECT project_id, name, status, effective_at FROM project_current_view WHERE status = 'active'");

        let mut stmt = conn.prepare(
            "SELECT project_id, name, status, effective_at FROM project_current_view WHERE status = 'active'"
        )?;

        let project_iter = stmt.query_map([], |row| {
            let project_id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let status_str: String = row.get(2)?;
            let effective_at_str: String = row.get(3)?;

            Ok((project_id, name, status_str, effective_at_str))
        })?;

        let mut projects = Vec::new();
        for project_result in project_iter {
            let (project_id, name, _status_str, effective_at_str) = project_result?;
            let project_id = ProjectId::new(project_id)?;
            let effective_at = Self::parse_datetime(&effective_at_str)?;
            let project = Project::new_with_time(project_id, name, effective_at)?;
            projects.push(project);
        }

        println!("[REPO] Found {} active projects", projects.len());
        eprintln!("[REPO] Found {} active projects", projects.len());
        
        Ok(projects)
    }

    async fn find_all(&self) -> anyhow::Result<Vec<Project>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let mut stmt = conn.prepare(
            "SELECT project_id, name, status, effective_at FROM project_current_view"
        )?;

        let project_iter = stmt.query_map([], |row| {
            let project_id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let status_str: String = row.get(2)?;
            let effective_at_str: String = row.get(3)?;

            Ok((project_id, name, status_str, effective_at_str))
        })?;

        let mut projects = Vec::new();
        for project_result in project_iter {
            let (project_id, name, status_str, effective_at_str) = project_result?;
            let project_id = ProjectId::new(project_id)?;
            let status = Status::from_str(&status_str)?;
            let effective_at = Self::parse_datetime(&effective_at_str)?;
            
            let mut project = Project::new_with_time(project_id, name, effective_at)?;
            if status.is_archived() {
                project = project.archive();
            }
            projects.push(project);
        }

        Ok(projects)
    }

    async fn find_by_status(&self, status: &Status) -> anyhow::Result<Vec<Project>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let mut stmt = conn.prepare(
            "SELECT project_id, name, status, effective_at FROM project_current_view WHERE status = ?1"
        )?;

        let project_iter = stmt.query_map(params![status.as_str()], |row| {
            let project_id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let status_str: String = row.get(2)?;
            let effective_at_str: String = row.get(3)?;

            Ok((project_id, name, status_str, effective_at_str))
        })?;

        let mut projects = Vec::new();
        for project_result in project_iter {
            let (project_id, name, status_str, effective_at_str) = project_result?;
            let project_id = ProjectId::new(project_id)?;
            let status = Status::from_str(&status_str)?;
            let effective_at = Self::parse_datetime(&effective_at_str)?;
            
            let mut project = Project::new_with_time(project_id, name, effective_at)?;
            if status.is_archived() {
                project = project.archive();
            }
            projects.push(project);
        }

        Ok(projects)
    }

    async fn find_by_name_prefix(&self, prefix: &str) -> anyhow::Result<Vec<Project>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let mut stmt = conn.prepare(
            "SELECT project_id, name, status, effective_at FROM project_current_view WHERE name LIKE ?1"
        )?;

        let like_pattern = format!("{}%", prefix);
        let project_iter = stmt.query_map(params![like_pattern], |row| {
            let project_id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let status_str: String = row.get(2)?;
            let effective_at_str: String = row.get(3)?;

            Ok((project_id, name, status_str, effective_at_str))
        })?;

        let mut projects = Vec::new();
        for project_result in project_iter {
            let (project_id, name, status_str, effective_at_str) = project_result?;
            let project_id = ProjectId::new(project_id)?;
            let status = Status::from_str(&status_str)?;
            let effective_at = Self::parse_datetime(&effective_at_str)?;
            
            let mut project = Project::new_with_time(project_id, name, effective_at)?;
            if status.is_archived() {
                project = project.archive();
            }
            projects.push(project);
        }

        Ok(projects)
    }

    async fn next_id(&self) -> anyhow::Result<ProjectId> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let next_id: i64 = conn.query_row(
            "SELECT COALESCE(MAX(id), 0) + 1 FROM projects",
            [],
            |row| row.get(0),
        )?;

        ProjectId::new(next_id)
    }

    async fn exists(&self, id: ProjectId) -> anyhow::Result<bool> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM projects WHERE id = ?1",
            params![i64::from(id)],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    async fn find_history(&self, id: ProjectId) -> anyhow::Result<Vec<Project>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let mut stmt = conn.prepare(
            "SELECT name, status, effective_at FROM project_versions WHERE project_id = ?1 ORDER BY effective_at"
        )?;

        let project_iter = stmt.query_map(params![i64::from(id)], |row| {
            let name: String = row.get(0)?;
            let status_str: String = row.get(1)?;
            let effective_at_str: String = row.get(2)?;

            Ok((name, status_str, effective_at_str))
        })?;

        let mut projects = Vec::new();
        for project_result in project_iter {
            let (name, status_str, effective_at_str) = project_result?;
            let status = Status::from_str(&status_str)?;
            let effective_at = Self::parse_datetime(&effective_at_str)?;
            
            let mut project = Project::new_with_time(id, name, effective_at)?;
            if status.is_archived() {
                project = project.archive();
            }
            projects.push(project);
        }

        Ok(projects)
    }

    async fn find_at_time(&self, id: ProjectId, at: DateTime<Utc>) -> anyhow::Result<Option<Project>> {
        let db = self.db.lock().await;
        let conn = db.connection();

        let result = conn.query_row(
            r#"
            SELECT name, status, effective_at
            FROM project_versions
            WHERE project_id = ?1 AND effective_at <= ?2
            ORDER BY effective_at DESC, version DESC
            LIMIT 1
            "#,
            params![i64::from(id), Self::format_datetime(at)],
            |row| {
                let name: String = row.get(0)?;
                let status_str: String = row.get(1)?;
                let effective_at_str: String = row.get(2)?;

                Ok((name, status_str, effective_at_str))
            },
        );

        match result {
            Ok((name, status_str, effective_at_str)) => {
                let status = Status::from_str(&status_str)?;
                let effective_at = Self::parse_datetime(&effective_at_str)?;
                
                let mut project = Project::new_with_time(id, name, effective_at)?;
                if status.is_archived() {
                    project = project.archive();
                }
                Ok(Some(project))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

/// データベース接続管理
pub struct DatabaseConnection {
    connection: Connection,
}

impl DatabaseConnection {
    /// ファイルベースのデータベースに接続
    pub fn new<P: AsRef<Path>>(database_path: P) -> Result<Self> {
        let conn = Connection::open(database_path)?;
        
        // WALモードとforeign_keysを有効化
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        Ok(Self { connection: conn })
    }

    /// インメモリデータベースに接続（テスト用）
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        
        // foreign_keysを有効化
        conn.pragma_update(None, "foreign_keys", "ON")?;

        Ok(Self { connection: conn })
    }

    /// コネクションの参照を取得
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// マイグレーションを実行
    pub fn run_migrations(&self) -> Result<()> {
        self.create_schema()?;
        self.create_views()?;
        Ok(())
    }

    /// 基本スキーマを作成
    fn create_schema(&self) -> Result<()> {
        self.connection.execute_batch(r#"
            -- スキーマバージョン管理
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );

            -- プロジェクト識別子テーブル
            CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY
            );

            -- プロジェクトバージョンテーブル
            CREATE TABLE IF NOT EXISTS project_versions (
                id INTEGER PRIMARY KEY,
                project_id INTEGER NOT NULL,
                version INTEGER NOT NULL,
                name TEXT NOT NULL,
                status TEXT NOT NULL CHECK(status IN ('active','archived')),
                effective_at TEXT NOT NULL,
                FOREIGN KEY(project_id) REFERENCES projects(id),
                UNIQUE(project_id, version)
            );

            -- タスク識別子テーブル
            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY
            );

            -- タスクバージョンテーブル
            CREATE TABLE IF NOT EXISTS task_versions (
                id INTEGER PRIMARY KEY,
                task_id INTEGER NOT NULL,
                version INTEGER NOT NULL,
                project_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                status TEXT NOT NULL CHECK(status IN ('active','archived')),
                effective_at TEXT NOT NULL,
                FOREIGN KEY(task_id) REFERENCES tasks(id),
                FOREIGN KEY(project_id) REFERENCES projects(id),
                UNIQUE(task_id, version)
            );

            -- インデックス
            CREATE INDEX IF NOT EXISTS idx_project_versions_project_version 
                ON project_versions(project_id, version DESC);
            CREATE INDEX IF NOT EXISTS idx_task_versions_task_version 
                ON task_versions(task_id, version DESC);
        "#)?;

        Ok(())
    }

    /// ビューを作成
    fn create_views(&self) -> Result<()> {
        self.connection.execute_batch(r#"
            -- プロジェクト現在値ビュー
            CREATE VIEW IF NOT EXISTS project_current_view AS
            WITH latest AS (
                SELECT pv.project_id, MAX(pv.effective_at) AS max_effective_at
                FROM project_versions pv
                GROUP BY pv.project_id
            ), latest_tie_break AS (
                SELECT pv.project_id, MAX(pv.version) AS max_version
                FROM project_versions pv
                JOIN latest l ON l.project_id = pv.project_id AND l.max_effective_at = pv.effective_at
                GROUP BY pv.project_id
            )
            SELECT pv.project_id, pv.name, pv.status, pv.effective_at
            FROM project_versions pv
            JOIN latest l ON l.project_id = pv.project_id AND l.max_effective_at = pv.effective_at
            JOIN latest_tie_break lb ON lb.project_id = pv.project_id AND lb.max_version = pv.version;

            -- タスク現在値ビュー
            CREATE VIEW IF NOT EXISTS task_current_view AS
            WITH latest AS (
                SELECT tv.task_id, MAX(tv.effective_at) AS max_effective_at
                FROM task_versions tv
                GROUP BY tv.task_id
            ), latest_tie_break AS (
                SELECT tv.task_id, MAX(tv.version) AS max_version
                FROM task_versions tv
                JOIN latest l ON l.task_id = tv.task_id AND l.max_effective_at = tv.effective_at
                GROUP BY tv.task_id
            )
            SELECT tv.task_id, tv.project_id, tv.name, tv.status, tv.effective_at
            FROM task_versions tv
            JOIN latest l ON l.task_id = tv.task_id AND l.max_effective_at = tv.effective_at
            JOIN latest_tie_break lb ON lb.task_id = tv.task_id AND lb.max_version = tv.version;
        "#)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_connection_in_memory() {
        let db = DatabaseConnection::new_in_memory().unwrap();
        db.run_migrations().unwrap();

        // テーブルが作成されていることを確認
        let tables: Vec<String> = db.connection()
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")?
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"projects".to_string()));
        assert!(tables.contains(&"project_versions".to_string()));
        assert!(tables.contains(&"tasks".to_string()));
        assert!(tables.contains(&"task_versions".to_string()));
    }

    #[test]
    fn test_database_views_creation() {
        let db = DatabaseConnection::new_in_memory().unwrap();
        db.run_migrations().unwrap();

        // ビューが作成されていることを確認
        let views: Vec<String> = db.connection()
            .prepare("SELECT name FROM sqlite_master WHERE type='view'")?
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(views.contains(&"project_current_view".to_string()));
        assert!(views.contains(&"task_current_view".to_string()));
    }
}

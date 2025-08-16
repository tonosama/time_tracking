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
        tracing::info!("DatabaseConnection::new: Attempting to connect to database at {:?}", database_path.as_ref());
        
        let conn = match Connection::open(database_path.as_ref()) {
            Ok(conn) => {
                tracing::info!("DatabaseConnection::new: Successfully opened database connection");
                conn
            },
            Err(e) => {
                tracing::error!("DatabaseConnection::new: Failed to open database connection: {}", e);
                return Err(e.into());
            }
        };
        
        // WALモードとforeign_keysを有効化
        tracing::debug!("DatabaseConnection::new: Setting up database pragmas");
        
        if let Err(e) = conn.pragma_update(None, "journal_mode", "WAL") {
            tracing::error!("DatabaseConnection::new: Failed to set journal_mode to WAL: {}", e);
            return Err(e.into());
        }
        tracing::debug!("DatabaseConnection::new: journal_mode set to WAL");
        
        if let Err(e) = conn.pragma_update(None, "synchronous", "NORMAL") {
            tracing::error!("DatabaseConnection::new: Failed to set synchronous to NORMAL: {}", e);
            return Err(e.into());
        }
        tracing::debug!("DatabaseConnection::new: synchronous set to NORMAL");
        
        if let Err(e) = conn.pragma_update(None, "foreign_keys", "ON") {
            tracing::error!("DatabaseConnection::new: Failed to enable foreign_keys: {}", e);
            return Err(e.into());
        }
        tracing::debug!("DatabaseConnection::new: foreign_keys enabled");

        tracing::info!("DatabaseConnection::new: Database connection setup completed successfully");
        Ok(Self { connection: conn })
    }

    /// インメモリデータベースに接続（テスト用）
    pub fn new_in_memory() -> Result<Self> {
        tracing::info!("DatabaseConnection::new_in_memory: Creating in-memory database");
        
        let conn = match Connection::open_in_memory() {
            Ok(conn) => {
                tracing::info!("DatabaseConnection::new_in_memory: Successfully created in-memory database");
                conn
            },
            Err(e) => {
                tracing::error!("DatabaseConnection::new_in_memory: Failed to create in-memory database: {}", e);
                return Err(e.into());
            }
        };
        
        // foreign_keysを有効化
        if let Err(e) = conn.pragma_update(None, "foreign_keys", "ON") {
            tracing::error!("DatabaseConnection::new_in_memory: Failed to enable foreign_keys: {}", e);
            return Err(e.into());
        }
        tracing::debug!("DatabaseConnection::new_in_memory: foreign_keys enabled");

        tracing::info!("DatabaseConnection::new_in_memory: In-memory database setup completed successfully");
        Ok(Self { connection: conn })
    }

    /// コネクションの参照を取得
    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    /// マイグレーションを実行
    pub fn run_migrations(&self) -> Result<()> {
        tracing::info!("DatabaseConnection::run_migrations: Starting database migrations");
        
        if let Err(e) = self.create_schema() {
            tracing::error!("DatabaseConnection::run_migrations: Failed to create schema: {}", e);
            return Err(e);
        }
        
        if let Err(e) = self.create_views() {
            tracing::error!("DatabaseConnection::run_migrations: Failed to create views: {}", e);
            return Err(e);
        }
        
        if let Err(e) = self.load_sample_data() {
            tracing::error!("DatabaseConnection::run_migrations: Failed to load sample data: {}", e);
            return Err(e);
        }
        
        tracing::info!("DatabaseConnection::run_migrations: All migrations completed successfully");
        Ok(())
    }

    /// 基本スキーマを作成
    fn create_schema(&self) -> Result<()> {
        tracing::info!("DatabaseConnection::create_schema: Creating database schema");
        let schema_sql = include_str!("../../../../database/migrations/001_initial_schema.sql");
        
        tracing::debug!("DatabaseConnection::create_schema: Schema SQL length: {} characters", schema_sql.len());
        
        match self.connection.execute_batch(schema_sql) {
            Ok(_) => {
                tracing::info!("DatabaseConnection::create_schema: Database schema created successfully");
                Ok(())
            },
            Err(e) => {
                tracing::error!("DatabaseConnection::create_schema: Failed to create database schema: {}", e);
                tracing::error!("DatabaseConnection::create_schema: Schema SQL: {}", schema_sql);
                Err(e.into())
            }
        }
    }

    /// ビューを作成
    fn create_views(&self) -> Result<()> {
        tracing::info!("DatabaseConnection::create_views: Creating database views");
        let views_sql = include_str!("../../../../database/migrations/002_views.sql");
        
        tracing::debug!("DatabaseConnection::create_views: Views SQL length: {} characters", views_sql.len());
        
        match self.connection.execute_batch(views_sql) {
            Ok(_) => {
                tracing::info!("DatabaseConnection::create_views: Database views created successfully");
                Ok(())
            },
            Err(e) => {
                tracing::error!("DatabaseConnection::create_views: Failed to create database views: {}", e);
                tracing::error!("DatabaseConnection::create_views: Views SQL: {}", views_sql);
                Err(e.into())
            }
        }
    }

    /// サンプルデータを読み込み
    fn load_sample_data(&self) -> Result<()> {
        tracing::info!("DatabaseConnection::load_sample_data: Checking if sample data should be loaded");
        
        // プロジェクトが存在しない場合のみサンプルデータを読み込み
        let project_count: i64 = match self.connection
            .query_row("SELECT COUNT(*) FROM project_current_view", [], |row| row.get(0)) {
            Ok(count) => {
                tracing::debug!("DatabaseConnection::load_sample_data: Found {} existing projects", count);
                count
            },
            Err(e) => {
                tracing::warn!("DatabaseConnection::load_sample_data: Failed to count projects, assuming 0: {}", e);
                0
            }
        };
        
        if project_count == 0 {
            tracing::info!("DatabaseConnection::load_sample_data: No projects found, loading sample data");
            let sample_data_sql = include_str!("../../../../database/seeds/sample_data.sql");
            
            tracing::debug!("DatabaseConnection::load_sample_data: Sample data SQL length: {} characters", sample_data_sql.len());
            
            match self.connection.execute_batch(sample_data_sql) {
                Ok(_) => {
                    tracing::info!("DatabaseConnection::load_sample_data: Sample data loaded successfully");
                },
                Err(e) => {
                    tracing::error!("DatabaseConnection::load_sample_data: Failed to load sample data: {}", e);
                    return Err(e.into());
                }
            }
        } else {
            tracing::info!("DatabaseConnection::load_sample_data: Projects already exist, skipping sample data load");
        }
        
        Ok(())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn インメモリデータベース接続が正しく動作すること() -> Result<()> {
        let db = DatabaseConnection::new_in_memory()?;
        db.run_migrations()?;

        // テーブルが作成されていることを確認
        let mut stmt = db.connection()
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")?;
        let table_iter = stmt.query_map([], |row| row.get(0))?;
        let tables: Vec<String> = table_iter.collect::<Result<Vec<_>, _>>()?;

        assert!(tables.contains(&"projects".to_string()));
        assert!(tables.contains(&"project_versions".to_string()));
        assert!(tables.contains(&"tasks".to_string()));
        assert!(tables.contains(&"task_versions".to_string()));
        Ok(())
    }

    #[test]
    fn データベースビュー作成が正しく動作すること() -> Result<()> {
        let db = DatabaseConnection::new_in_memory()?;
        db.run_migrations()?;

        // ビューが作成されていることを確認
        let mut stmt = db.connection()
            .prepare("SELECT name FROM sqlite_master WHERE type='view'")?;
        let view_iter = stmt.query_map([], |row| row.get(0))?;
        let views: Vec<String> = view_iter.collect::<Result<Vec<_>, _>>()?;

        assert!(views.contains(&"project_current_view".to_string()));
        assert!(views.contains(&"task_current_view".to_string()));
        Ok(())
    }
}

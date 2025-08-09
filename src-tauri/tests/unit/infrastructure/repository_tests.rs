// インフラ層のリポジトリ実装テスト

#[cfg(test)]
mod tests {
    use time_tracker_go::infrastructure::database::DatabaseConnection;

    #[test]
    fn test_database_connection_in_memory() {
        let db = DatabaseConnection::new_in_memory().unwrap();
        db.run_migrations().unwrap();

        // テーブルが作成されていることを確認
        let mut stmt = db.connection()
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap();
        
        let table_iter = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            Ok(name)
        }).unwrap();

        let tables: Vec<String> = table_iter.collect::<Result<Vec<_>, _>>().unwrap();

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
        let mut stmt = db.connection()
            .prepare("SELECT name FROM sqlite_master WHERE type='view'")
            .unwrap();
        
        let view_iter = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            Ok(name)
        }).unwrap();

        let views: Vec<String> = view_iter.collect::<Result<Vec<_>, _>>().unwrap();

        assert!(views.contains(&"project_current_view".to_string()));
        assert!(views.contains(&"task_current_view".to_string()));
    }
}

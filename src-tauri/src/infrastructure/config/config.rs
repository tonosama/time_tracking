use std::path::PathBuf;

/// アプリケーション設定
#[derive(Debug, Clone)]
pub struct Config {
    pub database_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        // データディレクトリを取得
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("time-tracker-go");

        // ディレクトリが存在しない場合は作成
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir).unwrap_or_else(|e| {
                eprintln!("Failed to create data directory: {}", e);
            });
        }

        Self {
            database_path: data_dir.join("time_tracker.db"),
        }
    }
}

impl Config {
    /// 新しい設定を作成
    pub fn new(database_path: PathBuf) -> Self {
        Self { database_path }
    }

    /// インメモリデータベース用の設定
    pub fn in_memory() -> Self {
        Self {
            database_path: PathBuf::from(":memory:"),
        }
    }

    /// テスト用の設定
    #[cfg(test)]
    pub fn for_test() -> Self {
        use std::env;
        
        let temp_dir = env::temp_dir().join("time-tracker-go-test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        
        Self {
            database_path: temp_dir.join("test.db"),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn デフォルト設定が作成されること() {
        let config = Config::default();
        assert!(config.database_path.to_string_lossy().contains("time-tracker-go"));
        assert!(config.database_path.to_string_lossy().ends_with("time_tracker.db"));
    }

    #[test]
    fn インメモリ設定が作成されること() {
        let config = Config::in_memory();
        assert_eq!(config.database_path.to_string_lossy(), ":memory:");
    }

    #[test]
    fn テスト用設定が作成されること() {
        let config = Config::for_test();
        assert!(config.database_path.to_string_lossy().contains("test.db"));
    }

    #[test]
    fn カスタム設定が作成されること() {
        let custom_path = PathBuf::from("/custom/path/db.sqlite");
        let config = Config::new(custom_path.clone());
        assert_eq!(config.database_path, custom_path);
    }
}

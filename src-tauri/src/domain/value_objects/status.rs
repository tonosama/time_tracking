use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// エンティティのステータス値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Active,
    Archived,
}

impl Status {
    /// 文字列からステータスを作成
    pub fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "active" => Ok(Status::Active),
            "archived" => Ok(Status::Archived),
            _ => Err(anyhow::anyhow!("Invalid status: {}", s)),
        }
    }

    /// ステータスが有効かどうか
    pub fn is_active(&self) -> bool {
        matches!(self, Status::Active)
    }

    /// ステータスがアーカイブ済みかどうか
    pub fn is_archived(&self) -> bool {
        matches!(self, Status::Archived)
    }

    /// データベース保存用の文字列に変換
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Active => "active",
            Status::Archived => "archived",
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for Status {
    fn default() -> Self {
        Status::Active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_from_str() {
        assert_eq!(Status::from_str("active").unwrap(), Status::Active);
        assert_eq!(Status::from_str("archived").unwrap(), Status::Archived);
        assert!(Status::from_str("invalid").is_err());
    }

    #[test]
    fn test_status_is_active() {
        assert!(Status::Active.is_active());
        assert!(!Status::Archived.is_active());
    }

    #[test]
    fn test_status_is_archived() {
        assert!(!Status::Active.is_archived());
        assert!(Status::Archived.is_archived());
    }

    #[test]
    fn test_status_as_str() {
        assert_eq!(Status::Active.as_str(), "active");
        assert_eq!(Status::Archived.as_str(), "archived");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", Status::Active), "active");
        assert_eq!(format!("{}", Status::Archived), "archived");
    }

    #[test]
    fn test_status_default() {
        assert_eq!(Status::default(), Status::Active);
    }

    #[test]
    fn test_status_equality() {
        assert_eq!(Status::Active, Status::Active);
        assert_eq!(Status::Archived, Status::Archived);
        assert_ne!(Status::Active, Status::Archived);
    }
}

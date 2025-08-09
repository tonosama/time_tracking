use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// プロジェクトID値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(i64);

impl ProjectId {
    /// 新しいプロジェクトIDを作成
    pub fn new(id: i64) -> anyhow::Result<Self> {
        if id <= 0 {
            return Err(anyhow::anyhow!("ProjectId must be positive"));
        }
        Ok(Self(id))
    }

    /// 内部値を取得
    pub fn value(&self) -> i64 {
        self.0
    }
}

impl Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ProjectId> for i64 {
    fn from(id: ProjectId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_id_creation() {
        let id = ProjectId::new(1).unwrap();
        assert_eq!(id.value(), 1);
    }

    #[test]
    fn test_project_id_positive_validation() {
        assert!(ProjectId::new(1).is_ok());
        assert!(ProjectId::new(0).is_err());
        assert!(ProjectId::new(-1).is_err());
    }

    #[test]
    fn test_project_id_equality() {
        let id1 = ProjectId::new(1).unwrap();
        let id2 = ProjectId::new(1).unwrap();
        let id3 = ProjectId::new(2).unwrap();

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_project_id_display() {
        let id = ProjectId::new(123).unwrap();
        assert_eq!(format!("{}", id), "123");
    }

    #[test]
    fn test_project_id_conversion() {
        let id = ProjectId::new(42).unwrap();
        let value: i64 = id.into();
        assert_eq!(value, 42);
    }
}

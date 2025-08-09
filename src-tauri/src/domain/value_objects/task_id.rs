use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// タスクID値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(i64);

impl TaskId {
    /// 新しいタスクIDを作成
    pub fn new(id: i64) -> anyhow::Result<Self> {
        if id <= 0 {
            return Err(anyhow::anyhow!("TaskId must be positive"));
        }
        Ok(Self(id))
    }

    /// 内部値を取得
    pub fn value(&self) -> i64 {
        self.0
    }
}

impl Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<TaskId> for i64 {
    fn from(id: TaskId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_id_creation() {
        let id = TaskId::new(1).unwrap();
        assert_eq!(id.value(), 1);
    }

    #[test]
    fn test_task_id_positive_validation() {
        assert!(TaskId::new(1).is_ok());
        assert!(TaskId::new(0).is_err());
        assert!(TaskId::new(-1).is_err());
    }

    #[test]
    fn test_task_id_equality() {
        let id1 = TaskId::new(1).unwrap();
        let id2 = TaskId::new(1).unwrap();
        let id3 = TaskId::new(2).unwrap();

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_task_id_display() {
        let id = TaskId::new(456).unwrap();
        assert_eq!(format!("{}", id), "456");
    }

    #[test]
    fn test_task_id_conversion() {
        let id = TaskId::new(99).unwrap();
        let value: i64 = id.into();
        assert_eq!(value, 99);
    }
}

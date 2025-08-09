use crate::domain::value_objects::{ProjectId, TaskId, Status};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// タスクエンティティ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    id: TaskId,
    project_id: ProjectId,
    name: String,
    status: Status,
    effective_at: DateTime<Utc>,
}

impl Task {
    /// 新しいタスクを作成
    pub fn new(id: TaskId, project_id: ProjectId, name: String) -> anyhow::Result<Self> {
        if name.trim().is_empty() {
            return Err(anyhow::anyhow!("Task name cannot be empty"));
        }
        if name.len() > 255 {
            return Err(anyhow::anyhow!("Task name cannot exceed 255 characters"));
        }

        Ok(Self {
            id,
            project_id,
            name: name.trim().to_string(),
            status: Status::Active,
            effective_at: Utc::now(),
        })
    }

    /// 指定した時刻でタスクを作成
    pub fn new_with_time(
        id: TaskId,
        project_id: ProjectId,
        name: String,
        effective_at: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        if name.trim().is_empty() {
            return Err(anyhow::anyhow!("Task name cannot be empty"));
        }
        if name.len() > 255 {
            return Err(anyhow::anyhow!("Task name cannot exceed 255 characters"));
        }

        Ok(Self {
            id,
            project_id,
            name: name.trim().to_string(),
            status: Status::Active,
            effective_at,
        })
    }

    /// タスク名を変更
    pub fn change_name(&self, new_name: String) -> anyhow::Result<Self> {
        if new_name.trim().is_empty() {
            return Err(anyhow::anyhow!("Task name cannot be empty"));
        }
        if new_name.len() > 255 {
            return Err(anyhow::anyhow!("Task name cannot exceed 255 characters"));
        }

        Ok(Self {
            id: self.id,
            project_id: self.project_id,
            name: new_name.trim().to_string(),
            status: self.status.clone(),
            effective_at: Utc::now(),
        })
    }

    /// タスクを別のプロジェクトに移動
    pub fn move_to_project(&self, new_project_id: ProjectId) -> Self {
        Self {
            id: self.id,
            project_id: new_project_id,
            name: self.name.clone(),
            status: self.status.clone(),
            effective_at: Utc::now(),
        }
    }

    /// タスクをアーカイブ
    pub fn archive(&self) -> Self {
        Self {
            id: self.id,
            project_id: self.project_id,
            name: self.name.clone(),
            status: Status::Archived,
            effective_at: Utc::now(),
        }
    }

    /// タスクを復元（アクティブに戻す）
    pub fn restore(&self) -> anyhow::Result<Self> {
        if !self.status.is_archived() {
            return Err(anyhow::anyhow!("Task is not archived"));
        }

        Ok(Self {
            id: self.id,
            project_id: self.project_id,
            name: self.name.clone(),
            status: Status::Active,
            effective_at: Utc::now(),
        })
    }

    // Getters
    pub fn id(&self) -> TaskId {
        self.id
    }

    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn effective_at(&self) -> DateTime<Utc> {
        self.effective_at
    }

    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    pub fn is_archived(&self) -> bool {
        self.status.is_archived()
    }

    /// 指定したプロジェクトに属するかどうか
    pub fn belongs_to_project(&self, project_id: ProjectId) -> bool {
        self.project_id == project_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_task_creation() {
        let task_id = TaskId::new(1).unwrap();
        let project_id = ProjectId::new(1).unwrap();
        let task = Task::new(task_id, project_id, "Test Task".to_string()).unwrap();

        assert_eq!(task.id(), task_id);
        assert_eq!(task.project_id(), project_id);
        assert_eq!(task.name(), "Test Task");
        assert_eq!(task.status(), &Status::Active);
        assert!(task.is_active());
        assert!(!task.is_archived());
    }

    #[test]
    fn test_task_creation_with_time() {
        let task_id = TaskId::new(1).unwrap();
        let project_id = ProjectId::new(1).unwrap();
        let time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let task = Task::new_with_time(task_id, project_id, "Test Task".to_string(), time).unwrap();

        assert_eq!(task.effective_at(), time);
    }

    #[test]
    fn test_task_name_validation() {
        let task_id = TaskId::new(1).unwrap();
        let project_id = ProjectId::new(1).unwrap();

        // 空文字列
        assert!(Task::new(task_id, project_id, "".to_string()).is_err());
        assert!(Task::new(task_id, project_id, "   ".to_string()).is_err());

        // 長すぎる名前
        let long_name = "a".repeat(256);
        assert!(Task::new(task_id, project_id, long_name).is_err());

        // 正常な名前
        assert!(Task::new(task_id, project_id, "Valid Name".to_string()).is_ok());
    }

    #[test]
    fn test_task_name_trimming() {
        let task_id = TaskId::new(1).unwrap();
        let project_id = ProjectId::new(1).unwrap();
        let task = Task::new(task_id, project_id, "  Test Task  ".to_string()).unwrap();

        assert_eq!(task.name(), "Test Task");
    }

    #[test]
    fn test_task_change_name() {
        let task_id = TaskId::new(1).unwrap();
        let project_id = ProjectId::new(1).unwrap();
        let task = Task::new(task_id, project_id, "Original Name".to_string()).unwrap();
        let updated_task = task.change_name("New Name".to_string()).unwrap();

        assert_eq!(updated_task.name(), "New Name");
        assert_eq!(updated_task.id(), task_id);
        assert_eq!(updated_task.project_id(), project_id);
        assert_ne!(updated_task.effective_at(), task.effective_at());
    }

    #[test]
    fn test_task_change_name_validation() {
        let task_id = TaskId::new(1).unwrap();
        let project_id = ProjectId::new(1).unwrap();
        let task = Task::new(task_id, project_id, "Original Name".to_string()).unwrap();

        assert!(task.change_name("".to_string()).is_err());
        assert!(task.change_name("a".repeat(256)).is_err());
    }

    #[test]
    fn test_task_move_to_project() {
        let task_id = TaskId::new(1).unwrap();
        let project_id = ProjectId::new(1).unwrap();
        let new_project_id = ProjectId::new(2).unwrap();
        let task = Task::new(task_id, project_id, "Test Task".to_string()).unwrap();
        let moved_task = task.move_to_project(new_project_id);

        assert_eq!(moved_task.project_id(), new_project_id);
        assert_eq!(moved_task.id(), task_id);
        assert_eq!(moved_task.name(), "Test Task");
        assert_ne!(moved_task.effective_at(), task.effective_at());
    }

    #[test]
    fn test_task_archive() {
        let task_id = TaskId::new(1).unwrap();
        let project_id = ProjectId::new(1).unwrap();
        let task = Task::new(task_id, project_id, "Test Task".to_string()).unwrap();
        let archived_task = task.archive();

        assert_eq!(archived_task.status(), &Status::Archived);
        assert!(!archived_task.is_active());
        assert!(archived_task.is_archived());
        assert_ne!(archived_task.effective_at(), task.effective_at());
    }

    #[test]
    fn test_task_restore() {
        let task_id = TaskId::new(1).unwrap();
        let project_id = ProjectId::new(1).unwrap();
        let task = Task::new(task_id, project_id, "Test Task".to_string()).unwrap();
        let archived_task = task.archive();
        let restored_task = archived_task.restore().unwrap();

        assert_eq!(restored_task.status(), &Status::Active);
        assert!(restored_task.is_active());
        assert!(!restored_task.is_archived());
    }

    #[test]
    fn test_task_restore_validation() {
        let task_id = TaskId::new(1).unwrap();
        let project_id = ProjectId::new(1).unwrap();
        let task = Task::new(task_id, project_id, "Test Task".to_string()).unwrap();

        // アクティブなタスクは復元できない
        assert!(task.restore().is_err());
    }

    #[test]
    fn test_task_belongs_to_project() {
        let task_id = TaskId::new(1).unwrap();
        let project_id = ProjectId::new(1).unwrap();
        let other_project_id = ProjectId::new(2).unwrap();
        let task = Task::new(task_id, project_id, "Test Task".to_string()).unwrap();

        assert!(task.belongs_to_project(project_id));
        assert!(!task.belongs_to_project(other_project_id));
    }

    #[test]
    fn test_task_equality() {
        let task_id = TaskId::new(1).unwrap();
        let project_id = ProjectId::new(1).unwrap();
        let time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

        let task1 = Task::new_with_time(task_id, project_id, "Test".to_string(), time).unwrap();
        let task2 = Task::new_with_time(task_id, project_id, "Test".to_string(), time).unwrap();

        assert_eq!(task1, task2);
    }
}

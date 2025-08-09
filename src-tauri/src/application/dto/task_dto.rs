use crate::domain::entities::Task;
use crate::domain::value_objects::{ProjectId, TaskId, Status};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// タスク作成リクエストDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub project_id: i64,
    pub name: String,
}

/// タスク更新リクエストDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub id: i64,
    pub name: Option<String>,
    pub project_id: Option<i64>,
}

/// タスクアーカイブリクエストDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveTaskRequest {
    pub id: i64,
}

/// タスク復元リクエストDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreTaskRequest {
    pub id: i64,
}

/// タスクレスポンスDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDto {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub status: String,
    pub effective_at: String,
}

impl From<Task> for TaskDto {
    fn from(task: Task) -> Self {
        Self {
            id: task.id().value(),
            project_id: task.project_id().value(),
            name: task.name().to_string(),
            status: task.status().as_str().to_string(),
            effective_at: task.effective_at().to_rfc3339(),
        }
    }
}

impl TaskDto {
    /// DTOからドメインエンティティに変換
    pub fn to_domain(&self) -> anyhow::Result<Task> {
        let task_id = TaskId::new(self.id)?;
        let project_id = ProjectId::new(self.project_id)?;
        let status = Status::from_str(&self.status)?;
        let effective_at = DateTime::parse_from_rfc3339(&self.effective_at)?
            .with_timezone(&Utc);

        let mut task = Task::new_with_time(task_id, project_id, self.name.clone(), effective_at)?;
        
        if status.is_archived() {
            task = task.archive();
        }
        
        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_dto_from_domain() {
        let task_id = TaskId::new(1).unwrap();
        let project_id = ProjectId::new(1).unwrap();
        let task = Task::new(task_id, project_id, "Test Task".to_string()).unwrap();
        let dto = TaskDto::from(task.clone());

        assert_eq!(dto.id, 1);
        assert_eq!(dto.project_id, 1);
        assert_eq!(dto.name, "Test Task");
        assert_eq!(dto.status, "active");
    }

    #[test]
    fn test_task_dto_to_domain() {
        let dto = TaskDto {
            id: 1,
            project_id: 1,
            name: "Test Task".to_string(),
            status: "active".to_string(),
            effective_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let task = dto.to_domain().unwrap();
        assert_eq!(task.id().value(), 1);
        assert_eq!(task.project_id().value(), 1);
        assert_eq!(task.name(), "Test Task");
        assert!(task.is_active());
    }

    #[test]
    fn test_archived_task_dto() {
        let task_id = TaskId::new(1).unwrap();
        let project_id = ProjectId::new(1).unwrap();
        let task = Task::new(task_id, project_id, "Archived Task".to_string()).unwrap();
        let archived_task = task.archive();
        let dto = TaskDto::from(archived_task);

        assert_eq!(dto.status, "archived");

        let domain_task = dto.to_domain().unwrap();
        assert!(domain_task.is_archived());
    }
}

use crate::domain::entities::Project;
use crate::domain::value_objects::{ProjectId, Status};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// プロジェクト作成リクエストDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
}

impl CreateProjectRequest {
    pub fn to_command(&self) -> anyhow::Result<crate::application::use_cases::CreateProjectCommand> {
        Ok(crate::application::use_cases::CreateProjectCommand {
            name: self.name.clone(),
        })
    }
}

/// プロジェクト更新リクエストDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    pub id: i64,
    pub name: String,
}

/// プロジェクトアーカイブリクエストDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveProjectRequest {
    pub id: i64,
    pub force: bool,
}

/// プロジェクト復元リクエストDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreProjectRequest {
    pub id: i64,
}

/// プロジェクトレスポンスDTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDto {
    pub id: i64,
    pub name: String,
    pub status: String,
    pub effective_at: String,
}

impl From<Project> for ProjectDto {
    fn from(project: Project) -> Self {
        Self {
            id: project.id().value(),
            name: project.name().to_string(),
            status: project.status().as_str().to_string(),
            effective_at: project.effective_at().to_rfc3339(),
        }
    }
}

impl ProjectDto {
    /// DTOからドメインエンティティに変換
    pub fn to_domain(&self) -> anyhow::Result<Project> {
        let project_id = ProjectId::new(self.id)?;
        let status = Status::from_str(&self.status)?;
        let effective_at = DateTime::parse_from_rfc3339(&self.effective_at)?
            .with_timezone(&Utc);

        let mut project = Project::new_with_time(project_id, self.name.clone(), effective_at)?;
        
        if status.is_archived() {
            project = project.archive();
        }
        
        Ok(project)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;


    #[test]
    fn ドメインからプロジェクトDTO変換が正しく動作すること() {
        let project_id = ProjectId::new(1).unwrap();
        let project = Project::new(project_id, "Test Project".to_string()).unwrap();
        let dto = ProjectDto::from(project.clone());

        assert_eq!(dto.id, 1);
        assert_eq!(dto.name, "Test Project");
        assert_eq!(dto.status, "active");
    }

    #[test]
    fn プロジェクトDTOからドメイン変換が正しく動作すること() {
        let dto = ProjectDto {
            id: 1,
            name: "Test Project".to_string(),
            status: "active".to_string(),
            effective_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let project = dto.to_domain().unwrap();
        assert_eq!(project.id().value(), 1);
        assert_eq!(project.name(), "Test Project");
        assert!(project.is_active());
    }

    #[test]
    fn アーカイブプロジェクトDTO変換が正しく動作すること() {
        let project_id = ProjectId::new(1).unwrap();
        let project = Project::new(project_id, "Archived Project".to_string()).unwrap();
        let archived_project = project.archive();
        let dto = ProjectDto::from(archived_project);

        assert_eq!(dto.status, "archived");

        let domain_project = dto.to_domain().unwrap();
        assert!(domain_project.is_archived());
    }
}

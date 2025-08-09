use crate::domain::value_objects::{ProjectId, Status};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// プロジェクトエンティティ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    id: ProjectId,
    name: String,
    status: Status,
    effective_at: DateTime<Utc>,
}

impl Project {
    /// 新しいプロジェクトを作成
    pub fn new(id: ProjectId, name: String) -> anyhow::Result<Self> {
        if name.trim().is_empty() {
            return Err(anyhow::anyhow!("Project name cannot be empty"));
        }
        if name.len() > 255 {
            return Err(anyhow::anyhow!("Project name cannot exceed 255 characters"));
        }

        Ok(Self {
            id,
            name: name.trim().to_string(),
            status: Status::Active,
            effective_at: Utc::now(),
        })
    }

    /// 指定した時刻でプロジェクトを作成
    pub fn new_with_time(
        id: ProjectId,
        name: String,
        effective_at: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        if name.trim().is_empty() {
            return Err(anyhow::anyhow!("Project name cannot be empty"));
        }
        if name.len() > 255 {
            return Err(anyhow::anyhow!("Project name cannot exceed 255 characters"));
        }

        Ok(Self {
            id,
            name: name.trim().to_string(),
            status: Status::Active,
            effective_at,
        })
    }

    /// プロジェクト名を変更
    pub fn change_name(&self, new_name: String) -> anyhow::Result<Self> {
        if new_name.trim().is_empty() {
            return Err(anyhow::anyhow!("Project name cannot be empty"));
        }
        if new_name.len() > 255 {
            return Err(anyhow::anyhow!("Project name cannot exceed 255 characters"));
        }

        Ok(Self {
            id: self.id,
            name: new_name.trim().to_string(),
            status: self.status.clone(),
            effective_at: Utc::now(),
        })
    }

    /// プロジェクトをアーカイブ
    pub fn archive(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            status: Status::Archived,
            effective_at: Utc::now(),
        }
    }

    /// プロジェクトを復元（アクティブに戻す）
    pub fn restore(&self) -> anyhow::Result<Self> {
        if !self.status.is_archived() {
            return Err(anyhow::anyhow!("Project is not archived"));
        }

        Ok(Self {
            id: self.id,
            name: self.name.clone(),
            status: Status::Active,
            effective_at: Utc::now(),
        })
    }

    // Getters
    pub fn id(&self) -> ProjectId {
        self.id
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
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn プロジェクト作成ができること() {
        let id = ProjectId::new(1).unwrap();
        let project = Project::new(id, "Test Project".to_string()).unwrap();

        assert_eq!(project.id(), id);
        assert_eq!(project.name(), "Test Project");
        assert_eq!(project.status(), &Status::Active);
        assert!(project.is_active());
        assert!(!project.is_archived());
    }

    #[test]
    fn 指定時刻でプロジェクト作成ができること() {
        let id = ProjectId::new(1).unwrap();
        let time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let project = Project::new_with_time(id, "Test Project".to_string(), time).unwrap();

        assert_eq!(project.effective_at(), time);
    }

    #[test]
    fn プロジェクト名のバリデーションが機能すること() {
        let id = ProjectId::new(1).unwrap();

        // 空文字列
        assert!(Project::new(id, "".to_string()).is_err());
        assert!(Project::new(id, "   ".to_string()).is_err());

        // 長すぎる名前
        let long_name = "a".repeat(256);
        assert!(Project::new(id, long_name).is_err());

        // 正常な名前
        assert!(Project::new(id, "Valid Name".to_string()).is_ok());
    }

    #[test]
    fn プロジェクト名の前後空白が除去されること() {
        let id = ProjectId::new(1).unwrap();
        let project = Project::new(id, "  Test Project  ".to_string()).unwrap();

        assert_eq!(project.name(), "Test Project");
    }

    #[test]
    fn プロジェクト名変更ができること() {
        let id = ProjectId::new(1).unwrap();
        let project = Project::new(id, "Original Name".to_string()).unwrap();
        let updated_project = project.change_name("New Name".to_string()).unwrap();

        assert_eq!(updated_project.name(), "New Name");
        assert_eq!(updated_project.id(), id);
        assert!(updated_project.effective_at() >= project.effective_at());
    }

    #[test]
    fn プロジェクト名変更時のバリデーションが機能すること() {
        let id = ProjectId::new(1).unwrap();
        let project = Project::new(id, "Original Name".to_string()).unwrap();

        assert!(project.change_name("".to_string()).is_err());
        assert!(project.change_name("a".repeat(256)).is_err());
    }

    #[test]
    fn プロジェクトアーカイブができること() {
        let id = ProjectId::new(1).unwrap();
        let project = Project::new(id, "Test Project".to_string()).unwrap();
        let archived_project = project.archive();

        assert_eq!(archived_project.status(), &Status::Archived);
        assert!(!archived_project.is_active());
        assert!(archived_project.is_archived());
        assert!(archived_project.effective_at() >= project.effective_at());
    }

    #[test]
    fn プロジェクト復元ができること() {
        let id = ProjectId::new(1).unwrap();
        let project = Project::new(id, "Test Project".to_string()).unwrap();
        let archived_project = project.archive();
        let restored_project = archived_project.restore().unwrap();

        assert_eq!(restored_project.status(), &Status::Active);
        assert!(restored_project.is_active());
        assert!(!restored_project.is_archived());
    }

    #[test]
    fn アクティブプロジェクトの復元が失敗すること() {
        let id = ProjectId::new(1).unwrap();
        let project = Project::new(id, "Test Project".to_string()).unwrap();

        // アクティブなプロジェクトは復元できない
        assert!(project.restore().is_err());
    }

    #[test]
    fn プロジェクトの等価性判定が正しく動作すること() {
        let id = ProjectId::new(1).unwrap();
        let time = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();

        let project1 = Project::new_with_time(id, "Test".to_string(), time).unwrap();
        let project2 = Project::new_with_time(id, "Test".to_string(), time).unwrap();

        assert_eq!(project1, project2);
    }
}

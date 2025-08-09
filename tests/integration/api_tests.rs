// 統合テスト - Tauriコマンドのエンドツーエンドテスト

use serde_json::{json, Value};
use std::collections::HashMap;

// テスト用のモック型定義
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectDto {
    pub id: i64,
    pub name: String,
    pub status: String,
    pub effective_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskDto {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub status: String,
    pub effective_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateProjectRequest {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateTaskRequest {
    pub project_id: i64,
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UpdateTaskRequest {
    pub id: i64,
    pub name: Option<String>,
    pub project_id: Option<i64>,
}

// テスト用のTauriコマンドモック
pub struct MockTauriApi {
    projects: HashMap<i64, ProjectDto>,
    tasks: HashMap<i64, TaskDto>,
    next_project_id: i64,
    next_task_id: i64,
}

impl MockTauriApi {
    pub fn new() -> Self {
        Self {
            projects: HashMap::new(),
            tasks: HashMap::new(),
            next_project_id: 1,
            next_task_id: 1,
        }
    }

    // プロジェクト管理コマンド
    pub async fn create_project(&mut self, request: CreateProjectRequest) -> Result<ProjectDto, String> {
        // 名前の重複チェック
        for project in self.projects.values() {
            if project.name == request.name {
                return Err(format!("Project name '{}' already exists", request.name));
            }
        }

        let project = ProjectDto {
            id: self.next_project_id,
            name: request.name,
            status: "active".to_string(),
            effective_at: chrono::Utc::now().to_rfc3339(),
        };

        self.projects.insert(self.next_project_id, project.clone());
        self.next_project_id += 1;

        Ok(project)
    }

    pub async fn update_project(&mut self, request: UpdateProjectRequest) -> Result<ProjectDto, String> {
        let existing_project = self.projects.get(&request.id)
            .ok_or("Project not found")?;

        if existing_project.status == "archived" {
            return Err("Cannot update archived project".to_string());
        }

        // 名前の重複チェック（自分以外）
        for (id, project) in &self.projects {
            if *id != request.id && project.name == request.name {
                return Err(format!("Project name '{}' already exists", request.name));
            }
        }

        let updated_project = ProjectDto {
            id: request.id,
            name: request.name,
            status: existing_project.status.clone(),
            effective_at: chrono::Utc::now().to_rfc3339(),
        };

        self.projects.insert(request.id, updated_project.clone());
        Ok(updated_project)
    }

    pub async fn archive_project(&mut self, project_id: i64, force: bool) -> Result<(), String> {
        let existing_project = self.projects.get(&project_id)
            .ok_or("Project not found")?;

        if existing_project.status == "archived" {
            return Err("Project is already archived".to_string());
        }

        // アクティブなタスクがあるかチェック
        let active_tasks: Vec<_> = self.tasks.values()
            .filter(|task| task.project_id == project_id && task.status == "active")
            .collect();

        if !active_tasks.is_empty() && !force {
            return Err(format!(
                "Cannot archive project with {} active tasks. Use force=true to archive with tasks.",
                active_tasks.len()
            ));
        }

        // forceの場合は関連タスクもアーカイブ
        if force {
            for task_id in self.tasks.keys().cloned().collect::<Vec<_>>() {
                if let Some(task) = self.tasks.get_mut(&task_id) {
                    if task.project_id == project_id && task.status == "active" {
                        task.status = "archived".to_string();
                        task.effective_at = chrono::Utc::now().to_rfc3339();
                    }
                }
            }
        }

        // プロジェクトをアーカイブ
        if let Some(project) = self.projects.get_mut(&project_id) {
            project.status = "archived".to_string();
            project.effective_at = chrono::Utc::now().to_rfc3339();
        }

        Ok(())
    }

    pub async fn restore_project(&mut self, project_id: i64) -> Result<ProjectDto, String> {
        let existing_project = self.projects.get(&project_id)
            .ok_or("Project not found")?;

        if existing_project.status != "archived" {
            return Err("Project is not archived".to_string());
        }

        let restored_project = ProjectDto {
            id: project_id,
            name: existing_project.name.clone(),
            status: "active".to_string(),
            effective_at: chrono::Utc::now().to_rfc3339(),
        };

        self.projects.insert(project_id, restored_project.clone());
        Ok(restored_project)
    }

    pub async fn get_project(&self, project_id: i64) -> Result<Option<ProjectDto>, String> {
        Ok(self.projects.get(&project_id).cloned())
    }

    pub async fn get_all_active_projects(&self) -> Result<Vec<ProjectDto>, String> {
        let active_projects: Vec<ProjectDto> = self.projects.values()
            .filter(|project| project.status == "active")
            .cloned()
            .collect();
        Ok(active_projects)
    }

    pub async fn get_all_projects(&self) -> Result<Vec<ProjectDto>, String> {
        Ok(self.projects.values().cloned().collect())
    }

    // タスク管理コマンド
    pub async fn create_task(&mut self, request: CreateTaskRequest) -> Result<TaskDto, String> {
        let project = self.projects.get(&request.project_id)
            .ok_or("Project not found")?;

        if project.status == "archived" {
            return Err("Cannot create task in archived project".to_string());
        }

        let task = TaskDto {
            id: self.next_task_id,
            project_id: request.project_id,
            name: request.name,
            status: "active".to_string(),
            effective_at: chrono::Utc::now().to_rfc3339(),
        };

        self.tasks.insert(self.next_task_id, task.clone());
        self.next_task_id += 1;

        Ok(task)
    }

    pub async fn update_task(&mut self, request: UpdateTaskRequest) -> Result<TaskDto, String> {
        let existing_task = self.tasks.get(&request.id)
            .ok_or("Task not found")?;

        if existing_task.status == "archived" {
            return Err("Cannot update archived task".to_string());
        }

        let mut updated_task = existing_task.clone();

        // 名前の更新
        if let Some(name) = request.name {
            updated_task.name = name;
        }

        // プロジェクトの移動
        if let Some(new_project_id) = request.project_id {
            let target_project = self.projects.get(&new_project_id)
                .ok_or("Target project not found")?;

            if target_project.status == "archived" {
                return Err("Cannot move task to archived project".to_string());
            }

            updated_task.project_id = new_project_id;
        }

        updated_task.effective_at = chrono::Utc::now().to_rfc3339();
        self.tasks.insert(request.id, updated_task.clone());

        Ok(updated_task)
    }

    pub async fn archive_task(&mut self, task_id: i64) -> Result<(), String> {
        let existing_task = self.tasks.get(&task_id)
            .ok_or("Task not found")?;

        if existing_task.status == "archived" {
            return Err("Task is already archived".to_string());
        }

        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.status = "archived".to_string();
            task.effective_at = chrono::Utc::now().to_rfc3339();
        }

        Ok(())
    }

    pub async fn restore_task(&mut self, task_id: i64) -> Result<TaskDto, String> {
        let existing_task = self.tasks.get(&task_id)
            .ok_or("Task not found")?;

        if existing_task.status != "archived" {
            return Err("Task is not archived".to_string());
        }

        // 所属するプロジェクトがアクティブか確認
        let project = self.projects.get(&existing_task.project_id)
            .ok_or("Project not found")?;

        if project.status == "archived" {
            return Err("Cannot restore task in archived project".to_string());
        }

        let restored_task = TaskDto {
            id: task_id,
            project_id: existing_task.project_id,
            name: existing_task.name.clone(),
            status: "active".to_string(),
            effective_at: chrono::Utc::now().to_rfc3339(),
        };

        self.tasks.insert(task_id, restored_task.clone());
        Ok(restored_task)
    }

    pub async fn get_task(&self, task_id: i64) -> Result<Option<TaskDto>, String> {
        Ok(self.tasks.get(&task_id).cloned())
    }

    pub async fn get_tasks_by_project(&self, project_id: i64) -> Result<Vec<TaskDto>, String> {
        let mut tasks: Vec<TaskDto> = self.tasks.values()
            .filter(|task| task.project_id == project_id)
            .cloned()
            .collect();
        
        tasks.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(tasks)
    }

    pub async fn get_active_tasks_by_project(&self, project_id: i64) -> Result<Vec<TaskDto>, String> {
        let tasks: Vec<TaskDto> = self.tasks.values()
            .filter(|task| task.project_id == project_id && task.status == "active")
            .cloned()
            .collect();
        Ok(tasks)
    }

    pub async fn get_all_active_tasks(&self) -> Result<Vec<TaskDto>, String> {
        let tasks: Vec<TaskDto> = self.tasks.values()
            .filter(|task| task.status == "active")
            .cloned()
            .collect();
        Ok(tasks)
    }
}

// 統合テストケース
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_project_lifecycle() {
        let mut api = MockTauriApi::new();

        // プロジェクト作成
        let create_request = CreateProjectRequest {
            name: "Test Project".to_string(),
        };
        let project = api.create_project(create_request).await.unwrap();
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.status, "active");
        assert_eq!(project.id, 1);

        // プロジェクト取得
        let found_project = api.get_project(project.id).await.unwrap();
        assert!(found_project.is_some());
        assert_eq!(found_project.unwrap().name, "Test Project");

        // プロジェクト更新
        let update_request = UpdateProjectRequest {
            id: project.id,
            name: "Updated Project".to_string(),
        };
        let updated_project = api.update_project(update_request).await.unwrap();
        assert_eq!(updated_project.name, "Updated Project");

        // プロジェクトアーカイブ
        api.archive_project(project.id, false).await.unwrap();
        let archived_project = api.get_project(project.id).await.unwrap().unwrap();
        assert_eq!(archived_project.status, "archived");

        // プロジェクト復元
        let restored_project = api.restore_project(project.id).await.unwrap();
        assert_eq!(restored_project.status, "active");
    }

    #[tokio::test]
    async fn test_task_lifecycle() {
        let mut api = MockTauriApi::new();

        // プロジェクト作成
        let project = api.create_project(CreateProjectRequest {
            name: "Test Project".to_string(),
        }).await.unwrap();

        // タスク作成
        let create_request = CreateTaskRequest {
            project_id: project.id,
            name: "Test Task".to_string(),
        };
        let task = api.create_task(create_request).await.unwrap();
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.project_id, project.id);
        assert_eq!(task.status, "active");

        // タスク取得
        let found_task = api.get_task(task.id).await.unwrap();
        assert!(found_task.is_some());
        assert_eq!(found_task.unwrap().name, "Test Task");

        // タスク更新
        let update_request = UpdateTaskRequest {
            id: task.id,
            name: Some("Updated Task".to_string()),
            project_id: None,
        };
        let updated_task = api.update_task(update_request).await.unwrap();
        assert_eq!(updated_task.name, "Updated Task");

        // タスクアーカイブ
        api.archive_task(task.id).await.unwrap();
        let archived_task = api.get_task(task.id).await.unwrap().unwrap();
        assert_eq!(archived_task.status, "archived");

        // タスク復元
        let restored_task = api.restore_task(task.id).await.unwrap();
        assert_eq!(restored_task.status, "active");
    }

    #[tokio::test]
    async fn test_project_task_hierarchy() {
        let mut api = MockTauriApi::new();

        // プロジェクト作成
        let project = api.create_project(CreateProjectRequest {
            name: "Test Project".to_string(),
        }).await.unwrap();

        // タスク作成
        let task1 = api.create_task(CreateTaskRequest {
            project_id: project.id,
            name: "Task 1".to_string(),
        }).await.unwrap();

        let task2 = api.create_task(CreateTaskRequest {
            project_id: project.id,
            name: "Task 2".to_string(),
        }).await.unwrap();

        // プロジェクトのタスク一覧取得
        let tasks = api.get_tasks_by_project(project.id).await.unwrap();
        assert_eq!(tasks.len(), 2);

        // アクティブなタスク一覧取得
        let active_tasks = api.get_active_tasks_by_project(project.id).await.unwrap();
        assert_eq!(active_tasks.len(), 2);

        // タスクをアーカイブ
        api.archive_task(task1.id).await.unwrap();

        // アクティブなタスク一覧取得（1つ減る）
        let active_tasks = api.get_active_tasks_by_project(project.id).await.unwrap();
        assert_eq!(active_tasks.len(), 1);
        assert_eq!(active_tasks[0].id, task2.id);
    }

    #[tokio::test]
    async fn test_project_archive_with_tasks() {
        let mut api = MockTauriApi::new();

        // プロジェクト作成
        let project = api.create_project(CreateProjectRequest {
            name: "Test Project".to_string(),
        }).await.unwrap();

        // タスク作成
        let task = api.create_task(CreateTaskRequest {
            project_id: project.id,
            name: "Test Task".to_string(),
        }).await.unwrap();

        // アクティブなタスクがある場合はアーカイブできない
        let result = api.archive_project(project.id, false).await;
        assert!(result.is_err());

        // force=trueでアーカイブ（関連タスクも一緒にアーカイブ）
        api.archive_project(project.id, true).await.unwrap();

        // プロジェクトとタスクがアーカイブされていることを確認
        let archived_project = api.get_project(project.id).await.unwrap().unwrap();
        let archived_task = api.get_task(task.id).await.unwrap().unwrap();
        
        assert_eq!(archived_project.status, "archived");
        assert_eq!(archived_task.status, "archived");
    }

    #[tokio::test]
    async fn test_task_move_between_projects() {
        let mut api = MockTauriApi::new();

        // 2つのプロジェクトを作成
        let project1 = api.create_project(CreateProjectRequest {
            name: "Project 1".to_string(),
        }).await.unwrap();

        let project2 = api.create_project(CreateProjectRequest {
            name: "Project 2".to_string(),
        }).await.unwrap();

        // プロジェクト1にタスクを作成
        let task = api.create_task(CreateTaskRequest {
            project_id: project1.id,
            name: "Test Task".to_string(),
        }).await.unwrap();

        // タスクをプロジェクト2に移動
        let update_request = UpdateTaskRequest {
            id: task.id,
            name: None,
            project_id: Some(project2.id),
        };
        let moved_task = api.update_task(update_request).await.unwrap();
        assert_eq!(moved_task.project_id, project2.id);

        // プロジェクト1にはタスクがない
        let project1_tasks = api.get_tasks_by_project(project1.id).await.unwrap();
        assert_eq!(project1_tasks.len(), 0);

        // プロジェクト2にタスクがある
        let project2_tasks = api.get_tasks_by_project(project2.id).await.unwrap();
        assert_eq!(project2_tasks.len(), 1);
        assert_eq!(project2_tasks[0].id, task.id);
    }

    #[tokio::test]
    async fn test_validation_rules() {
        let mut api = MockTauriApi::new();

        // 同じ名前のプロジェクトは作成できない
        api.create_project(CreateProjectRequest {
            name: "Duplicate Name".to_string(),
        }).await.unwrap();

        let result = api.create_project(CreateProjectRequest {
            name: "Duplicate Name".to_string(),
        }).await;
        assert!(result.is_err());

        // アーカイブ済みプロジェクトにはタスクを作成できない
        let project = api.create_project(CreateProjectRequest {
            name: "Archive Test".to_string(),
        }).await.unwrap();

        api.archive_project(project.id, false).await.unwrap();

        let result = api.create_task(CreateTaskRequest {
            project_id: project.id,
            name: "Test Task".to_string(),
        }).await;
        assert!(result.is_err());

        // アーカイブ済みプロジェクトのタスクは復元できない
        let restored_project = api.restore_project(project.id).await.unwrap();
        
        let task = api.create_task(CreateTaskRequest {
            project_id: restored_project.id,
            name: "Test Task".to_string(),
        }).await.unwrap();

        api.archive_task(task.id).await.unwrap();
        api.archive_project(restored_project.id, false).await.unwrap();

        let result = api.restore_task(task.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_all_active_items() {
        let mut api = MockTauriApi::new();

        // 複数のプロジェクトとタスクを作成
        let project1 = api.create_project(CreateProjectRequest {
            name: "Active Project 1".to_string(),
        }).await.unwrap();

        let project2 = api.create_project(CreateProjectRequest {
            name: "Active Project 2".to_string(),
        }).await.unwrap();

        let archived_project = api.create_project(CreateProjectRequest {
            name: "Archived Project".to_string(),
        }).await.unwrap();

        let task1 = api.create_task(CreateTaskRequest {
            project_id: project1.id,
            name: "Active Task 1".to_string(),
        }).await.unwrap();

        let task2 = api.create_task(CreateTaskRequest {
            project_id: project2.id,
            name: "Active Task 2".to_string(),
        }).await.unwrap();

        let archived_task = api.create_task(CreateTaskRequest {
            project_id: project1.id,
            name: "Archived Task".to_string(),
        }).await.unwrap();

        // いくつかをアーカイブ
        api.archive_project(archived_project.id, false).await.unwrap();
        api.archive_task(archived_task.id).await.unwrap();

        // アクティブなプロジェクト一覧取得
        let active_projects = api.get_all_active_projects().await.unwrap();
        assert_eq!(active_projects.len(), 2);

        // アクティブなタスク一覧取得
        let active_tasks = api.get_all_active_tasks().await.unwrap();
        assert_eq!(active_tasks.len(), 2);

        // 全プロジェクト取得（アーカイブ済み含む）
        let all_projects = api.get_all_projects().await.unwrap();
        assert_eq!(all_projects.len(), 3);
    }
}
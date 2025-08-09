use crate::application::dto::{
    CreateProjectRequest, UpdateProjectRequest, ArchiveProjectRequest, 
    RestoreProjectRequest, ProjectDto
};
use crate::application::use_cases::{
    ProjectUseCases, CreateProjectCommand, UpdateProjectCommand, 
    ArchiveProjectCommand, RestoreProjectCommand
};
use crate::domain::value_objects::ProjectId;
use tauri::State;

/// プロジェクト作成コマンド
#[tauri::command]
pub async fn create_project(
    request: CreateProjectRequest,
    project_use_cases: State<'_, Box<dyn ProjectUseCases>>,
) -> Result<ProjectDto, String> {
    let command = CreateProjectCommand {
        name: request.name,
    };

    match project_use_cases.create_project(command).await {
        Ok(project) => Ok(ProjectDto::from(project)),
        Err(e) => Err(e.to_string()),
    }
}

/// プロジェクト更新コマンド
#[tauri::command]
pub async fn update_project(
    request: UpdateProjectRequest,
    project_use_cases: State<'_, Box<dyn ProjectUseCases>>,
) -> Result<ProjectDto, String> {
    let project_id = ProjectId::new(request.id).map_err(|e| e.to_string())?;
    let command = UpdateProjectCommand {
        id: project_id,
        name: request.name,
    };

    match project_use_cases.update_project(command).await {
        Ok(project) => Ok(ProjectDto::from(project)),
        Err(e) => Err(e.to_string()),
    }
}

/// プロジェクトアーカイブコマンド
#[tauri::command]
pub async fn archive_project(
    request: ArchiveProjectRequest,
    project_use_cases: State<'_, Box<dyn ProjectUseCases>>,
) -> Result<(), String> {
    let project_id = ProjectId::new(request.id).map_err(|e| e.to_string())?;
    let command = ArchiveProjectCommand {
        id: project_id,
        force: request.force,
    };

    match project_use_cases.archive_project(command).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// プロジェクト復元コマンド
#[tauri::command]
pub async fn restore_project(
    request: RestoreProjectRequest,
    project_use_cases: State<'_, Box<dyn ProjectUseCases>>,
) -> Result<ProjectDto, String> {
    let project_id = ProjectId::new(request.id).map_err(|e| e.to_string())?;
    let command = RestoreProjectCommand {
        id: project_id,
    };

    match project_use_cases.restore_project(command).await {
        Ok(project) => Ok(ProjectDto::from(project)),
        Err(e) => Err(e.to_string()),
    }
}

/// プロジェクト取得コマンド
#[tauri::command]
pub async fn get_project(
    id: i64,
    project_use_cases: State<'_, Box<dyn ProjectUseCases>>,
) -> Result<Option<ProjectDto>, String> {
    let project_id = ProjectId::new(id).map_err(|e| e.to_string())?;

    match project_use_cases.get_project(project_id).await {
        Ok(Some(project)) => Ok(Some(ProjectDto::from(project))),
        Ok(None) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// 全アクティブプロジェクト取得コマンド
#[tauri::command]
pub async fn get_all_active_projects(
    project_use_cases: State<'_, Box<dyn ProjectUseCases>>,
) -> Result<Vec<ProjectDto>, String> {
    match project_use_cases.get_all_active_projects().await {
        Ok(projects) => Ok(projects.into_iter().map(ProjectDto::from).collect()),
        Err(e) => Err(e.to_string()),
    }
}

/// 全プロジェクト取得コマンド
#[tauri::command]
pub async fn get_all_projects(
    project_use_cases: State<'_, Box<dyn ProjectUseCases>>,
) -> Result<Vec<ProjectDto>, String> {
    match project_use_cases.get_all_projects().await {
        Ok(projects) => Ok(projects.into_iter().map(ProjectDto::from).collect()),
        Err(e) => Err(e.to_string()),
    }
}

/// プロジェクト履歴取得コマンド
#[tauri::command]
pub async fn get_project_history(
    id: i64,
    project_use_cases: State<'_, Box<dyn ProjectUseCases>>,
) -> Result<Vec<ProjectDto>, String> {
    let project_id = ProjectId::new(id).map_err(|e| e.to_string())?;

    match project_use_cases.get_project_history(project_id).await {
        Ok(projects) => Ok(projects.into_iter().map(ProjectDto::from).collect()),
        Err(e) => Err(e.to_string()),
    }
}

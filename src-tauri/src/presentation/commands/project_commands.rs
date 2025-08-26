use crate::application::dto::{
    CreateProjectRequest, UpdateProjectRequest, ArchiveProjectRequest, 
    RestoreProjectRequest, ProjectDto
};
use crate::application::services::ApplicationService;
use crate::application::use_cases::{
    CreateProjectCommand, UpdateProjectCommand, 
    ArchiveProjectCommand, RestoreProjectCommand
};
use crate::domain::value_objects::ProjectId;
use tauri::State;

/// プロジェクト作成コマンド
#[tauri::command]
pub async fn create_project(
    app_service: State<'_, ApplicationService>,
    request: CreateProjectRequest,
) -> Result<ProjectDto, String> {
    let command = CreateProjectCommand {
        name: request.name,
    };

    match app_service.project_use_cases().create_project(command).await {
        Ok(project) => Ok(ProjectDto::from(project)),
        Err(e) => Err(e.to_string()),
    }
}

/// プロジェクト更新コマンド
#[tauri::command]
pub async fn update_project(
    app_service: State<'_, ApplicationService>,
    request: UpdateProjectRequest,
) -> Result<ProjectDto, String> {
    let project_id = ProjectId::new(request.id).map_err(|e| e.to_string())?;
    let command = UpdateProjectCommand {
        id: project_id,
        name: request.name,
    };

    match app_service.project_use_cases().update_project(command).await {
        Ok(project) => Ok(ProjectDto::from(project)),
        Err(e) => Err(e.to_string()),
    }
}

/// プロジェクトアーカイブコマンド
#[tauri::command]
pub async fn archive_project(
    app_service: State<'_, ApplicationService>,
    request: ArchiveProjectRequest,
) -> Result<(), String> {
    let project_id = ProjectId::new(request.id).map_err(|e| e.to_string())?;
    let command = ArchiveProjectCommand {
        id: project_id,
        force: request.force,
    };

    match app_service.project_use_cases().archive_project(command).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// プロジェクト復元コマンド
#[tauri::command]
pub async fn restore_project(
    app_service: State<'_, ApplicationService>,
    request: RestoreProjectRequest,
) -> Result<ProjectDto, String> {
    let project_id = ProjectId::new(request.id).map_err(|e| e.to_string())?;
    let command = RestoreProjectCommand {
        id: project_id,
    };

    match app_service.project_use_cases().restore_project(command).await {
        Ok(project) => Ok(ProjectDto::from(project)),
        Err(e) => Err(e.to_string()),
    }
}

/// プロジェクト取得コマンド
#[tauri::command]
pub async fn get_project(
    app_service: State<'_, ApplicationService>,
    id: i64,
) -> Result<Option<ProjectDto>, String> {
    let project_id = ProjectId::new(id).map_err(|e| e.to_string())?;

    match app_service.project_use_cases().get_project(project_id).await {
        Ok(Some(project)) => Ok(Some(ProjectDto::from(project))),
        Ok(None) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// 全アクティブプロジェクト取得コマンド
#[tauri::command]
pub async fn get_all_active_projects(
    app_service: State<'_, ApplicationService>,
) -> Result<Vec<ProjectDto>, String> {
    println!("[BACKEND] get_all_active_projects called");
    eprintln!("[BACKEND] get_all_active_projects called");
    
    match app_service.project_use_cases().get_all_active_projects().await {
        Ok(projects) => {
            println!("[BACKEND] Found {} active projects", projects.len());
            eprintln!("[BACKEND] Found {} active projects", projects.len());
            Ok(projects.into_iter().map(ProjectDto::from).collect())
        },
        Err(e) => {
            println!("[BACKEND] Error getting active projects: {}", e);
            eprintln!("[BACKEND] Error getting active projects: {}", e);
            // データが存在しない場合は空の配列を返す
            if e.to_string().contains("no rows") {
                Ok(Vec::new())
            } else {
                Err(e.to_string())
            }
        }
    }
}

/// 全プロジェクト取得コマンド
#[tauri::command]
pub async fn get_all_projects(
    app_service: State<'_, ApplicationService>,
) -> Result<Vec<ProjectDto>, String> {
    println!("[BACKEND] get_all_projects called");
    eprintln!("[BACKEND] get_all_projects called");
    
    match app_service.project_use_cases().get_all_projects().await {
        Ok(projects) => {
            println!("[BACKEND] Found {} projects", projects.len());
            eprintln!("[BACKEND] Found {} projects", projects.len());
            Ok(projects.into_iter().map(ProjectDto::from).collect())
        },
        Err(e) => {
            println!("[BACKEND] Error getting projects: {}", e);
            eprintln!("[BACKEND] Error getting projects: {}", e);
            Err(e.to_string())
        }
    }
}

/// プロジェクト履歴取得コマンド
#[tauri::command]
pub async fn get_project_history(
    app_service: State<'_, ApplicationService>,
    id: i64,
) -> Result<Vec<ProjectDto>, String> {
    let project_id = ProjectId::new(id).map_err(|e| e.to_string())?;

    match app_service.project_use_cases().get_project_history(project_id).await {
        Ok(projects) => Ok(projects.into_iter().map(ProjectDto::from).collect()),
        Err(e) => Err(e.to_string()),
    }
}

use crate::application::dto::{
    CreateTaskRequest, UpdateTaskRequest, ArchiveTaskRequest, 
    RestoreTaskRequest, TaskDto
};
use crate::application::services::ApplicationService;
use crate::application::use_cases::{
    CreateTaskCommand, UpdateTaskCommand, 
    ArchiveTaskCommand, RestoreTaskCommand
};
use crate::domain::value_objects::{ProjectId, TaskId};
use tauri::State;

/// タスク作成コマンド
#[tauri::command]
pub async fn create_task(
    app_service: State<'_, ApplicationService>,
    request: CreateTaskRequest,
) -> Result<TaskDto, String> {
    eprintln!("[BACKEND] ===== create_task START =====");
    println!("[BACKEND] ===== create_task START =====");
    eprintln!("[BACKEND] create_task called with request: {:?}", request);
    println!("[BACKEND] create_task called with request: {:?}", request);
    
    let project_id = match ProjectId::new(request.project_id) {
        Ok(id) => {
            eprintln!("[BACKEND] ProjectId created successfully: {:?}", id);
            println!("[BACKEND] ProjectId created successfully: {:?}", id);
            id
        },
        Err(e) => {
            eprintln!("[BACKEND] Failed to create ProjectId: {}", e);
            println!("[BACKEND] Failed to create ProjectId: {}", e);
            return Err(e.to_string());
        }
    };
    
    let command = CreateTaskCommand {
        project_id,
        name: request.name.clone(),
    };
    
    eprintln!("[BACKEND] CreateTaskCommand created: {:?}", command);
    println!("[BACKEND] CreateTaskCommand created: {:?}", command);

    eprintln!("[BACKEND] About to call app_service.task_use_cases().create_task");
    println!("[BACKEND] About to call app_service.task_use_cases().create_task");
    
    match app_service.task_use_cases().create_task(command).await {
        Ok(task) => {
            eprintln!("[BACKEND] Task created successfully: {:?}", task);
            println!("[BACKEND] Task created successfully: {:?}", task);
            eprintln!("[BACKEND] ===== create_task SUCCESS =====");
            println!("[BACKEND] ===== create_task SUCCESS =====");
            Ok(TaskDto::from(task))
        },
        Err(e) => {
            eprintln!("[BACKEND] Task creation failed: {}", e);
            println!("[BACKEND] Task creation failed: {}", e);
            eprintln!("[BACKEND] ===== create_task ERROR =====");
            println!("[BACKEND] ===== create_task ERROR =====");
            Err(e.to_string())
        }
    }
}

/// タスク更新コマンド
#[tauri::command]
pub async fn update_task(
    app_service: State<'_, ApplicationService>,
    request: UpdateTaskRequest,
) -> Result<TaskDto, String> {
    let task_id = TaskId::new(request.id).map_err(|e| e.to_string())?;
    let project_id = request.project_id.map(|id| ProjectId::new(id)).transpose().map_err(|e| e.to_string())?;
    let command = UpdateTaskCommand {
        id: task_id,
        name: request.name,
        project_id,
    };

    match app_service.task_use_cases().update_task(command).await {
        Ok(task) => Ok(TaskDto::from(task)),
        Err(e) => Err(e.to_string()),
    }
}

/// タスクアーカイブコマンド
#[tauri::command]
pub async fn archive_task(
    app_service: State<'_, ApplicationService>,
    request: ArchiveTaskRequest,
) -> Result<(), String> {
    let task_id = TaskId::new(request.id).map_err(|e| e.to_string())?;
    let command = ArchiveTaskCommand {
        id: task_id,
    };

    match app_service.task_use_cases().archive_task(command).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// タスク復元コマンド
#[tauri::command]
pub async fn restore_task(
    app_service: State<'_, ApplicationService>,
    request: RestoreTaskRequest,
) -> Result<TaskDto, String> {
    let task_id = TaskId::new(request.id).map_err(|e| e.to_string())?;
    let command = RestoreTaskCommand {
        id: task_id,
    };

    match app_service.task_use_cases().restore_task(command).await {
        Ok(task) => Ok(TaskDto::from(task)),
        Err(e) => Err(e.to_string()),
    }
}

/// タスク取得コマンド
#[tauri::command]
pub async fn get_task(
    app_service: State<'_, ApplicationService>,
    id: i64,
) -> Result<Option<TaskDto>, String> {
    let task_id = TaskId::new(id).map_err(|e| e.to_string())?;

    match app_service.task_use_cases().get_task(task_id).await {
        Ok(Some(task)) => Ok(Some(TaskDto::from(task))),
        Ok(None) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// プロジェクト別タスク取得コマンド
#[tauri::command]
pub async fn get_tasks_by_project(
    app_service: State<'_, ApplicationService>,
    project_id: i64,
    date: Option<String>,
) -> Result<Vec<TaskDto>, String> {
    println!("[BACKEND] get_tasks_by_project called with project_id: {}, date: {:?}", project_id, date);
    eprintln!("[BACKEND] get_tasks_by_project called with project_id: {}, date: {:?}", project_id, date);
    
    let project_id = ProjectId::new(project_id).map_err(|e| e.to_string())?;

    match app_service.task_use_cases().get_tasks_by_project(project_id).await {
        Ok(tasks) => {
            println!("[BACKEND] Found {} tasks for project {}", tasks.len(), project_id);
            eprintln!("[BACKEND] Found {} tasks for project {}", tasks.len(), project_id);
            
            // タスクの詳細をログ出力
            for (i, task) in tasks.iter().enumerate() {
                println!("[BACKEND] Task {}: id={}, name='{}', project_id={}, status={:?}", 
                    i + 1, task.id().value(), task.name(), task.project_id().value(), task.status());
                eprintln!("[BACKEND] Task {}: id={}, name='{}', project_id={}, status={:?}", 
                    i + 1, task.id().value(), task.name(), task.project_id().value(), task.status());
            }
            
            let dto_tasks: Vec<TaskDto> = tasks.into_iter().map(TaskDto::from).collect();
            
            // DTOの詳細をログ出力
            for (i, dto) in dto_tasks.iter().enumerate() {
                println!("[BACKEND] TaskDto {}: id={}, name='{}', project_id={}, status='{}'", 
                    i + 1, dto.id, dto.name, dto.project_id, dto.status);
                eprintln!("[BACKEND] TaskDto {}: id={}, name='{}', project_id={}, status='{}'", 
                    i + 1, dto.id, dto.name, dto.project_id, dto.status);
            }
            
            Ok(dto_tasks)
        },
        Err(e) => {
            println!("[BACKEND] Error getting tasks: {}", e);
            eprintln!("[BACKEND] Error getting tasks: {}", e);
            Err(e.to_string())
        }
    }
}

/// データベース状態確認コマンド（デバッグ用）
#[tauri::command]
pub async fn debug_database_state(
    app_service: State<'_, ApplicationService>,
) -> Result<String, String> {
    println!("[DEBUG] Database state check requested");
    
    // プロジェクト一覧を取得
    let projects = match app_service.project_use_cases().get_all_active_projects().await {
        Ok(projects) => {
            println!("[DEBUG] Found {} active projects", projects.len());
            projects
        },
        Err(e) => {
            println!("[DEBUG] Failed to get projects: {}", e);
            return Err(format!("Failed to get projects: {}", e));
        }
    };
    
    // タスク一覧を取得
    let tasks = match app_service.task_use_cases().get_all_active_tasks().await {
        Ok(tasks) => {
            println!("[DEBUG] Found {} active tasks", tasks.len());
            tasks
        },
        Err(e) => {
            println!("[DEBUG] Failed to get tasks: {}", e);
            return Err(format!("Failed to get tasks: {}", e));
        }
    };
    
    let result = format!(
        "Database State:\nProjects: {}\nTasks: {}\nProject Details: {:?}\nTask Details: {:?}",
        projects.len(),
        tasks.len(),
        projects,
        tasks
    );
    
    println!("[DEBUG] Database state result: {}", result);
    Ok(result)
}

/// プロジェクト別アクティブタスク取得コマンド
#[tauri::command]
pub async fn get_active_tasks_by_project(
    app_service: State<'_, ApplicationService>,
    project_id: i64,
) -> Result<Vec<TaskDto>, String> {
    let project_id = ProjectId::new(project_id).map_err(|e| e.to_string())?;

    match app_service.task_use_cases().get_active_tasks_by_project(project_id).await {
        Ok(tasks) => Ok(tasks.into_iter().map(TaskDto::from).collect()),
        Err(e) => {
            // データが存在しない場合は空の配列を返す
            if e.to_string().contains("no rows") {
                Ok(Vec::new())
            } else {
                Err(e.to_string())
            }
        }
    }
}

/// 全アクティブタスク取得コマンド
#[tauri::command]
pub async fn get_all_active_tasks(
    app_service: State<'_, ApplicationService>,
) -> Result<Vec<TaskDto>, String> {
    match app_service.task_use_cases().get_all_active_tasks().await {
        Ok(tasks) => Ok(tasks.into_iter().map(TaskDto::from).collect()),
        Err(e) => Err(e.to_string()),
    }
}

/// タスク履歴取得コマンド
#[tauri::command]
pub async fn get_task_history(
    app_service: State<'_, ApplicationService>,
    id: i64,
) -> Result<Vec<TaskDto>, String> {
    let task_id = TaskId::new(id).map_err(|e| e.to_string())?;

    match app_service.task_use_cases().get_task_history(task_id).await {
        Ok(tasks) => Ok(tasks.into_iter().map(TaskDto::from).collect()),
        Err(e) => Err(e.to_string()),
    }
}

/// タスクプロジェクト移動コマンド
#[tauri::command]
pub async fn move_task_to_project(
    app_service: State<'_, ApplicationService>,
    task_id: i64,
    new_project_id: i64,
) -> Result<TaskDto, String> {
    let task_id = TaskId::new(task_id).map_err(|e| e.to_string())?;
    let new_project_id = ProjectId::new(new_project_id).map_err(|e| e.to_string())?;

    match app_service.task_use_cases().move_task_to_project(task_id, new_project_id).await {
        Ok(task) => Ok(TaskDto::from(task)),
        Err(e) => Err(e.to_string()),
    }
}

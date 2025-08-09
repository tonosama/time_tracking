use crate::application::dto::{
    CreateTaskRequest, UpdateTaskRequest, ArchiveTaskRequest, 
    RestoreTaskRequest, TaskDto
};
use crate::application::use_cases::{
    TaskUseCases, CreateTaskCommand, UpdateTaskCommand, 
    ArchiveTaskCommand, RestoreTaskCommand
};
use crate::domain::value_objects::{ProjectId, TaskId};
use tauri::State;

/// タスク作成コマンド
#[tauri::command]
pub async fn create_task(
    request: CreateTaskRequest,
    task_use_cases: State<'_, Box<dyn TaskUseCases>>,
) -> Result<TaskDto, String> {
    let project_id = ProjectId::new(request.project_id).map_err(|e| e.to_string())?;
    let command = CreateTaskCommand {
        project_id,
        name: request.name,
    };

    match task_use_cases.create_task(command).await {
        Ok(task) => Ok(TaskDto::from(task)),
        Err(e) => Err(e.to_string()),
    }
}

/// タスク更新コマンド
#[tauri::command]
pub async fn update_task(
    request: UpdateTaskRequest,
    task_use_cases: State<'_, Box<dyn TaskUseCases>>,
) -> Result<TaskDto, String> {
    let task_id = TaskId::new(request.id).map_err(|e| e.to_string())?;
    let project_id = request.project_id
        .map(|id| ProjectId::new(id))
        .transpose()
        .map_err(|e| e.to_string())?;

    let command = UpdateTaskCommand {
        id: task_id,
        name: request.name,
        project_id,
    };

    match task_use_cases.update_task(command).await {
        Ok(task) => Ok(TaskDto::from(task)),
        Err(e) => Err(e.to_string()),
    }
}

/// タスクアーカイブコマンド
#[tauri::command]
pub async fn archive_task(
    request: ArchiveTaskRequest,
    task_use_cases: State<'_, Box<dyn TaskUseCases>>,
) -> Result<(), String> {
    let task_id = TaskId::new(request.id).map_err(|e| e.to_string())?;
    let command = ArchiveTaskCommand {
        id: task_id,
    };

    match task_use_cases.archive_task(command).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// タスク復元コマンド
#[tauri::command]
pub async fn restore_task(
    request: RestoreTaskRequest,
    task_use_cases: State<'_, Box<dyn TaskUseCases>>,
) -> Result<TaskDto, String> {
    let task_id = TaskId::new(request.id).map_err(|e| e.to_string())?;
    let command = RestoreTaskCommand {
        id: task_id,
    };

    match task_use_cases.restore_task(command).await {
        Ok(task) => Ok(TaskDto::from(task)),
        Err(e) => Err(e.to_string()),
    }
}

/// タスク取得コマンド
#[tauri::command]
pub async fn get_task(
    id: i64,
    task_use_cases: State<'_, Box<dyn TaskUseCases>>,
) -> Result<Option<TaskDto>, String> {
    let task_id = TaskId::new(id).map_err(|e| e.to_string())?;

    match task_use_cases.get_task(task_id).await {
        Ok(Some(task)) => Ok(Some(TaskDto::from(task))),
        Ok(None) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// プロジェクトのタスク一覧取得コマンド
#[tauri::command]
pub async fn get_tasks_by_project(
    project_id: i64,
    task_use_cases: State<'_, Box<dyn TaskUseCases>>,
) -> Result<Vec<TaskDto>, String> {
    let project_id = ProjectId::new(project_id).map_err(|e| e.to_string())?;

    match task_use_cases.get_tasks_by_project(project_id).await {
        Ok(tasks) => Ok(tasks.into_iter().map(TaskDto::from).collect()),
        Err(e) => Err(e.to_string()),
    }
}

/// プロジェクトのアクティブタスク一覧取得コマンド
#[tauri::command]
pub async fn get_active_tasks_by_project(
    project_id: i64,
    task_use_cases: State<'_, Box<dyn TaskUseCases>>,
) -> Result<Vec<TaskDto>, String> {
    let project_id = ProjectId::new(project_id).map_err(|e| e.to_string())?;

    match task_use_cases.get_active_tasks_by_project(project_id).await {
        Ok(tasks) => Ok(tasks.into_iter().map(TaskDto::from).collect()),
        Err(e) => Err(e.to_string()),
    }
}

/// 全アクティブタスク取得コマンド
#[tauri::command]
pub async fn get_all_active_tasks(
    task_use_cases: State<'_, Box<dyn TaskUseCases>>,
) -> Result<Vec<TaskDto>, String> {
    match task_use_cases.get_all_active_tasks().await {
        Ok(tasks) => Ok(tasks.into_iter().map(TaskDto::from).collect()),
        Err(e) => Err(e.to_string()),
    }
}

/// タスク履歴取得コマンド
#[tauri::command]
pub async fn get_task_history(
    id: i64,
    task_use_cases: State<'_, Box<dyn TaskUseCases>>,
) -> Result<Vec<TaskDto>, String> {
    let task_id = TaskId::new(id).map_err(|e| e.to_string())?;

    match task_use_cases.get_task_history(task_id).await {
        Ok(tasks) => Ok(tasks.into_iter().map(TaskDto::from).collect()),
        Err(e) => Err(e.to_string()),
    }
}

/// タスクのプロジェクト移動コマンド
#[tauri::command]
pub async fn move_task_to_project(
    task_id: i64,
    new_project_id: i64,
    task_use_cases: State<'_, Box<dyn TaskUseCases>>,
) -> Result<TaskDto, String> {
    let task_id = TaskId::new(task_id).map_err(|e| e.to_string())?;
    let new_project_id = ProjectId::new(new_project_id).map_err(|e| e.to_string())?;

    match task_use_cases.move_task_to_project(task_id, new_project_id).await {
        Ok(task) => Ok(TaskDto::from(task)),
        Err(e) => Err(e.to_string()),
    }
}

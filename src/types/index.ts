// プロジェクト型定義
export interface Project {
  id: number
  name: string
  status: 'active' | 'archived'
  effective_at: string
  color?: string // Toggl風のプロジェクト色
}

// タスク型定義
export interface Task {
  id: number
  project_id: number
  name: string
  status: 'active' | 'archived'
  effective_at: string
}

// タイムトラッキング関連の型
export interface TimeEntry {
  id?: number
  task_id: number
  start_event_id: number
  start_time: string
  end_time?: string
  duration_in_seconds?: number
  elapsed_duration: string
  is_running: boolean
  is_completed: boolean
  description?: string // Toggl風のタスク説明
  tags?: string[] // Toggl風のタグ
  // Toggl風UIで使いやすいエイリアス
  startedAt?: string
  endAt?: string
}

export interface TimerStatus {
  is_running: boolean
  current_entry?: TimeEntry
  elapsed_seconds?: number
  elapsed_duration?: string
}

export interface CurrentTimer {
  task_id?: number
  elapsed_seconds?: number
  elapsed_duration?: string
}

export interface TaskTimeSummary {
  task_id: number
  total_duration_seconds: number
  total_duration_formatted: string
  entry_count: number
  is_running: boolean
}

// API リクエスト型定義
export interface CreateProjectRequest {
  name: string
}

export interface UpdateProjectRequest {
  id: number
  name: string
}

export interface ArchiveProjectRequest {
  id: number
  force: boolean
}

export interface RestoreProjectRequest {
  id: number
}

export interface CreateTaskRequest {
  project_id: number
  name: string
}

export interface UpdateTaskRequest {
  id: number
  name?: string
  project_id?: number
}

export interface ArchiveTaskRequest {
  id: number
}

export interface RestoreTaskRequest {
  id: number
}

// 時間エントリ型定義（重複削除）

// タグ型定義
export interface Tag {
  id: number
  name: string
}

// UI状態型定義
export interface AppState {
  currentProject?: Project
  currentTask?: Task
  isTracking: boolean
}

// エラー型定義
export interface ApiError {
  message: string
  code?: string
}
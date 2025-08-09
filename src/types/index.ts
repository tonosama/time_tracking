// 型定義 - フロントエンド全体で使用する型定義

export interface Project {
  id: number;
  name: string;
  status: 'active' | 'archived';
  effective_at: string;
}

export interface Task {
  id: number;
  project_id: number;
  name: string;
  status: 'active' | 'archived';
  effective_at: string;
  tags?: Tag[];
}

export interface TimeEntry {
  task_id: number;
  start_event_id: number;
  start_time: string;
  end_time?: string;
  duration_in_seconds?: number;
}

export interface Tag {
  id: number;
  name: string;
}

export interface TimeEntryEvent {
  id: number;
  task_id: number;
  event_type: 'start' | 'stop' | 'annotate';
  at: string;
  start_event_id?: number;
  payload?: string;
}

// API レスポンス型
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

// UI状態管理用の型
export interface AppState {
  currentProject?: Project;
  currentTask?: Task;
  isTracking: boolean;
  currentTimeEntry?: TimeEntry;
}


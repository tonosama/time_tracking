import { invoke } from '@tauri-apps/api/core'
import type { TimeEntry, TimerStatus, CurrentTimer, TaskTimeSummary, Task } from '@/types'

export interface StartTimerRequest {
  task_id: number
}

export interface StopTimerRequest {
  task_id: number
}

export interface AddManualEntryRequest {
  task_id: number
  start_time: string
  end_time: string
  note?: string
}

export interface TimeEntryEventResponse {
  id: number
  task_id: number
  event_type: string
  at: string
  start_event_id?: number
  payload?: string
}

/**
 * タイムトラッキングAPI
 */
export class TimeTrackingApi {
  /**
   * タイマーを開始する
   */
  static async startTimer(taskId: number): Promise<TimeEntryEventResponse> {
    const request: StartTimerRequest = { task_id: taskId }
    return await invoke('start_timer', { request })
  }

  /**
   * タイマーを停止する
   */
  static async stopTimer(taskId: number): Promise<TimeEntryEventResponse | null> {
    const request: StopTimerRequest = { task_id: taskId }
    return await invoke('stop_timer', { request })
  }

  /**
   * 現在実行中のタイマーを取得する
   */
  static async getCurrentTimer(): Promise<CurrentTimer> {
    return await invoke('get_current_timer')
  }

  /**
   * 指定タスクのタイマー状態を取得する
   */
  static async getTimerStatus(taskId: number): Promise<TimerStatus> {
    return await invoke('get_timer_status', { taskId })
  }

  /**
   * 指定タスクを取得する
   */
  static async getTask(taskId: number): Promise<Task | null> {
    return await invoke('get_task', { id: taskId })
  }

  /**
   * 指定タスクが実行中かどうかを取得する
   */
  static async isTaskRunning(taskId: number): Promise<boolean> {
    return await invoke('is_task_running', { taskId })
  }

  /**
   * 手動で時間エントリを追加する
   */
  static async addManualEntry(request: AddManualEntryRequest): Promise<void> {
    return await invoke('add_manual_entry', { request })
  }

  /**
   * 指定タスクの時間エントリ一覧を取得する
   */
  static async getTaskEntries(taskId: number): Promise<TimeEntry[]> {
    return await invoke('get_task_entries', { taskId })
  }

  /**
   * 指定タスクの最近の時間エントリを取得する
   */
  static async getRecentTaskEntries(taskId: number, limit: number): Promise<TimeEntry[]> {
    return await invoke('get_recent_task_entries', { taskId, limit })
  }

  /**
   * 指定プロジェクトの時間エントリ一覧を取得する
   */
  static async getProjectEntries(projectId: number): Promise<TimeEntry[]> {
    return await invoke('get_project_entries', { projectId })
  }

  /**
   * 最近の時間エントリを取得する
   */
  static async getRecentEntries(limit: number): Promise<TimeEntry[]> {
    return await invoke('get_recent_entries', { limit })
  }

  /**
   * 指定タスクの時間サマリーを取得する
   */
  static async getTaskTimeSummary(taskId: number): Promise<TaskTimeSummary> {
    return await invoke('get_task_time_summary', { taskId })
  }

  /**
   * 全ての実行中タイマーを停止する
   */
  static async stopAllTimers(): Promise<TimeEntryEventResponse[]> {
    return await invoke('stop_all_timers')
  }
}

/**
 * 時間フォーマット用のユーティリティ関数
 */
export class TimeFormatUtils {
  /**
   * 秒数をHH:MM:SS形式に変換
   */
  static formatDuration(seconds: number): string {
    const hours = Math.floor(seconds / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    const secs = seconds % 60
    return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`
  }

  /**
   * ISO 8601日時文字列を日本語フォーマットに変換
   */
  static formatDateTime(isoString: string): string {
    const date = new Date(isoString)
    return date.toLocaleString('ja-JP', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    })
  }

  /**
   * ISO 8601日時文字列を時刻のみのフォーマットに変換
   */
  static formatTime(isoString: string): string {
    const date = new Date(isoString)
    return date.toLocaleTimeString('ja-JP', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    })
  }

  /**
   * 今日のISO 8601日時文字列を生成
   */
  static getCurrentISOString(): string {
    return new Date().toISOString()
  }

  /**
   * 指定時刻のISO 8601日時文字列を生成
   */
  static createISOString(hours: number, minutes: number): string {
    const date = new Date()
    date.setHours(hours, minutes, 0, 0)
    return date.toISOString()
  }
}

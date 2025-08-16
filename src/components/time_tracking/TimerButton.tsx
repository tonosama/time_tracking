import { useState, useEffect } from 'react'
import { TimeTrackingApi } from '@/services'
import type { Task, TimerStatus } from '@/types'

interface TimerButtonProps {
  task: Task
  onTimerStateChanged?: (taskId: number, isRunning: boolean) => void
  disabled?: boolean
}

export function TimerButton({ task, onTimerStateChanged, disabled = false }: TimerButtonProps) {
  const [status, setStatus] = useState<TimerStatus>({ is_running: false })
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // タイマー状態を取得
  const loadTimerStatus = async () => {
    try {
      const timerStatus = await TimeTrackingApi.getTimerStatus(task.id)
      setStatus(timerStatus)
    } catch (err) {
      console.error('Failed to load timer status:', err)
      setError('タイマー状態の取得に失敗しました')
    }
  }

  // 初期読み込み
  useEffect(() => {
    loadTimerStatus()
  }, [task.id])

  // タイマー開始
  const handleStartTimer = async () => {
    if (loading || disabled) return

    setLoading(true)
    setError(null)

    try {
      await TimeTrackingApi.startTimer(task.id)
      await loadTimerStatus() // 状態を再読み込み
      onTimerStateChanged?.(task.id, true)
    } catch (err) {
      console.error('Failed to start timer:', err)
      setError('タイマーの開始に失敗しました')
    } finally {
      setLoading(false)
    }
  }

  // タイマー停止
  const handleStopTimer = async () => {
    if (loading || disabled) return

    setLoading(true)
    setError(null)

    try {
      await TimeTrackingApi.stopTimer(task.id)
      await loadTimerStatus() // 状態を再読み込み
      onTimerStateChanged?.(task.id, false)
    } catch (err) {
      console.error('Failed to stop timer:', err)
      setError('タイマーの停止に失敗しました')
    } finally {
      setLoading(false)
    }
  }

  // ボタンのスタイルクラス
  const getButtonClass = () => {
    const baseClass = 'btn btn-sm'
    
    if (disabled) {
      return `${baseClass} btn-secondary`
    }
    
    if (status.is_running) {
      return `${baseClass} btn-danger timer-button-running`
    }
    
    return `${baseClass} btn-success timer-button-stopped`
  }

  // ボタンのテキスト
  const getButtonText = () => {
    if (loading) {
      return status.is_running ? '停止中...' : '開始中...'
    }
    
    return status.is_running ? 'タイマー停止' : 'タイマー開始'
  }

  // アクセシビリティ用のaria-label
  const getAriaLabel = () => {
    const statusText = status.is_running ? '実行中' : '停止中'
    const actionText = status.is_running ? '停止' : '開始'
    return `${task.name}のタイマー（${statusText}）を${actionText}`
  }

  return (
    <div className="timer-button-container">
      <button
        type="button"
        className={getButtonClass()}
        onClick={status.is_running ? handleStopTimer : handleStartTimer}
        disabled={loading || disabled}
        aria-label={getAriaLabel()}
        data-testid="timer-button"
      >
        <span className="timer-button-icon">
          {status.is_running ? '⏹️' : '▶️'}
        </span>
        <span className="timer-button-text">
          {getButtonText()}
        </span>
      </button>
      
      {/* エラー表示 */}
      {error && (
        <div className="timer-button-error" role="alert">
          <small className="text-danger">{error}</small>
        </div>
      )}
      
      {/* 経過時間表示 */}
      {status.is_running && status.elapsed_duration && (
        <div className="timer-button-elapsed" data-testid="timer-elapsed">
          <small className="text-muted">
            経過時間: {status.elapsed_duration}
          </small>
        </div>
      )}
    </div>
  )
}

// タイマーの状態が変更された時に呼び出される関数の型
export type TimerStateChangeHandler = (taskId: number, isRunning: boolean) => void

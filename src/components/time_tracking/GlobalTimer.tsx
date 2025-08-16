import { useState, useEffect, useRef } from 'react'
import { TimeTrackingApi } from '@/services'
import { Logger } from '@/utils'
import type { CurrentTimer, Task } from '@/types'



export function GlobalTimer() {
  const [currentTimer, setCurrentTimer] = useState<CurrentTimer | null>(null)
  const [currentTask, setCurrentTask] = useState<Task | null>(null)
  const [elapsedSeconds, setElapsedSeconds] = useState<number>(0)
  const [isVisible, setIsVisible] = useState(false)
  const intervalRef = useRef<number | null>(null)
  const startTimeRef = useRef<Date | null>(null)

  // 現在のタイマー情報を取得
  const loadCurrentTimer = async () => {
    try {
      const timer = await TimeTrackingApi.getCurrentTimer()
      
      if (timer.task_id && timer.elapsed_seconds !== undefined) {
        setCurrentTimer(timer)
        setElapsedSeconds(timer.elapsed_seconds)
        startTimeRef.current = new Date(Date.now() - timer.elapsed_seconds * 1000)
        setIsVisible(true)
        
        // タスク情報も取得（バックエンドにAPIがあると仮定）
        // 実際の実装では、TaskApiから取得する必要があります
        // const task = await TaskApi.getTask(timer.task_id)
        // setCurrentTask(task)
        
        // 仮のタスク情報
        setCurrentTask({
          id: timer.task_id,
          project_id: 1,
          name: `タスク ${timer.task_id}`,
          status: 'active',
          effective_at: new Date().toISOString()
        })
      } else {
        setCurrentTimer(null)
        setCurrentTask(null)
        setIsVisible(false)
        setElapsedSeconds(0)
        startTimeRef.current = null
      }
    } catch (err) {
      console.error('Failed to load current timer:', err)
      setIsVisible(false)
    }
  }

  // 1秒ごとに経過時間を更新
  useEffect(() => {
    if (currentTimer && startTimeRef.current) {
      intervalRef.current = setInterval(() => {
        if (startTimeRef.current) {
          const now = new Date()
          const elapsed = Math.floor((now.getTime() - startTimeRef.current.getTime()) / 1000)
          setElapsedSeconds(elapsed)
        }
      }, 1000)

      return () => {
        if (intervalRef.current) {
          clearInterval(intervalRef.current)
        }
      }
    }
  }, [currentTimer])

  // 初期読み込みと定期的な更新
  useEffect(() => {
    loadCurrentTimer()
    
    // 30秒ごとにサーバーから状態を再確認
    const updateInterval = setInterval(loadCurrentTimer, 30000)
    
    return () => {
      clearInterval(updateInterval)
      if (intervalRef.current) {
        clearInterval(intervalRef.current)
      }
    }
  }, [])

  // 経過時間をフォーマット
  const formatElapsed = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    const secs = seconds % 60
    return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`
  }

  // タイマーが実行中でない場合は非表示
  if (!isVisible || !currentTimer || !currentTask) {
    return null
  }

  const handleStop = async () => {
    Logger.userAction('GlobalTimer', 'timer_stop_initiated', { 
      taskId: currentTimer?.task_id 
    });
    
    try {
      if (currentTimer?.task_id) {
        Logger.apiCall('GlobalTimer', 'POST', 'stop_timer', { 
          taskId: currentTimer.task_id 
        });
        
        await TimeTrackingApi.stopTimer(currentTimer.task_id)
        
        Logger.apiSuccess('GlobalTimer', 'POST', 'stop_timer', { 
          taskId: currentTimer.task_id 
        });
        Logger.userAction('GlobalTimer', 'timer_stopped', { 
          taskId: currentTimer.task_id 
        });
      }
      setCurrentTimer(null)
      setCurrentTask(null)
      setIsVisible(false)
      setElapsedSeconds(0)
      startTimeRef.current = null
    } catch (error) {
      Logger.apiError('GlobalTimer', 'POST', 'stop_timer', error, { 
        taskId: currentTimer?.task_id 
      });
    }
  }

  return (
    <div className="running-timer-header">
      <div className="timer-content">
        <span className="timer-icon">🎯</span>
        <span className="timer-task-name">
          {currentTask.name}
        </span>
        <span className="timer-separator">-</span>
        <span className="timer-duration">{formatElapsed(elapsedSeconds)}</span>
      </div>
      <button className="timer-stop-btn" onClick={handleStop}>
        ⏹
      </button>
    </div>
  )
}

// グローバルタイマーの状態管理用フック
export function useGlobalTimer() {
  const [isRunning, setIsRunning] = useState(false)
  const [currentTaskId, setCurrentTaskId] = useState<number | null>(null)

  const refreshTimer = async () => {
    try {
      const timer = await TimeTrackingApi.getCurrentTimer()
      setIsRunning(!!timer.task_id)
      setCurrentTaskId(timer.task_id || null)
    } catch (err) {
      console.error('Failed to refresh timer:', err)
      setIsRunning(false)
      setCurrentTaskId(null)
    }
  }

  useEffect(() => {
    refreshTimer()
  }, [])

  return {
    isRunning,
    currentTaskId,
    refreshTimer
  }
}

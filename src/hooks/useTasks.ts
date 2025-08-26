import { useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { Task, CreateTaskRequest, UpdateTaskRequest, ArchiveTaskRequest, RestoreTaskRequest } from '@/types'

interface UseTasksReturn {
  tasks: Task[]
  loading: boolean
  error: string | null
  selectedDate: Date
  createTask: (request: CreateTaskRequest) => Promise<Task>
  updateTask: (request: UpdateTaskRequest) => Promise<Task>
  archiveTask: (request: ArchiveTaskRequest) => Promise<void>
  restoreTask: (request: RestoreTaskRequest) => Promise<Task>
  loadTasks: (projectId: number | null, date?: Date) => Promise<void>
  setSelectedDate: (date: Date) => void
  goToPreviousDay: () => void
  goToNextDay: () => void
  clearError: () => void
}

export function useTasks(): UseTasksReturn {
  console.log('[useTasks] Hook initialized')
  const [tasks, setTasks] = useState<Task[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [selectedDate, setSelectedDate] = useState<Date>(new Date())

  const clearError = useCallback(() => {
    console.log('[useTasks] clearError called')
    setError(null)
  }, [])

  const createTask = useCallback(async (request: CreateTaskRequest): Promise<Task> => {
    try {
      setLoading(true)
      setError(null)
      
      const newTask = await invoke<Task>('create_task', { request })
      
      setTasks(prev => [...prev, newTask])
      return newTask
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'タスクの作成に失敗しました'
      setError(errorMessage)
      throw err
    } finally {
      setLoading(false)
    }
  }, [])

  const updateTask = useCallback(async (request: UpdateTaskRequest): Promise<Task> => {
    try {
      setLoading(true)
      setError(null)
      
      const updatedTask = await invoke<Task>('update_task', { request })
      
      setTasks(prev => prev.map(task => 
        task.id === request.id ? updatedTask : task
      ))
      return updatedTask
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'タスクの更新に失敗しました'
      setError(errorMessage)
      throw err
    } finally {
      setLoading(false)
    }
  }, [])

  const archiveTask = useCallback(async (request: ArchiveTaskRequest): Promise<void> => {
    try {
      setLoading(true)
      setError(null)
      
      await invoke('archive_task', { request })
      
      setTasks(prev => prev.map(task => 
        task.id === request.id ? { ...task, status: 'archived' as const } : task
      ))
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'タスクのアーカイブに失敗しました'
      setError(errorMessage)
      throw err
    } finally {
      setLoading(false)
    }
  }, [])

  const restoreTask = useCallback(async (request: RestoreTaskRequest): Promise<Task> => {
    try {
      setLoading(true)
      setError(null)
      
      const restoredTask = await invoke<Task>('restore_task', { request })
      
      setTasks(prev => prev.map(task => 
        task.id === request.id ? restoredTask : task
      ))
      return restoredTask
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'タスクの復元に失敗しました'
      setError(errorMessage)
      throw err
    } finally {
      setLoading(false)
    }
  }, [])

  const loadTasks = useCallback(async (projectId: number | null, date?: Date): Promise<void> => {
    console.log('[useTasks] loadTasks called:', { projectId, date })
    try {
      setLoading(true)
      setError(null)
      
      if (projectId === null) {
        console.log('[useTasks] ProjectId is null, clearing tasks')
        setTasks([])
        return
      }
      
      const targetDate = date || selectedDate
      const dateString = targetDate.toISOString().split('T')[0]
      console.log('[useTasks] Calling get_tasks_by_project with:', { projectId, date: dateString })
      
      const projectTasks = await invoke<Task[]>('get_tasks_by_project', { 
        projectId,
        date: dateString // YYYY-MM-DD形式
      })
      
      console.log('[useTasks] Received tasks:', projectTasks)
      console.log('[useTasks] Number of tasks received:', projectTasks.length)
      
      // 受け取ったタスクの詳細をログ出力
      projectTasks.forEach((task, i) => {
        console.log(`[useTasks] Task ${i + 1}:`, {
          id: task.id,
          name: task.name,
          project_id: task.project_id,
          status: task.status,
          effective_at: task.effective_at
        })
      })
      
      setTasks(projectTasks)
    } catch (err) {
      console.error('[useTasks] Error loading tasks:', err)
      const errorMessage = err instanceof Error ? err.message : 'タスクの読み込みに失敗しました'
      setError(errorMessage)
      throw err
    } finally {
      setLoading(false)
    }
  }, [selectedDate])

  const handleSetSelectedDate = useCallback((date: Date) => {
    setSelectedDate(date)
  }, [])

  const goToPreviousDay = useCallback(() => {
    const prevDate = new Date(selectedDate)
    prevDate.setDate(prevDate.getDate() - 1)
    setSelectedDate(prevDate)
  }, [selectedDate])

  const goToNextDay = useCallback(() => {
    const nextDate = new Date(selectedDate)
    nextDate.setDate(nextDate.getDate() + 1)
    const today = new Date()
    
    // 日付のみで比較（時刻は無視）
    const nextDateOnly = new Date(nextDate.getFullYear(), nextDate.getMonth(), nextDate.getDate())
    const todayOnly = new Date(today.getFullYear(), today.getMonth(), today.getDate())
    
    if (nextDateOnly <= todayOnly) {
      setSelectedDate(nextDate)
    }
  }, [selectedDate])

  return {
    tasks,
    loading,
    error,
    selectedDate,
    createTask,
    updateTask,
    archiveTask,
    restoreTask,
    loadTasks,
    setSelectedDate: handleSetSelectedDate,
    goToPreviousDay,
    goToNextDay,
    clearError
  }
}

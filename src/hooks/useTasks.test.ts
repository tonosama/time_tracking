import { renderHook, act, waitFor } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { useTasks } from './useTasks'
import type { Task } from '@/types'

// Tauri APIのモック
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

const mockInvoke = vi.mocked(invoke)

// モックデータ
const mockTask: Task = {
  id: 1,
  project_id: 1,
  name: 'テストタスク',
  status: 'active',
  effective_at: '2025-01-01T00:00:00Z'
}

const mockTasks: Task[] = [
  mockTask,
  {
    id: 2,
    project_id: 1,
    name: 'テストタスク2',
    status: 'active',
    effective_at: '2025-01-01T00:00:00Z'
  }
]

describe('useTasks', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // デフォルトのモック設定
    mockInvoke.mockResolvedValue(mockTask)
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  describe('初期状態', () => {
    it('初期状態が正しく設定される', () => {
      const { result } = renderHook(() => useTasks())

      expect(result.current.tasks).toEqual([])
      expect(result.current.loading).toBe(false)
      expect(result.current.error).toBeNull()
      expect(typeof result.current.createTask).toBe('function')
      expect(typeof result.current.updateTask).toBe('function')
      expect(typeof result.current.archiveTask).toBe('function')
      expect(typeof result.current.restoreTask).toBe('function')
      expect(typeof result.current.loadTasks).toBe('function')
      expect(typeof result.current.clearError).toBe('function')
    })
  })

  describe('createTask', () => {
    it('正常にタスクを作成できる', async () => {
      const { result } = renderHook(() => useTasks())

      await act(async () => {
        const newTask = await result.current.createTask({
          project_id: 1,
          name: '新しいタスク'
        })

        expect(newTask).toEqual(mockTask)
      })

      expect(mockInvoke).toHaveBeenCalledWith('create_task', {
        request: {
          project_id: 1,
          name: '新しいタスク'
        }
      })

      expect(result.current.tasks).toContain(mockTask)
      expect(result.current.loading).toBe(false)
      expect(result.current.error).toBeNull()
    })

    it('タスク作成中はローディング状態になる', async () => {
      // 遅延をシミュレート
      mockInvoke.mockImplementation(() => new Promise(resolve => setTimeout(() => resolve(mockTask), 100)))

      const { result } = renderHook(() => useTasks())

      act(() => {
        result.current.createTask({
          project_id: 1,
          name: '新しいタスク'
        })
      })

      expect(result.current.loading).toBe(true)

      await waitFor(() => {
        expect(result.current.loading).toBe(false)
      })
    })

    it('エラーが発生した場合、エラーメッセージが設定される', async () => {
      const errorMessage = 'タスクの作成に失敗しました'
      mockInvoke.mockRejectedValue(new Error(errorMessage))

      const { result } = renderHook(() => useTasks())

      await act(async () => {
        try {
          await result.current.createTask({
            project_id: 1,
            name: '新しいタスク'
          })
        } catch (err) {
          // エラーは期待される動作
        }
      })

      expect(result.current.error).toBe(errorMessage)
      expect(result.current.loading).toBe(false)
      expect(result.current.tasks).toEqual([])
    })

    it('作成されたタスクがタスクリストに追加される', async () => {
      const { result } = renderHook(() => useTasks())

      await act(async () => {
        await result.current.createTask({
          project_id: 1,
          name: '新しいタスク'
        })
      })

      expect(result.current.tasks).toHaveLength(1)
      expect(result.current.tasks[0]).toEqual(mockTask)
    })
  })

  describe('updateTask', () => {
    it('正常にタスクを更新できる', async () => {
      const { result } = renderHook(() => useTasks())

      // 初期タスクを設定（モックを使用）
      mockInvoke.mockResolvedValueOnce([mockTask])
      await act(async () => {
        await result.current.loadTasks(1)
      })

      const updatedTask: Task = {
        ...mockTask,
        name: '更新されたタスク'
      }
      mockInvoke.mockResolvedValue(updatedTask)

      await act(async () => {
        const updatedTaskResult = await result.current.updateTask({
          id: 1,
          name: '更新されたタスク'
        })

        expect(updatedTaskResult).toEqual(updatedTask)
      })

      expect(mockInvoke).toHaveBeenCalledWith('update_task', {
        request: {
          id: 1,
          name: '更新されたタスク'
        }
      })

      expect(result.current.tasks[0].name).toBe('更新されたタスク')
      expect(result.current.loading).toBe(false)
      expect(result.current.error).toBeNull()
    })

    it('存在しないタスクの更新でエラーが発生する', async () => {
      const errorMessage = 'タスクの更新に失敗しました'
      mockInvoke.mockRejectedValue(new Error(errorMessage))

      const { result } = renderHook(() => useTasks())

      await act(async () => {
        try {
          await result.current.updateTask({
            id: 999,
            name: '存在しないタスク'
          })
        } catch (err) {
          // エラーは期待される動作
        }
      })

      expect(result.current.error).toBe(errorMessage)
      expect(result.current.loading).toBe(false)
    })
  })

  describe('archiveTask', () => {
    it('正常にタスクをアーカイブできる', async () => {
      const { result } = renderHook(() => useTasks())

      // 初期タスクを設定（モックを使用）
      mockInvoke.mockResolvedValueOnce([mockTask])
      await act(async () => {
        await result.current.loadTasks(1)
      })

      await act(async () => {
        await result.current.archiveTask({ id: 1 })
      })

      expect(mockInvoke).toHaveBeenCalledWith('archive_task', {
        request: { id: 1 }
      })

      expect(result.current.tasks[0].status).toBe('archived')
      expect(result.current.loading).toBe(false)
      expect(result.current.error).toBeNull()
    })

    it('アーカイブエラーが発生した場合、エラーメッセージが設定される', async () => {
      const errorMessage = 'タスクのアーカイブに失敗しました'
      mockInvoke.mockRejectedValue(new Error(errorMessage))

      const { result } = renderHook(() => useTasks())

      await act(async () => {
        try {
          await result.current.archiveTask({ id: 1 })
        } catch (err) {
          // エラーは期待される動作
        }
      })

      expect(result.current.error).toBe(errorMessage)
      expect(result.current.loading).toBe(false)
    })
  })

  describe('restoreTask', () => {
    it('正常にタスクを復元できる', async () => {
      const { result } = renderHook(() => useTasks())

      const archivedTask: Task = {
        ...mockTask,
        status: 'archived'
      }

      // 初期タスクを設定（モックを使用）
      mockInvoke.mockResolvedValueOnce([archivedTask])
      await act(async () => {
        await result.current.loadTasks(1)
      })

      await act(async () => {
        const restoredTask = await result.current.restoreTask({ id: 1 })

        expect(restoredTask).toEqual(mockTask)
      })

      expect(mockInvoke).toHaveBeenCalledWith('restore_task', {
        request: { id: 1 }
      })

      expect(result.current.tasks[0].status).toBe('active')
      expect(result.current.loading).toBe(false)
      expect(result.current.error).toBeNull()
    })

    it('復元エラーが発生した場合、エラーメッセージが設定される', async () => {
      const errorMessage = 'タスクの復元に失敗しました'
      mockInvoke.mockRejectedValue(new Error(errorMessage))

      const { result } = renderHook(() => useTasks())

      await act(async () => {
        try {
          await result.current.restoreTask({ id: 1 })
        } catch (err) {
          // エラーは期待される動作
        }
      })

      expect(result.current.error).toBe(errorMessage)
      expect(result.current.loading).toBe(false)
    })
  })

  describe('loadTasks', () => {
    it('正常にタスクを読み込める', async () => {
      mockInvoke.mockResolvedValue(mockTasks)

      const { result } = renderHook(() => useTasks())

      await act(async () => {
        await result.current.loadTasks(1)
      })

      expect(mockInvoke).toHaveBeenCalledWith('get_tasks_by_project', {
        projectId: 1,
        date: expect.any(String)
      })

      expect(result.current.tasks).toEqual(mockTasks)
      expect(result.current.loading).toBe(false)
      expect(result.current.error).toBeNull()
    })

    it('読み込み中はローディング状態になる', async () => {
      // 遅延をシミュレート
      mockInvoke.mockImplementation(() => new Promise(resolve => setTimeout(() => resolve(mockTasks), 100)))

      const { result } = renderHook(() => useTasks())

      act(() => {
        result.current.loadTasks(1)
      })

      expect(result.current.loading).toBe(true)

      await waitFor(() => {
        expect(result.current.loading).toBe(false)
      })
    })

    it('読み込みエラーが発生した場合、エラーメッセージが設定される', async () => {
      const errorMessage = 'タスクの読み込みに失敗しました'
      mockInvoke.mockRejectedValue(new Error(errorMessage))

      const { result } = renderHook(() => useTasks())

      await act(async () => {
        try {
          await result.current.loadTasks(1)
        } catch (err) {
          // エラーは期待される動作
        }
      })

      expect(result.current.error).toBe(errorMessage)
      expect(result.current.loading).toBe(false)
      expect(result.current.tasks).toEqual([])
    })
  })

  describe('clearError', () => {
    it('エラーメッセージをクリアできる', async () => {
      const { result } = renderHook(() => useTasks())

      // エラーを設定（エラーを発生させる）
      mockInvoke.mockRejectedValueOnce(new Error('テストエラー'))
      
      await act(async () => {
        try {
          await result.current.createTask({
            project_id: 1,
            name: 'テストタスク'
          })
        } catch (err) {
          // エラーは期待される動作
        }
      })

      expect(result.current.error).toBe('テストエラー')

      // エラーをクリア
      act(() => {
        result.current.clearError()
      })

      expect(result.current.error).toBeNull()
    })
  })

  describe('エラーハンドリング', () => {
    it('ネットワークエラーの場合、適切なエラーメッセージが設定される', async () => {
      mockInvoke.mockRejectedValue(new Error('Network error'))

      const { result } = renderHook(() => useTasks())

      await act(async () => {
        try {
          await result.current.createTask({
            project_id: 1,
            name: '新しいタスク'
          })
        } catch (err) {
          // エラーは期待される動作
        }
      })

      await waitFor(() => {
        expect(result.current.error).toBe('Network error')
      })
    })

    it('エラー発生後も他の操作が可能', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network error'))
      mockInvoke.mockResolvedValueOnce(mockTask)

      const { result } = renderHook(() => useTasks())

      // 最初の操作でエラー
      await act(async () => {
        try {
          await result.current.createTask({
            project_id: 1,
            name: 'エラーになるタスク'
          })
        } catch (err) {
          // エラーは期待される動作
        }
      })

      await waitFor(() => {
        expect(result.current.error).toBe('Network error')
      })

      // エラーをクリア
      act(() => {
        result.current.clearError()
      })

      // 次の操作は成功
      await act(async () => {
        await result.current.createTask({
          project_id: 1,
          name: '成功するタスク'
        })
      })

      expect(result.current.error).toBeNull()
      expect(result.current.tasks).toHaveLength(1)
    })
  })

  describe('状態管理', () => {
    it('複数の操作が連続で実行されても状態が正しく管理される', async () => {
      const { result } = renderHook(() => useTasks())

      // 複数のタスクを作成
      await act(async () => {
        await result.current.createTask({
          project_id: 1,
          name: 'タスク1'
        })
        await result.current.createTask({
          project_id: 1,
          name: 'タスク2'
        })
      })

      expect(result.current.tasks).toHaveLength(2)
      expect(result.current.loading).toBe(false)
      expect(result.current.error).toBeNull()
    })

    it('ローディング状態が適切に管理される', async () => {
      // 遅延をシミュレート
      mockInvoke.mockImplementation(() => new Promise(resolve => setTimeout(() => resolve(mockTask), 50)))

      const { result } = renderHook(() => useTasks())

      // 非同期処理を開始
      act(() => {
        result.current.createTask({
          project_id: 1,
          name: '新しいタスク'
        })
      })

      // ローディング中
      await waitFor(() => {
        expect(result.current.loading).toBe(true)
      })

      // 完了を待機
      await waitFor(() => {
        expect(result.current.loading).toBe(false)
      })

      // ローディング完了
      expect(result.current.loading).toBe(false)
    })
  })

  describe('loadTasks', () => {
    it('正常にタスク一覧を読み込める', async () => {
      mockInvoke.mockResolvedValue(mockTasks)
      const { result } = renderHook(() => useTasks())

      await act(async () => {
        await result.current.loadTasks(1)
      })

      expect(mockInvoke).toHaveBeenCalledWith('get_tasks_by_project', {
        projectId: 1,
        date: expect.any(String)
      })

      expect(result.current.tasks).toEqual(mockTasks)
      expect(result.current.loading).toBe(false)
      expect(result.current.error).toBeNull()
    })

    it('タスク読み込み中はローディング状態になる', async () => {
      mockInvoke.mockImplementation(() => new Promise(resolve => setTimeout(() => resolve(mockTasks), 100)))
      const { result } = renderHook(() => useTasks())

      act(() => {
        result.current.loadTasks(1)
      })

      expect(result.current.loading).toBe(true)

      await waitFor(() => {
        expect(result.current.loading).toBe(false)
      })
    })

    it('エラーが発生した場合、エラーメッセージが設定される', async () => {
      const errorMessage = 'タスクの取得に失敗しました'
      mockInvoke.mockRejectedValue(new Error(errorMessage))

      const { result } = renderHook(() => useTasks())

      await act(async () => {
        try {
          await result.current.loadTasks(1)
        } catch (err) {
          // エラーは期待される動作
        }
      })

      expect(result.current.error).toBe(errorMessage)
      expect(result.current.loading).toBe(false)
    })

    it('プロジェクトIDがnullの場合、タスクをクリアする', async () => {
      const { result } = renderHook(() => useTasks())

      // 最初にタスクを設定
      await act(async () => {
        result.current.tasks = mockTasks
      })

      await act(async () => {
        await result.current.loadTasks(null)
      })

      expect(result.current.tasks).toEqual([])
      expect(mockInvoke).not.toHaveBeenCalled()
    })
  })

  describe('日付ナビゲーション', () => {
    it('初期状態で今日の日付が設定される', () => {
      const { result } = renderHook(() => useTasks())
      const today = new Date()
      
      expect(result.current.selectedDate.toDateString()).toBe(today.toDateString())
    })

    it('前日に移動できる', () => {
      const { result } = renderHook(() => useTasks())
      const initialDate = new Date(result.current.selectedDate)
      
      act(() => {
        result.current.goToPreviousDay()
      })

      const expectedDate = new Date(initialDate)
      expectedDate.setDate(expectedDate.getDate() - 1)
      
      expect(result.current.selectedDate.toDateString()).toBe(expectedDate.toDateString())
    })

    it('翌日に移動できる（今日まで）', () => {
      const { result } = renderHook(() => useTasks())
      const initialDate = new Date(result.current.selectedDate)
      
      act(() => {
        result.current.goToNextDay()
      })

      const expectedDate = new Date(initialDate)
      expectedDate.setDate(expectedDate.getDate() + 1)
      
      // 今日の日付を取得して比較
      const today = new Date()
      today.setHours(23, 59, 59, 999)
      
      if (expectedDate <= today) {
        expect(result.current.selectedDate.toDateString()).toBe(expectedDate.toDateString())
      } else {
        // 未来の日付には移動できないため、元の日付のまま
        expect(result.current.selectedDate.toDateString()).toBe(initialDate.toDateString())
      }
    })

    it.skip('未来の日付には移動できない', () => {
      const { result } = renderHook(() => useTasks())
      
      // 今日の日付に設定
      const today = new Date()
      act(() => {
        result.current.setSelectedDate(today)
      })

      // 明日に移動を試行
      act(() => {
        result.current.goToNextDay()
      })

      // 明日に移動できる
      const tomorrow = new Date(today)
      tomorrow.setDate(tomorrow.getDate() + 1)
      expect(result.current.selectedDate.toDateString()).toBe(tomorrow.toDateString())

      // さらに翌日に移動を試行
      act(() => {
        result.current.goToNextDay()
      })

      // 明後日には移動できないため、明日のまま
      expect(result.current.selectedDate.toDateString()).toBe(tomorrow.toDateString())
    })

    it('日付を直接設定できる', () => {
      const { result } = renderHook(() => useTasks())
      const targetDate = new Date('2025-01-15')
      
      act(() => {
        result.current.setSelectedDate(targetDate)
      })

      expect(result.current.selectedDate.toDateString()).toBe(targetDate.toDateString())
    })
  })
})

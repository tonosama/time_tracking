import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { TaskList } from './TaskList'
import { CreateTaskModal } from './CreateTaskModal'
import { EditTaskModal } from './EditTaskModal'
import type { Task, Project } from '@/types'

// モックデータ
const mockProject: Project = {
  id: 1,
  name: 'テストプロジェクト',
  status: 'active',
  effective_at: '2025-01-01T00:00:00Z'
}

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

describe('TaskManagement', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // デフォルトのモック設定
    vi.mocked(invoke).mockImplementation(async (command: string, params?: any) => {
      switch (command) {
        case 'create_task':
          return {
            id: params.request.project_id === 99999 ? 999 : 1, // 存在しないプロジェクトIDの場合は別のID
            project_id: params.request.project_id,
            name: params.request.name,
            status: 'active',
            effective_at: '2025-01-01T00:00:00Z'
          }
        case 'update_task':
          return {
            id: params.request.id,
            project_id: params.request.project_id || mockTask.project_id,
            name: params.request.name,
            status: 'active',
            effective_at: '2025-01-01T00:00:00Z'
          }
        case 'archive_task':
          return {}
        case 'restore_task':
          return {
            id: params.request.id,
            project_id: mockTask.project_id,
            name: mockTask.name,
            status: 'active',
            effective_at: '2025-01-01T00:00:00Z'
          }
        case 'get_all_active_projects':
          return [mockProject]
        default:
          throw new Error(`Unknown command: ${command}`)
      }
    })
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  describe('TaskList', () => {
    it('タスク一覧を正常に表示する', () => {
      const mockOnTaskUpdate = vi.fn()
      const mockOnTaskArchived = vi.fn()

      render(
        <TaskList
          tasks={mockTasks}
          onTaskUpdate={mockOnTaskUpdate}
          onTaskArchived={mockOnTaskArchived}
        />
      )

      expect(screen.getByText('テストタスク')).toBeInTheDocument()
      expect(screen.getByText('テストタスク2')).toBeInTheDocument()
      expect(screen.getAllByTestId('task-item')).toHaveLength(2)
    })

    it('タスクが空の場合に適切なメッセージを表示する', () => {
      render(<TaskList tasks={[]} />)

      expect(screen.getByText('まだタスクがありません。新しいタスクを作成してみましょう。')).toBeInTheDocument()
    })

    it('タスクのアーカイブ機能が正常に動作する', async () => {
      const mockOnTaskArchived = vi.fn()

      render(
        <TaskList
          tasks={[mockTask]}
          onTaskArchived={mockOnTaskArchived}
        />
      )

      const archiveButton = screen.getByRole('button', { name: 'アーカイブ' })
      fireEvent.click(archiveButton)

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('archive_task', {
          request: { id: mockTask.id }
        })
      })

      expect(mockOnTaskArchived).toHaveBeenCalledWith(mockTask.id)
    })

    it('タスクの復元機能が正常に動作する', async () => {
      const archivedTask: Task = { ...mockTask, status: 'archived' }
      const mockOnTaskUpdate = vi.fn()

      render(
        <TaskList
          tasks={[archivedTask]}
          onTaskUpdate={mockOnTaskUpdate}
        />
      )

      const restoreButton = screen.getByRole('button', { name: '復元' })
      fireEvent.click(restoreButton)

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('restore_task', {
          request: { id: archivedTask.id }
        })
      })

      await waitFor(() => {
        expect(mockOnTaskUpdate).toHaveBeenCalled()
      })
    })

    it('タスクの編集モーダルが正常に開く', () => {
      const mockOnTaskUpdate = vi.fn()

      render(
        <TaskList
          tasks={[mockTask]}
          onTaskUpdate={mockOnTaskUpdate}
        />
      )

      const editButton = screen.getByRole('button', { name: '編集' })
      fireEvent.click(editButton)

      // 編集モーダルが表示されることを確認
      expect(screen.getByText('タスクを編集')).toBeInTheDocument()
      expect(screen.getByDisplayValue('テストタスク')).toBeInTheDocument()
    })
  })

  describe('CreateTaskModal', () => {
    it('タスク作成モーダルが正常に表示される', () => {
      const mockOnClose = vi.fn()
      const mockOnTaskCreated = vi.fn()

      render(
        <CreateTaskModal
          isOpen={true}
          projectId={1}
          onClose={mockOnClose}
          onTaskCreated={mockOnTaskCreated}
        />
      )

      expect(screen.getByText('新しいタスク')).toBeInTheDocument()
      expect(screen.getByLabelText('タスク名')).toBeInTheDocument()
      expect(screen.getByPlaceholderText('タスク名を入力')).toBeInTheDocument()
      expect(screen.getByRole('button', { name: '作成' })).toBeInTheDocument()
      expect(screen.getByRole('button', { name: 'キャンセル' })).toBeInTheDocument()
    })

    it('タスク作成が正常に動作する', async () => {
      const mockOnClose = vi.fn()
      const mockOnTaskCreated = vi.fn()

      render(
        <CreateTaskModal
          isOpen={true}
          projectId={1}
          onClose={mockOnClose}
          onTaskCreated={mockOnTaskCreated}
        />
      )

      // タスク名を入力
      const input = screen.getByLabelText('タスク名')
      fireEvent.change(input, { target: { value: '新しいタスク' } })

      // 作成ボタンをクリック
      const createButton = screen.getByRole('button', { name: '作成' })
      fireEvent.click(createButton)

      // APIが呼ばれることを確認
      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('create_task', {
          request: {
            project_id: 1,
            name: '新しいタスク'
          }
        })
      })

      // コールバックが呼ばれることを確認
      await waitFor(() => {
        expect(mockOnTaskCreated).toHaveBeenCalledWith({
          id: 1,
          project_id: 1,
          name: '新しいタスク',
          status: 'active',
          effective_at: '2025-01-01T00:00:00Z'
        })
      })

      // モーダルが閉じられることを確認
      expect(mockOnClose).toHaveBeenCalled()
    })

    it('空のタスク名で作成ボタンを押すとエラーメッセージが表示される', () => {
      const mockOnClose = vi.fn()
      const mockOnTaskCreated = vi.fn()

      render(
        <CreateTaskModal
          isOpen={true}
          projectId={1}
          onClose={mockOnClose}
          onTaskCreated={mockOnTaskCreated}
        />
      )

      // 作成ボタンをクリック（タスク名は空）
      const createButton = screen.getByRole('button', { name: '作成' })
      fireEvent.click(createButton)

      // エラーメッセージが表示されることを確認
      expect(screen.getByText('タスク名を入力してください')).toBeInTheDocument()

      // APIは呼ばれないことを確認
      expect(invoke).not.toHaveBeenCalled()
    })

    it('存在しないプロジェクトでタスク作成を試行するとエラーが発生する', async () => {
      const mockOnClose = vi.fn()
      const mockOnTaskCreated = vi.fn()

      // 存在しないプロジェクトIDでモックを設定
      vi.mocked(invoke).mockImplementation(async (command: string, params?: any) => {
        if (command === 'create_task' && params.request.project_id === 99999) {
          throw new Error('Project not found')
        }
        return {
          id: 1,
          project_id: params.request.project_id,
          name: params.request.name,
          status: 'active',
          effective_at: '2025-01-01T00:00:00Z'
        }
      })

      render(
        <CreateTaskModal
          isOpen={true}
          projectId={99999}
          onClose={mockOnClose}
          onTaskCreated={mockOnTaskCreated}
        />
      )

      // タスク名を入力
      const input = screen.getByLabelText('タスク名')
      fireEvent.change(input, { target: { value: 'エラーテストタスク' } })

      // 作成ボタンをクリック
      const createButton = screen.getByRole('button', { name: '作成' })
      fireEvent.click(createButton)

      // エラーメッセージが表示されることを確認
      await waitFor(() => {
        expect(screen.getByText('Project not found')).toBeInTheDocument()
      })

      // モーダルは閉じられないことを確認
      expect(mockOnClose).not.toHaveBeenCalled()
    })
  })

  describe('EditTaskModal', () => {
    it('タスク編集モーダルが正常に表示される', () => {
      const mockOnClose = vi.fn()
      const mockOnTaskUpdated = vi.fn()

      render(
        <EditTaskModal
          isOpen={true}
          task={mockTask}
          onClose={mockOnClose}
          onTaskUpdated={mockOnTaskUpdated}
        />
      )

      expect(screen.getByText('タスクを編集')).toBeInTheDocument()
      expect(screen.getByDisplayValue('テストタスク')).toBeInTheDocument()
      expect(screen.getByRole('button', { name: '保存' })).toBeInTheDocument()
      expect(screen.getByRole('button', { name: 'キャンセル' })).toBeInTheDocument()
    })

    it('タスク編集が正常に動作する', async () => {
      const mockOnClose = vi.fn()
      const mockOnTaskUpdated = vi.fn()

      render(
        <EditTaskModal
          isOpen={true}
          task={mockTask}
          onClose={mockOnClose}
          onTaskUpdated={mockOnTaskUpdated}
        />
      )

      // タスク名を変更
      const input = screen.getByDisplayValue('テストタスク')
      fireEvent.change(input, { target: { value: '編集されたタスク' } })

      // 保存ボタンをクリック
      const saveButton = screen.getByRole('button', { name: '保存' })
      fireEvent.click(saveButton)

      // APIが呼ばれることを確認
      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('update_task', {
          request: {
            id: mockTask.id,
            name: '編集されたタスク'
          }
        })
      })

      // コールバックが呼ばれることを確認
      await waitFor(() => {
        expect(mockOnTaskUpdated).toHaveBeenCalledWith({
          id: mockTask.id,
          project_id: mockTask.project_id,
          name: '編集されたタスク',
          status: 'active',
          effective_at: '2025-01-01T00:00:00Z'
        })
      })

      // モーダルが閉じられることを確認
      expect(mockOnClose).toHaveBeenCalled()
    })

    it('プロジェクト変更が正常に動作する', async () => {
      const mockOnClose = vi.fn()
      const mockOnTaskUpdated = vi.fn()

      render(
        <EditTaskModal
          isOpen={true}
          task={mockTask}
          onClose={mockOnClose}
          onTaskUpdated={mockOnTaskUpdated}
        />
      )

      // プロジェクトを変更
      const projectSelect = screen.getByLabelText('プロジェクト')
      fireEvent.change(projectSelect, { target: { value: '2' } })

      // 保存ボタンをクリック
      const saveButton = screen.getByRole('button', { name: '保存' })
      fireEvent.click(saveButton)

      // APIが呼ばれることを確認
      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('update_task', {
          request: {
            id: mockTask.id,
            name: mockTask.name,
            project_id: 2
          }
        })
      })

      // コールバックが呼ばれることを確認
      await waitFor(() => {
        expect(mockOnTaskUpdated).toHaveBeenCalled()
      })
    })

    it('空のタスク名で保存ボタンを押すとエラーメッセージが表示される', () => {
      const mockOnClose = vi.fn()
      const mockOnTaskUpdated = vi.fn()

      render(
        <EditTaskModal
          isOpen={true}
          task={mockTask}
          onClose={mockOnClose}
          onTaskUpdated={mockOnTaskUpdated}
        />
      )

      // タスク名を空にする
      const input = screen.getByDisplayValue('テストタスク')
      fireEvent.change(input, { target: { value: '' } })

      // 保存ボタンをクリック
      const saveButton = screen.getByRole('button', { name: '保存' })
      fireEvent.click(saveButton)

      // エラーメッセージが表示されることを確認
      expect(screen.getByText('タスク名を入力してください')).toBeInTheDocument()

      // APIは呼ばれないことを確認
      expect(invoke).not.toHaveBeenCalled()
    })
  })

  describe('統合テスト', () => {
    it('タスクの作成→編集→アーカイブ→復元の一連の流れが正常に動作する', async () => {
      const mockOnTaskUpdate = vi.fn()
      const mockOnTaskArchived = vi.fn()

      // タスク作成
      const { rerender } = render(
        <TaskList
          tasks={[]}
          onTaskUpdate={mockOnTaskUpdate}
          onTaskArchived={mockOnTaskArchived}
        />
      )

      // タスクが作成された状態をシミュレート
      rerender(
        <TaskList
          tasks={[mockTask]}
          onTaskUpdate={mockOnTaskUpdate}
          onTaskArchived={mockOnTaskArchived}
        />
      )

      expect(screen.getByText('テストタスク')).toBeInTheDocument()

      // タスク編集
      const editButton = screen.getByRole('button', { name: '編集' })
      fireEvent.click(editButton)

      const nameInput = screen.getByDisplayValue('テストタスク')
      fireEvent.change(nameInput, { target: { value: '編集されたタスク' } })

      const saveButton = screen.getByRole('button', { name: '保存' })
      fireEvent.click(saveButton)

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('update_task', {
          request: {
            id: mockTask.id,
            name: '編集されたタスク'
          }
        })
      })

      // タスクアーカイブ
      const archiveButton = screen.getByRole('button', { name: 'アーカイブ' })
      fireEvent.click(archiveButton)

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('archive_task', {
          request: { id: mockTask.id }
        })
      })

      // アーカイブされたタスクを表示
      const archivedTask: Task = { ...mockTask, status: 'archived', name: '編集されたタスク' }
      rerender(
        <TaskList
          tasks={[archivedTask]}
          onTaskUpdate={mockOnTaskUpdate}
          onTaskArchived={mockOnTaskArchived}
        />
      )

      // タスク復元
      const restoreButton = screen.getByRole('button', { name: '復元' })
      fireEvent.click(restoreButton)

      await waitFor(() => {
        expect(invoke).toHaveBeenCalledWith('restore_task', {
          request: { id: mockTask.id }
        })
      })

      // すべてのAPI呼び出しが正しく行われたことを確認
      expect(invoke).toHaveBeenCalledTimes(3)
    })
  })
})

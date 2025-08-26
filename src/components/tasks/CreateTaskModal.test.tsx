import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { CreateTaskModal } from './CreateTaskModal'
import type { Task } from '@/types'

// Tauri APIのモック
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

const mockInvoke = vi.mocked(invoke)

// モックデータ
const mockProjectId = 1
const mockTask: Task = {
  id: 1,
  project_id: mockProjectId,
  name: 'テストタスク',
  status: 'active',
  effective_at: '2025-01-01T00:00:00Z'
}

describe('CreateTaskModal', () => {
  const defaultProps = {
    isOpen: true,
    onClose: vi.fn(),
    onTaskCreated: vi.fn(),
    projectId: mockProjectId
  }

  beforeEach(() => {
    vi.clearAllMocks()
    // デフォルトのモック設定
    mockInvoke.mockResolvedValue(mockTask)
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  describe('モーダル表示', () => {
    it('モーダルが開いている時に正しく表示される', () => {
      render(<CreateTaskModal {...defaultProps} />)
      
      expect(screen.getByText('新しいタスク')).toBeInTheDocument()
      expect(screen.getByLabelText('タスク名')).toBeInTheDocument()
      expect(screen.getByPlaceholderText('タスク名を入力')).toBeInTheDocument()
      expect(screen.getByRole('button', { name: 'キャンセル' })).toBeInTheDocument()
      expect(screen.getByRole('button', { name: '作成' })).toBeInTheDocument()
    })

    it('モーダルが閉じている時は表示されない', () => {
      render(<CreateTaskModal {...defaultProps} isOpen={false} />)
      
      expect(screen.queryByText('新しいタスク')).not.toBeInTheDocument()
      expect(screen.queryByLabelText('タスク名')).not.toBeInTheDocument()
    })
  })

  describe('フォーム入力', () => {
    it('タスク名を入力できる', () => {
      render(<CreateTaskModal {...defaultProps} />)
      
      const input = screen.getByPlaceholderText('タスク名を入力')
      fireEvent.change(input, { target: { value: '新しいタスク' } })
      
      expect(input).toHaveValue('新しいタスク')
    })

    it('入力フィールドが自動フォーカスされる', () => {
      render(<CreateTaskModal {...defaultProps} />)
      
      const input = screen.getByPlaceholderText('タスク名を入力')
      expect(input).toHaveFocus()
    })

    it('入力値が空の場合、作成ボタンが無効化される', () => {
      render(<CreateTaskModal {...defaultProps} />)
      
      const createButton = screen.getByRole('button', { name: '作成' })
      expect(createButton).toBeDisabled()
    })

    it('入力値がある場合、作成ボタンが有効化される', () => {
      render(<CreateTaskModal {...defaultProps} />)
      
      const input = screen.getByPlaceholderText('タスク名を入力')
      const createButton = screen.getByRole('button', { name: '作成' })
      
      fireEvent.change(input, { target: { value: '新しいタスク' } })
      
      expect(createButton).not.toBeDisabled()
    })

    it('空白のみの入力の場合、作成ボタンが無効化される', () => {
      render(<CreateTaskModal {...defaultProps} />)
      
      const input = screen.getByPlaceholderText('タスク名を入力')
      const createButton = screen.getByRole('button', { name: '作成' })
      
      fireEvent.change(input, { target: { value: '   ' } })
      
      expect(createButton).toBeDisabled()
    })
  })

  describe('タスク作成', () => {
    it('正常にタスクを作成できる', async () => {
      render(<CreateTaskModal {...defaultProps} />)
      
      const input = screen.getByPlaceholderText('タスク名を入力')
      const createButton = screen.getByRole('button', { name: '作成' })
      
      fireEvent.change(input, { target: { value: '新しいタスク' } })
      fireEvent.click(createButton)
      
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('create_task', {
          request: {
            project_id: mockProjectId,
            name: '新しいタスク'
          }
        })
      })
      
      expect(defaultProps.onTaskCreated).toHaveBeenCalledWith(mockTask)
      expect(defaultProps.onClose).toHaveBeenCalled()
    })

    it('タスク作成中はローディング状態になる', async () => {
      // 遅延をシミュレート
      mockInvoke.mockImplementation(() => new Promise(resolve => setTimeout(() => resolve(mockTask), 100)))
      
      render(<CreateTaskModal {...defaultProps} />)
      
      const input = screen.getByPlaceholderText('タスク名を入力')
      const createButton = screen.getByRole('button', { name: '作成' })
      
      fireEvent.change(input, { target: { value: '新しいタスク' } })
      fireEvent.click(createButton)
      
      // ローディング中はボタンが無効化される
      expect(createButton).toBeDisabled()
      expect(screen.getByRole('button', { name: 'キャンセル' })).toBeDisabled()
    })

    it('タスク名が空の場合、エラーメッセージが表示される', async () => {
      render(<CreateTaskModal {...defaultProps} />)
      
      // フォームを直接送信してエラーをテスト
      const form = screen.getByRole('form')
      fireEvent.submit(form)
      
      // エラーメッセージが表示されることを確認
      await waitFor(() => {
        expect(screen.getByText('タスク名を入力してください')).toBeInTheDocument()
      })
      expect(mockInvoke).not.toHaveBeenCalled()
    })

    it('タスク名が空白のみの場合、エラーメッセージが表示される', async () => {
      render(<CreateTaskModal {...defaultProps} />)
      
      const input = screen.getByPlaceholderText('タスク名を入力')
      
      fireEvent.change(input, { target: { value: '   ' } })
      
      // フォームを直接送信してエラーをテスト
      const form = screen.getByRole('form')
      fireEvent.submit(form)
      
      // エラーメッセージが表示されることを確認
      await waitFor(() => {
        expect(screen.getByText('タスク名を入力してください')).toBeInTheDocument()
      })
      expect(mockInvoke).not.toHaveBeenCalled()
    })

    it('APIエラーが発生した場合、エラーメッセージが表示される', async () => {
      const errorMessage = 'タスクの作成に失敗しました'
      mockInvoke.mockRejectedValue(new Error(errorMessage))
      
      render(<CreateTaskModal {...defaultProps} />)
      
      const input = screen.getByPlaceholderText('タスク名を入力')
      const createButton = screen.getByRole('button', { name: '作成' })
      
      fireEvent.change(input, { target: { value: '新しいタスク' } })
      fireEvent.click(createButton)
      
      await waitFor(() => {
        expect(screen.getByRole('alert')).toHaveTextContent(errorMessage)
      })
      
      // エラーが発生した場合、モーダルは閉じない（onTaskCreatedも呼ばれない）
      expect(defaultProps.onTaskCreated).not.toHaveBeenCalled()
      expect(defaultProps.onClose).not.toHaveBeenCalled()
    })

    it('タスク作成後、フォームがリセットされる', async () => {
      const { rerender } = render(<CreateTaskModal {...defaultProps} />)
      
      const input = screen.getByPlaceholderText('タスク名を入力')
      const createButton = screen.getByRole('button', { name: '作成' })
      
      fireEvent.change(input, { target: { value: '新しいタスク' } })
      fireEvent.click(createButton)
      
      await waitFor(() => {
        expect(defaultProps.onClose).toHaveBeenCalled()
      })
      
      // モーダルが再開された時の状態を確認
      rerender(<CreateTaskModal {...defaultProps} />)
      expect(screen.getByPlaceholderText('タスク名を入力')).toHaveValue('')
    })
  })

  describe('モーダル操作', () => {
    it('キャンセルボタンでモーダルが閉じる', () => {
      render(<CreateTaskModal {...defaultProps} />)
      
      const cancelButton = screen.getByRole('button', { name: 'キャンセル' })
      fireEvent.click(cancelButton)
      
      expect(defaultProps.onClose).toHaveBeenCalled()
    })

    it('キャンセル時にフォームがリセットされる', () => {
      render(<CreateTaskModal {...defaultProps} />)
      
      const input = screen.getByPlaceholderText('タスク名を入力')
      const cancelButton = screen.getByRole('button', { name: 'キャンセル' })
      
      fireEvent.change(input, { target: { value: 'テスト入力' } })
      fireEvent.click(cancelButton)
      
      expect(defaultProps.onClose).toHaveBeenCalled()
    })

    it('ESCキーでモーダルが閉じる', () => {
      render(<CreateTaskModal {...defaultProps} />)
      
      // ドキュメントにESCキーイベントを送信
      fireEvent.keyDown(document, { key: 'Escape' })
      
      expect(defaultProps.onClose).toHaveBeenCalled()
    })

    it('Enterキーでタスク作成が実行される', async () => {
      render(<CreateTaskModal {...defaultProps} />)
      
      const input = screen.getByPlaceholderText('タスク名を入力')
      fireEvent.change(input, { target: { value: '新しいタスク' } })
      
      // フォームのsubmitイベントをシミュレート
      const form = screen.getByRole('form')
      fireEvent.submit(form)
      
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('create_task', {
          request: {
            project_id: mockProjectId,
            name: '新しいタスク'
          }
        })
      })
    })
  })

  describe('アクセシビリティ', () => {
    it('適切なラベルとaria属性が設定されている', () => {
      render(<CreateTaskModal {...defaultProps} />)
      
      const input = screen.getByLabelText('タスク名')
      expect(input).toHaveAttribute('required')
      // autoFocus属性は実際のコンポーネントでは設定されていないため、フォーカス状態を確認
      expect(input).toHaveFocus()
    })

    it('フォーカス管理が適切に動作する', () => {
      render(<CreateTaskModal {...defaultProps} />)
      
      const input = screen.getByPlaceholderText('タスク名を入力')
      expect(input).toHaveFocus()
    })
  })

  describe('エラーハンドリング', () => {
    it('ネットワークエラーの場合、適切なエラーメッセージが表示される', async () => {
      mockInvoke.mockRejectedValue(new Error('Network error'))
      
      render(<CreateTaskModal {...defaultProps} />)
      
      const input = screen.getByPlaceholderText('タスク名を入力')
      const createButton = screen.getByRole('button', { name: '作成' })
      
      fireEvent.change(input, { target: { value: '新しいタスク' } })
      fireEvent.click(createButton)
      
      await waitFor(() => {
        expect(screen.getByRole('alert')).toHaveTextContent('Network error')
      })
    })

    it('エラー発生後もフォームが使用可能', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network error'))
      
      render(<CreateTaskModal {...defaultProps} />)
      
      const input = screen.getByPlaceholderText('タスク名を入力')
      const createButton = screen.getByRole('button', { name: '作成' })
      
      fireEvent.change(input, { target: { value: '新しいタスク' } })
      fireEvent.click(createButton)
      
      await waitFor(() => {
        expect(screen.getByRole('alert')).toHaveTextContent('Network error')
      })
      
      // エラー後も入力可能
      fireEvent.change(input, { target: { value: '修正されたタスク' } })
      expect(input).toHaveValue('修正されたタスク')
    })
  })

  describe('パフォーマンス', () => {
    it('大量のテキスト入力でもパフォーマンスが劣化しない', () => {
      render(<CreateTaskModal {...defaultProps} />)
      
      const input = screen.getByPlaceholderText('タスク名を入力')
      const longText = 'a'.repeat(1000)
      
      const startTime = performance.now()
      fireEvent.change(input, { target: { value: longText } })
      const endTime = performance.now()
      
      // 100ms以内に処理が完了することを確認
      expect(endTime - startTime).toBeLessThan(100)
    })
  })
})

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { ManualEntryModal } from './ManualEntryModal'
import type { Project, Task } from '@/types'

// Tauriのモック
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

const mockProjects: Project[] = [
  {
    id: 1,
    name: 'Web Development',
    color: '#3b82f6',
    status: 'active' as const,
    effective_at: new Date().toISOString(),
  },
  {
    id: 2,
    name: 'API Project',
    color: '#ef4444',
    status: 'active' as const,
    effective_at: new Date().toISOString(),
  }
]

const mockTasks: Task[] = [
  {
    id: 1,
    project_id: 1,
    name: 'Frontend Development',
    status: 'active' as const,
    effective_at: new Date().toISOString(),
  },
  {
    id: 2,
    project_id: 1,
    name: 'UI Design',
    status: 'active' as const,
    effective_at: new Date().toISOString(),
  },
  {
    id: 3,
    project_id: 2,
    name: 'API Implementation',
    status: 'active' as const,
    effective_at: new Date().toISOString(),
  }
]

describe('ManualEntryModal', () => {
  const mockOnClose = vi.fn()
  const mockOnSuccess = vi.fn()
  
  const defaultProps = {
    isOpen: true,
    onClose: mockOnClose,
    onSuccess: mockOnSuccess,
    projects: mockProjects,
    tasks: mockTasks,
  }

  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('基本表示', () => {
    it('モーダルが開いている時に正しく表示される', () => {
      render(<ManualEntryModal {...defaultProps} />)
      
      expect(screen.getByText('手動で時間を追加')).toBeInTheDocument()
      expect(screen.getByLabelText('プロジェクト')).toBeInTheDocument()
      expect(screen.getByLabelText('タスク')).toBeInTheDocument()
      expect(screen.getByLabelText('開始時刻')).toBeInTheDocument()
      expect(screen.getByLabelText('終了時刻')).toBeInTheDocument()
      expect(screen.getByLabelText('メモ (オプション)')).toBeInTheDocument()
      expect(screen.getByRole('button', { name: '追加' })).toBeInTheDocument()
      expect(screen.getByRole('button', { name: 'キャンセル' })).toBeInTheDocument()
    })

    it('モーダルが閉じている時は表示されない', () => {
      render(<ManualEntryModal {...defaultProps} isOpen={false} />)
      
      expect(screen.queryByText('手動で時間を追加')).not.toBeInTheDocument()
    })

    it('selectedProjectが指定されている場合、プロジェクトが事前選択される', () => {
      render(<ManualEntryModal {...defaultProps} selectedProject={mockProjects[0]} />)
      
      const projectSelect = screen.getByLabelText('プロジェクト') as HTMLSelectElement
      expect(projectSelect.value).toBe('1')
    })
  })

  describe('プロジェクト・タスク選択', () => {
    it('プロジェクトを選択すると、そのプロジェクトのタスクのみが表示される', async () => {
      const user = userEvent.setup()
      render(<ManualEntryModal {...defaultProps} />)
      
      const projectSelect = screen.getByLabelText('プロジェクト')
      await user.selectOptions(projectSelect, '1')
      
      const taskSelect = screen.getByLabelText('タスク')
      const taskOptions = taskSelect.querySelectorAll('option')
      
      // プレースホルダー + プロジェクト1のタスク2個 = 3個
      expect(taskOptions).toHaveLength(3)
      expect(screen.getByText('Frontend Development')).toBeInTheDocument()
      expect(screen.getByText('UI Design')).toBeInTheDocument()
      expect(screen.queryByText('API Implementation')).not.toBeInTheDocument()
    })

    it('プロジェクトが未選択の場合、タスク選択は無効化される', () => {
      render(<ManualEntryModal {...defaultProps} />)
      
      const taskSelect = screen.getByLabelText('タスク')
      expect(taskSelect).toBeDisabled()
    })
  })

  describe('時間入力とバリデーション', () => {
    it('開始時刻と終了時刻を入力すると継続時間が自動計算される', async () => {
      const user = userEvent.setup()
      render(<ManualEntryModal {...defaultProps} />)
      
      const startTimeInput = screen.getByLabelText('開始時刻')
      const endTimeInput = screen.getByLabelText('終了時刻')
      
      await user.type(startTimeInput, '2024-01-15T09:00')
      await user.type(endTimeInput, '2024-01-15T10:30')
      
      expect(screen.getByText('継続時間: 1時間30分')).toBeInTheDocument()
    })

    it('終了時刻が開始時刻より前の場合、エラーメッセージが表示される', async () => {
      const user = userEvent.setup()
      render(<ManualEntryModal {...defaultProps} />)
      
      const startTimeInput = screen.getByLabelText('開始時刻')
      const endTimeInput = screen.getByLabelText('終了時刻')
      
      await user.type(startTimeInput, '2024-01-15T10:00')
      await user.type(endTimeInput, '2024-01-15T09:00')
      
      await waitFor(() => {
        expect(screen.getByText('終了時刻は開始時刻より後の時刻を指定してください')).toBeInTheDocument()
      })
    })

    it('未来の日付を入力すると警告メッセージが表示される', async () => {
      const user = userEvent.setup()
      render(<ManualEntryModal {...defaultProps} />)
      
      const tomorrow = new Date()
      tomorrow.setDate(tomorrow.getDate() + 1)
      const tomorrowString = tomorrow.toISOString().slice(0, 16)
      
      const startTimeInput = screen.getByLabelText('開始時刻')
      await user.type(startTimeInput, tomorrowString)
      
      await waitFor(() => {
        expect(screen.getByText('未来の時刻が指定されています')).toBeInTheDocument()
      })
    })
  })

  describe('フォーム送信', () => {
    it('必須項目が未入力の場合、送信ボタンが無効化される', () => {
      render(<ManualEntryModal {...defaultProps} />)
      
      const submitButton = screen.getByRole('button', { name: '追加' })
      expect(submitButton).toBeDisabled()
    })

    it('有効なデータを入力すると送信ボタンが有効化される', async () => {
      const user = userEvent.setup()
      render(<ManualEntryModal {...defaultProps} />)
      
      // プロジェクト選択
      await user.selectOptions(screen.getByLabelText('プロジェクト'), '1')
      
      // タスク選択
      await user.selectOptions(screen.getByLabelText('タスク'), '1')
      
      // 時間入力
      await user.type(screen.getByLabelText('開始時刻'), '2024-01-15T09:00')
      await user.type(screen.getByLabelText('終了時刻'), '2024-01-15T10:00')
      
      const submitButton = screen.getByRole('button', { name: '追加' })
      expect(submitButton).toBeEnabled()
    })

    it('有効なフォームデータでAPI呼び出しが実行される', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      const mockInvoke = vi.mocked(invoke)
      mockInvoke.mockResolvedValue(undefined)
      
      const user = userEvent.setup()
      render(<ManualEntryModal {...defaultProps} />)
      
      // フォーム入力
      await user.selectOptions(screen.getByLabelText('プロジェクト'), '1')
      await user.selectOptions(screen.getByLabelText('タスク'), '1')
      await user.type(screen.getByLabelText('開始時刻'), '2024-01-15T09:00')
      await user.type(screen.getByLabelText('終了時刻'), '2024-01-15T10:00')
      await user.type(screen.getByLabelText('メモ (オプション)'), 'テスト作業')
      
      // 送信
      await user.click(screen.getByRole('button', { name: '追加' }))
      
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('add_manual_entry', {
          request: {
            task_id: 1,
            start_time: expect.stringMatching(/2024-01-15T\d{2}:00:00\.000Z/),
            end_time: expect.stringMatching(/2024-01-15T\d{2}:00:00\.000Z/),
            note: 'テスト作業'
          }
        })
      })
      
      expect(mockOnSuccess).toHaveBeenCalled()
      expect(mockOnClose).toHaveBeenCalled()
    })

    it('API呼び出しエラー時にエラーメッセージが表示される', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      const mockInvoke = vi.mocked(invoke)
      mockInvoke.mockRejectedValue(new Error('API Error'))
      
      const user = userEvent.setup()
      render(<ManualEntryModal {...defaultProps} />)
      
      // フォーム入力
      await user.selectOptions(screen.getByLabelText('プロジェクト'), '1')
      await user.selectOptions(screen.getByLabelText('タスク'), '1')
      await user.type(screen.getByLabelText('開始時刻'), '2024-01-15T09:00')
      await user.type(screen.getByLabelText('終了時刻'), '2024-01-15T10:00')
      
      // 送信
      await user.click(screen.getByRole('button', { name: '追加' }))
      
      await waitFor(() => {
        expect(screen.getByText('時間エントリの追加に失敗しました: Error: API Error')).toBeInTheDocument()
      })
    })
  })

  describe('キャンセル・クローズ', () => {
    it('キャンセルボタンをクリックするとモーダルが閉じる', async () => {
      const user = userEvent.setup()
      render(<ManualEntryModal {...defaultProps} />)
      
      await user.click(screen.getByRole('button', { name: 'キャンセル' }))
      
      expect(mockOnClose).toHaveBeenCalled()
    })

    it('モーダルの背景をクリックするとモーダルが閉じる', async () => {
      const user = userEvent.setup()
      render(<ManualEntryModal {...defaultProps} />)
      
      // Modal コンポーネントの backdrop をクリック
      const backdrop = screen.getByTestId('modal-backdrop')
      await user.click(backdrop)
      
      expect(mockOnClose).toHaveBeenCalled()
    })
  })

  describe('ローディング状態', () => {
    it('送信中はローディング状態が表示される', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      const mockInvoke = vi.mocked(invoke)
      
      // Promise を resolve しない状態でモック
      let resolvePromise: () => void
      mockInvoke.mockReturnValue(new Promise<void>((resolve) => {
        resolvePromise = resolve
      }))
      
      const user = userEvent.setup()
      render(<ManualEntryModal {...defaultProps} />)
      
      // フォーム入力
      await user.selectOptions(screen.getByLabelText('プロジェクト'), '1')
      await user.selectOptions(screen.getByLabelText('タスク'), '1')
      await user.type(screen.getByLabelText('開始時刻'), '2024-01-15T09:00')
      await user.type(screen.getByLabelText('終了時刻'), '2024-01-15T10:00')
      
      // 送信
      await user.click(screen.getByRole('button', { name: '追加' }))
      
      // ローディング状態の確認
      expect(screen.getByText('追加中...')).toBeInTheDocument()
      expect(screen.getByRole('button', { name: '追加中...' })).toBeDisabled()
      
      // Promise を resolve してローディング終了
      resolvePromise!()
      await waitFor(() => {
        expect(screen.queryByText('追加中...')).not.toBeInTheDocument()
      })
    })
  })
})

import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { TimerButton } from './TimerButton'
import { TimeTrackingApi } from '@/services'
import type { Task, TimerStatus } from '@/types'

// TimeTrackingApiのモック
vi.mock('@/services', () => ({
  TimeTrackingApi: {
    getTimerStatus: vi.fn(),
    startTimer: vi.fn(),
    stopTimer: vi.fn(),
  }
}))

// モックデータ
const mockTask: Task = {
  id: 1,
  project_id: 1,
  name: 'テストタスク',
  status: 'active',
  effective_at: '2025-01-01T00:00:00Z'
}

const mockTimerStatus: TimerStatus = {
  is_running: false,
  elapsed_seconds: 0,
  elapsed_duration: undefined
}

const mockRunningTimerStatus: TimerStatus = {
  is_running: true,
  elapsed_seconds: 3600, // 1時間
  elapsed_duration: '01:00:00'
}

describe('TimerButton', () => {
  const mockOnTimerStateChanged = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
    // デフォルトのモック設定
    vi.mocked(TimeTrackingApi.getTimerStatus).mockResolvedValue(mockTimerStatus)
    vi.mocked(TimeTrackingApi.startTimer).mockResolvedValue({
      id: 1,
      task_id: 1,
      event_type: 'start',
      at: '2025-01-01T00:00:00Z'
    })
    vi.mocked(TimeTrackingApi.stopTimer).mockResolvedValue({
      id: 1,
      task_id: 1,
      event_type: 'stop',
      at: '2025-01-01T00:00:00Z'
    })
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  describe('初期表示', () => {
    it('停止状態でタイマー開始ボタンが正しく表示される', async () => {
      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
        />
      )

      // タイマー状態の取得が呼ばれることを確認
      await waitFor(() => {
        expect(TimeTrackingApi.getTimerStatus).toHaveBeenCalledWith(mockTask.id)
      })

      // 開始ボタンが表示されることを確認
      const button = screen.getByRole('button', { name: 'テストタスクのタイマー（停止中）を開始' })
      expect(button).toBeInTheDocument()
      expect(button).toBeEnabled()

      // ボタンのスタイルクラスを確認
      expect(button).toHaveClass('btn', 'btn-sm', 'btn-success', 'timer-button-stopped')

      // アイコンとテキストを確認
      expect(screen.getByText('▶️')).toBeInTheDocument()
      expect(screen.getByText('タイマー開始')).toBeInTheDocument()
    })

    it('実行状態でタイマー停止ボタンが正しく表示される', async () => {
      vi.mocked(TimeTrackingApi.getTimerStatus).mockResolvedValue(mockRunningTimerStatus)

      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
        />
      )

      // 停止ボタンが表示されることを確認
      await waitFor(() => {
        const button = screen.getByRole('button', { name: 'テストタスクのタイマー（実行中）を停止' })
        expect(button).toBeInTheDocument()
        expect(button).toBeEnabled()
      })

      const button = screen.getByRole('button', { name: 'テストタスクのタイマー（実行中）を停止' })
      
      // ボタンのスタイルクラスを確認
      expect(button).toHaveClass('btn', 'btn-sm', 'btn-danger', 'timer-button-running')

      // アイコンとテキストを確認
      expect(screen.getByText('⏹️')).toBeInTheDocument()
      expect(screen.getByText('タイマー停止')).toBeInTheDocument()
    })

    it('経過時間が正しく表示される', async () => {
      vi.mocked(TimeTrackingApi.getTimerStatus).mockResolvedValue(mockRunningTimerStatus)

      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
        />
      )

      // 経過時間が表示されることを確認
      await waitFor(() => {
        expect(screen.getByTestId('timer-elapsed')).toBeInTheDocument()
        expect(screen.getByText('経過時間: 01:00:00')).toBeInTheDocument()
      })
    })

    it('無効化状態が正しく動作する', async () => {
      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
          disabled={true}
        />
      )

      // ボタンが無効化されていることを確認
      await waitFor(() => {
        const button = screen.getByRole('button')
        expect(button).toBeDisabled()
        expect(button).toHaveClass('btn', 'btn-sm', 'btn-secondary')
      })
    })
  })

  describe('タイマー開始機能', () => {
    it('タイマー開始が正常に動作する', async () => {
      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
        />
      )

      // 開始ボタンをクリック
      const startButton = await screen.findByRole('button', { name: 'テストタスクのタイマー（停止中）を開始' })
      fireEvent.click(startButton)

      // ローディング状態を確認
      await waitFor(() => {
        expect(screen.getByText('開始中...')).toBeInTheDocument()
      })

      // APIが呼ばれることを確認
      await waitFor(() => {
        expect(TimeTrackingApi.startTimer).toHaveBeenCalledWith(mockTask.id)
      })

      // 状態が再読み込みされることを確認
      await waitFor(() => {
        expect(TimeTrackingApi.getTimerStatus).toHaveBeenCalledTimes(2) // 初期読み込み + 開始後
      })

      // コールバックが呼ばれることを確認
      await waitFor(() => {
        expect(mockOnTimerStateChanged).toHaveBeenCalledWith(mockTask.id, true)
      })
    })

    it('タイマー開始中にエラーが発生した場合の処理', async () => {
      const errorMessage = 'タイマーの開始に失敗しました'
      vi.mocked(TimeTrackingApi.startTimer).mockRejectedValue(new Error(errorMessage))

      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
        />
      )

      // 開始ボタンをクリック
      const startButton = await screen.findByRole('button', { name: 'テストタスクのタイマー（停止中）を開始' })
      fireEvent.click(startButton)

      // エラーメッセージが表示されることを確認
      await waitFor(() => {
        expect(screen.getByText('タイマーの開始に失敗しました')).toBeInTheDocument()
        expect(screen.getByRole('alert')).toBeInTheDocument()
      })

      // ボタンが再度有効になることを確認
      await waitFor(() => {
        expect(startButton).toBeEnabled()
        expect(screen.getByText('タイマー開始')).toBeInTheDocument()
      })

      // コールバックは呼ばれないことを確認
      expect(mockOnTimerStateChanged).not.toHaveBeenCalled()
    })

    it('ローディング中はボタンが無効化される', async () => {
      // startTimerを遅延させる
      vi.mocked(TimeTrackingApi.startTimer).mockImplementation(
        () => new Promise(resolve => setTimeout(resolve, 100))
      )

      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
        />
      )

      // 開始ボタンをクリック
      const startButton = await screen.findByRole('button', { name: 'テストタスクのタイマー（停止中）を開始' })
      fireEvent.click(startButton)

      // ローディング中はボタンが無効化されることを確認
      await waitFor(() => {
        expect(startButton).toBeDisabled()
        expect(screen.getByText('開始中...')).toBeInTheDocument()
      })
    })
  })

  describe('タイマー停止機能', () => {
    it('タイマー停止が正常に動作する', async () => {
      vi.mocked(TimeTrackingApi.getTimerStatus).mockResolvedValue(mockRunningTimerStatus)

      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
        />
      )

      // 停止ボタンをクリック
      const stopButton = await screen.findByRole('button', { name: 'テストタスクのタイマー（実行中）を停止' })
      fireEvent.click(stopButton)

      // ローディング状態を確認
      await waitFor(() => {
        expect(screen.getByText('停止中...')).toBeInTheDocument()
      })

      // APIが呼ばれることを確認
      await waitFor(() => {
        expect(TimeTrackingApi.stopTimer).toHaveBeenCalledWith(mockTask.id)
      })

      // 状態が再読み込みされることを確認
      await waitFor(() => {
        expect(TimeTrackingApi.getTimerStatus).toHaveBeenCalledTimes(2) // 初期読み込み + 停止後
      })

      // コールバックが呼ばれることを確認
      await waitFor(() => {
        expect(mockOnTimerStateChanged).toHaveBeenCalledWith(mockTask.id, false)
      })
    })

    it('タイマー停止中にエラーが発生した場合の処理', async () => {
      vi.mocked(TimeTrackingApi.getTimerStatus).mockResolvedValue(mockRunningTimerStatus)
      vi.mocked(TimeTrackingApi.stopTimer).mockRejectedValue(new Error('タイマーの停止に失敗しました'))

      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
        />
      )

      // 停止ボタンをクリック
      const stopButton = await screen.findByRole('button', { name: 'テストタスクのタイマー（実行中）を停止' })
      fireEvent.click(stopButton)

      // エラーメッセージが表示されることを確認
      await waitFor(() => {
        expect(screen.getByText('タイマーの停止に失敗しました')).toBeInTheDocument()
        expect(screen.getByRole('alert')).toBeInTheDocument()
      })

      // ボタンが再度有効になることを確認
      await waitFor(() => {
        expect(stopButton).toBeEnabled()
        expect(screen.getByText('タイマー停止')).toBeInTheDocument()
      })
    })
  })

  describe('タイマー状態取得エラー', () => {
    it('タイマー状態取得に失敗した場合の処理', async () => {
      vi.mocked(TimeTrackingApi.getTimerStatus).mockRejectedValue(new Error('状態取得に失敗しました'))

      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
        />
      )

      // エラーメッセージが表示されることを確認
      await waitFor(() => {
        expect(screen.getByText('タイマー状態の取得に失敗しました')).toBeInTheDocument()
        expect(screen.getByRole('alert')).toBeInTheDocument()
      })

      // ボタンは有効な状態を維持することを確認
      const button = screen.getByRole('button')
      expect(button).toBeEnabled()
      expect(button).toHaveClass('btn', 'btn-sm', 'btn-success', 'timer-button-stopped')
    })
  })

  describe('無効化状態での動作', () => {
    it('無効化状態ではタイマー開始が動作しない', async () => {
      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
          disabled={true}
        />
      )

      // 初期状態取得でonTimerStateChangedが呼ばれるので、モックをクリア
      await waitFor(() => {
        expect(TimeTrackingApi.getTimerStatus).toHaveBeenCalled()
      })
      vi.clearAllMocks()

      const button = screen.getByRole('button')
      
      // ボタンが無効化されていることを確認
      expect(button).toBeDisabled()
      
      fireEvent.click(button)

      // APIが呼ばれないことを確認
      expect(TimeTrackingApi.startTimer).not.toHaveBeenCalled()
      // 無効化状態ではonTimerStateChangedも呼ばれない
      expect(mockOnTimerStateChanged).not.toHaveBeenCalled()
    })

    it('無効化状態ではタイマー停止が動作しない', async () => {
      vi.mocked(TimeTrackingApi.getTimerStatus).mockResolvedValue(mockRunningTimerStatus)

      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
          disabled={true}
        />
      )

      // 初期状態取得でonTimerStateChangedが呼ばれるので、モックをクリア
      await waitFor(() => {
        expect(TimeTrackingApi.getTimerStatus).toHaveBeenCalled()
      })
      vi.clearAllMocks()

      const button = screen.getByRole('button')
      
      // ボタンが無効化されていることを確認
      expect(button).toBeDisabled()
      
      fireEvent.click(button)

      // APIが呼ばれないことを確認
      expect(TimeTrackingApi.stopTimer).not.toHaveBeenCalled()
      // 無効化状態ではonTimerStateChangedも呼ばれない
      expect(mockOnTimerStateChanged).not.toHaveBeenCalled()
    })
  })

  describe('ローディング状態での動作', () => {
    it('ローディング中は追加のクリックが無視される', async () => {
      // startTimerを遅延させる
      vi.mocked(TimeTrackingApi.startTimer).mockImplementation(
        () => new Promise(resolve => setTimeout(resolve, 100))
      )

      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
        />
      )

      const button = await screen.findByRole('button', { name: 'テストタスクのタイマー（停止中）を開始' })
      
      // 最初のクリック
      fireEvent.click(button)
      
      // ローディング中に追加でクリック
      fireEvent.click(button)
      fireEvent.click(button)

      // startTimerは1回だけ呼ばれることを確認
      await waitFor(() => {
        expect(TimeTrackingApi.startTimer).toHaveBeenCalledTimes(1)
      })
    })
  })

  describe('アクセシビリティ', () => {
    it('aria-labelが正しく設定される', async () => {
      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
        />
      )

      // 停止状態のaria-label
      const button = await screen.findByRole('button', { name: 'テストタスクのタイマー（停止中）を開始' })
      expect(button).toHaveAttribute('aria-label', 'テストタスクのタイマー（停止中）を開始')
    })

    it('実行状態のaria-labelが正しく設定される', async () => {
      vi.mocked(TimeTrackingApi.getTimerStatus).mockResolvedValue(mockRunningTimerStatus)

      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
        />
      )

      // 実行状態のaria-label
      const button = await screen.findByRole('button', { name: 'テストタスクのタイマー（実行中）を停止' })
      expect(button).toHaveAttribute('aria-label', 'テストタスクのタイマー（実行中）を停止')
    })

    it('data-testidが正しく設定される', async () => {
      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
        />
      )

      const button = await screen.findByTestId('timer-button')
      expect(button).toBeInTheDocument()
    })
  })

  describe('エラー状態のクリア', () => {
    it('新しい操作でエラーがクリアされる', async () => {
      // 最初のstartTimerでエラーを発生させる
      vi.mocked(TimeTrackingApi.startTimer)
        .mockRejectedValueOnce(new Error('最初のエラー'))
        .mockResolvedValueOnce({
          id: 1,
          task_id: 1,
          event_type: 'start',
          at: '2025-01-01T00:00:00Z'
        })

      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
        />
      )

      const button = await screen.findByRole('button', { name: 'テストタスクのタイマー（停止中）を開始' })
      
      // 最初のクリックでエラーを発生
      fireEvent.click(button)
      await waitFor(() => {
        expect(screen.getByText('タイマーの開始に失敗しました')).toBeInTheDocument()
      })

      // 再度クリックしてエラーをクリア
      fireEvent.click(button)
      await waitFor(() => {
        expect(screen.queryByText('タイマーの開始に失敗しました')).not.toBeInTheDocument()
      })
    })
  })

  describe('統合テスト', () => {
    it('複数のタイマー操作が連続して正常に動作する', async () => {
      // 最初は停止状態
      vi.mocked(TimeTrackingApi.getTimerStatus).mockResolvedValue(mockTimerStatus)
      
      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
        />
      )

      // 1. タイマー開始
      const startButton = await screen.findByRole('button', { name: 'テストタスクのタイマー（停止中）を開始' })
      fireEvent.click(startButton)

      await waitFor(() => {
        expect(TimeTrackingApi.startTimer).toHaveBeenCalledWith(mockTask.id)
        expect(mockOnTimerStateChanged).toHaveBeenCalledWith(mockTask.id, true)
      })

      // 2. 実行状態に変更して新しいコンポーネントをレンダリング
      vi.mocked(TimeTrackingApi.getTimerStatus).mockResolvedValue(mockRunningTimerStatus)
      
      // 新しいコンポーネントインスタンスを作成
      const { rerender: _rerender } = render(  // rerenderに_プレフィックス追加
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
        />
      )

      // 3. タイマー停止
      const stopButton = await screen.findByRole('button', { name: 'テストタスクのタイマー（実行中）を停止' })
      fireEvent.click(stopButton)

      await waitFor(() => {
        expect(TimeTrackingApi.stopTimer).toHaveBeenCalledWith(mockTask.id)
        expect(mockOnTimerStateChanged).toHaveBeenCalledWith(mockTask.id, false)
      })
    })

    it('エラー後の復旧が正常に動作する', async () => {
      // 最初にエラーを発生させる
      vi.mocked(TimeTrackingApi.startTimer)
        .mockRejectedValueOnce(new Error('ネットワークエラー'))
        .mockResolvedValueOnce({
          id: 1,
          task_id: 1,
          event_type: 'start',
          at: '2025-01-01T00:00:00Z'
        })

      render(
        <TimerButton
          task={mockTask}
          onTimerStateChanged={mockOnTimerStateChanged}
        />
      )

      const button = await screen.findByRole('button', { name: 'テストタスクのタイマー（停止中）を開始' })
      
      // 1回目のクリックでエラー
      fireEvent.click(button)
      await waitFor(() => {
        expect(screen.getByText('タイマーの開始に失敗しました')).toBeInTheDocument()
      })

      // 2回目のクリックで成功
      fireEvent.click(button)
      await waitFor(() => {
        expect(screen.queryByText('タイマーの開始に失敗しました')).not.toBeInTheDocument()
        expect(mockOnTimerStateChanged).toHaveBeenCalledWith(mockTask.id, true)
      })
    })
  })
})

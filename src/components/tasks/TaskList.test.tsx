import { render, screen, waitFor, fireEvent } from '@testing-library/react'
import { vi, describe, it, expect, beforeEach } from 'vitest'
import { TaskList } from './TaskList'
import type { Task, Project } from '@/types'

// Mock the invoke function
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

// Mock the useTasks hook
vi.mock('@/hooks/useTasks', () => ({
  useTasks: vi.fn()
}))

// Mock the dateUtils
vi.mock('@/utils/dateUtils', () => ({
  formatDate: vi.fn(() => `2025年1月1日`)
}))

// Import the mocked functions
import { useTasks } from '@/hooks/useTasks'
const mockUseTasks = vi.mocked(useTasks)

describe('TaskList', () => {
  const mockTasks: Task[] = [
    {
      id: 1,
      name: 'テストタスク1',
      project_id: 1,
      status: 'active',
      effective_at: '2025-01-01T00:00:00Z'
    },
    {
      id: 2,
      name: 'テストタスク2',
      project_id: 1,
      status: 'archived',
      effective_at: '2025-01-01T00:00:00Z'
    }
  ]

  const mockProjects: Project[] = [
    {
      id: 1,
      name: 'テストプロジェクト',
      status: 'active',
      effective_at: '2025-01-01T00:00:00Z'
    }
  ]

  const mockUseTasksReturn = {
    tasks: mockTasks,
    loading: false,
    error: null,
    selectedDate: new Date('2025-01-01'),
    createTask: vi.fn(),
    updateTask: vi.fn(),
    archiveTask: vi.fn(),
    restoreTask: vi.fn(),
    loadTasks: vi.fn(),
    setSelectedDate: vi.fn(),
    goToPreviousDay: vi.fn(),
    goToNextDay: vi.fn(),
    clearError: vi.fn()
  }

  beforeEach(() => {
    vi.clearAllMocks()
    mockUseTasks.mockReturnValue(mockUseTasksReturn)
  })

  it('タスク一覧が正常に表示される', async () => {
    render(<TaskList selectedProjectId={1} projects={mockProjects} showArchived={true} />)

    await waitFor(() => {
      expect(screen.getByText('テストタスク1')).toBeInTheDocument()
      expect(screen.getByText('テストタスク2')).toBeInTheDocument()
    })
  })

  it('ローディング状態が正しく表示される', () => {
    mockUseTasks.mockReturnValue({
      ...mockUseTasksReturn,
      loading: true
    })

    render(<TaskList selectedProjectId={1} projects={mockProjects} />)
    
    expect(screen.getByTestId('task-list-loading')).toBeInTheDocument()
  })

  it('エラー状態が正しく表示される', () => {
    mockUseTasks.mockReturnValue({
      ...mockUseTasksReturn,
      error: 'タスクの取得に失敗しました'
    })

    render(<TaskList selectedProjectId={1} projects={mockProjects} />)
    
    expect(screen.getByText('タスクの取得に失敗しました')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '再試行' })).toBeInTheDocument()
  })

  it('タスクが存在しない場合、空状態が表示される', () => {
    mockUseTasks.mockReturnValue({
      ...mockUseTasksReturn,
      tasks: []
    })

    render(<TaskList selectedProjectId={1} projects={mockProjects} />)
    
    expect(screen.getByText('タスクがありません')).toBeInTheDocument()
    expect(screen.getByText('新しいタスクを作成してください。')).toBeInTheDocument()
  })

  it('プロジェクトが選択されていない場合、プロジェクト選択を促すメッセージが表示される', () => {
    render(<TaskList selectedProjectId={null} projects={mockProjects} />)
    
    expect(screen.getByText('プロジェクトを選択してください')).toBeInTheDocument()
  })

  it('プロジェクトが変更された時、タスクが再読み込みされる', async () => {
    const { rerender } = render(<TaskList selectedProjectId={1} projects={mockProjects} />)

    rerender(<TaskList selectedProjectId={2} projects={mockProjects} />)

    await waitFor(() => {
      expect(mockUseTasksReturn.loadTasks).toHaveBeenCalledWith(2, expect.any(Date))
    })
  })

  it('再試行ボタンがクリックされた時、タスクが再読み込みされる', async () => {
    mockUseTasks.mockReturnValue({
      ...mockUseTasksReturn,
      error: 'エラーが発生しました'
    })

    render(<TaskList selectedProjectId={1} projects={mockProjects} />)
    
    const retryButton = screen.getByRole('button', { name: '再試行' })
    fireEvent.click(retryButton)

    await waitFor(() => {
      expect(mockUseTasksReturn.loadTasks).toHaveBeenCalledWith(1, expect.any(Date))
    })
  })

  it('アクティブタスクのみが表示される（フィルター機能）', () => {
    render(<TaskList selectedProjectId={1} projects={mockProjects} showArchived={false} />)

    expect(screen.getByText('テストタスク1')).toBeInTheDocument()
    expect(screen.queryByText('テストタスク2')).not.toBeInTheDocument()
  })

  it('アーカイブ済みタスクも表示される（フィルター機能）', () => {
    render(<TaskList selectedProjectId={1} projects={mockProjects} showArchived={true} />)

    expect(screen.getByText('テストタスク1')).toBeInTheDocument()
    expect(screen.getByText('テストタスク2')).toBeInTheDocument()
  })

  it('日付ナビゲーションが表示される', () => {
    render(<TaskList selectedProjectId={1} projects={mockProjects} />)

    expect(screen.getByTestId('previous-day-btn')).toBeInTheDocument()
    expect(screen.getByTestId('next-day-btn')).toBeInTheDocument()
    expect(screen.getByText(/2025年1月1日 - 1件のタスク/)).toBeInTheDocument()
  })

  it('前日ボタンがクリックされた時、goToPreviousDayが呼ばれる', () => {
    render(<TaskList selectedProjectId={1} projects={mockProjects} />)
    
    const previousButton = screen.getByTestId('previous-day-btn')
    fireEvent.click(previousButton)

    expect(mockUseTasksReturn.goToPreviousDay).toHaveBeenCalled()
  })

  it('翌日ボタンがクリックされた時、goToNextDayが呼ばれる', () => {
    render(<TaskList selectedProjectId={1} projects={mockProjects} />)
    
    const nextButton = screen.getByTestId('next-day-btn')
    fireEvent.click(nextButton)

    expect(mockUseTasksReturn.goToNextDay).toHaveBeenCalled()
  })

  it('日付が変更された時、タスクが再読み込みされる', async () => {
    const { rerender } = render(<TaskList selectedProjectId={1} projects={mockProjects} />)

    // 日付を変更
    const newDate = new Date('2025-01-02')
    mockUseTasks.mockReturnValue({
      ...mockUseTasksReturn,
      selectedDate: newDate
    })

    rerender(<TaskList selectedProjectId={1} projects={mockProjects} />)

    await waitFor(() => {
      expect(mockUseTasksReturn.loadTasks).toHaveBeenCalledWith(1, expect.any(Date))
    })
  })
})

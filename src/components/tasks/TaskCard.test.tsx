import { render, screen, fireEvent } from '@testing-library/react'
import { vi, describe, it, expect, beforeEach } from 'vitest'
import { TaskCard } from './TaskCard'
import type { Task } from '@/types'

describe('TaskCard', () => {
  const mockTask: Task = {
    id: 1,
    name: 'テストタスク',
    project_id: 1,
    status: 'active',
    effective_at: '2025-01-01T00:00:00Z'
  }

  const mockOnEdit = vi.fn()
  const mockOnArchive = vi.fn()
  const mockOnRestore = vi.fn()
  const mockOnStartTimer = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('タスク情報が正しく表示される', () => {
    render(
      <TaskCard
        task={mockTask}
        onEdit={mockOnEdit}
        onArchive={mockOnArchive}
        onRestore={mockOnRestore}
        onStartTimer={mockOnStartTimer}
      />
    )

    expect(screen.getByText('テストタスク')).toBeInTheDocument()
    expect(screen.getByText('アクティブ')).toBeInTheDocument()
  })

  it('アクティブタスクの場合、適切なボタンが表示される', () => {
    render(
      <TaskCard
        task={mockTask}
        onEdit={mockOnEdit}
        onArchive={mockOnArchive}
        onRestore={mockOnRestore}
        onStartTimer={mockOnStartTimer}
      />
    )

    expect(screen.getByRole('button', { name: /編集/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /アーカイブ/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /タイマー開始/i })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /復元/i })).not.toBeInTheDocument()
  })

  it('アーカイブ済みタスクの場合、復元ボタンが表示される', () => {
    const archivedTask = { ...mockTask, status: 'archived' as const }
    
    render(
      <TaskCard
        task={archivedTask}
        onEdit={mockOnEdit}
        onArchive={mockOnArchive}
        onRestore={mockOnRestore}
        onStartTimer={mockOnStartTimer}
      />
    )

    expect(screen.getByRole('button', { name: /復元/i })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /アーカイブ/i })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /タイマー開始/i })).not.toBeInTheDocument()
  })

  it('編集ボタンがクリックされた時、onEditが呼ばれる', () => {
    render(
      <TaskCard
        task={mockTask}
        onEdit={mockOnEdit}
        onArchive={mockOnArchive}
        onRestore={mockOnRestore}
        onStartTimer={mockOnStartTimer}
      />
    )

    const editButton = screen.getByRole('button', { name: /編集/i })
    fireEvent.click(editButton)

    expect(mockOnEdit).toHaveBeenCalledWith(mockTask)
  })

  it('アーカイブボタンがクリックされた時、onArchiveが呼ばれる', () => {
    render(
      <TaskCard
        task={mockTask}
        onEdit={mockOnEdit}
        onArchive={mockOnArchive}
        onRestore={mockOnRestore}
        onStartTimer={mockOnStartTimer}
      />
    )

    const archiveButton = screen.getByRole('button', { name: /アーカイブ/i })
    fireEvent.click(archiveButton)

    expect(mockOnArchive).toHaveBeenCalledWith(mockTask)
  })

  it('復元ボタンがクリックされた時、onRestoreが呼ばれる', () => {
    const archivedTask = { ...mockTask, status: 'archived' as const }
    
    render(
      <TaskCard
        task={archivedTask}
        onEdit={mockOnEdit}
        onArchive={mockOnArchive}
        onRestore={mockOnRestore}
        onStartTimer={mockOnStartTimer}
      />
    )

    const restoreButton = screen.getByRole('button', { name: /復元/i })
    fireEvent.click(restoreButton)

    expect(mockOnRestore).toHaveBeenCalledWith(archivedTask)
  })

  it('タイマー開始ボタンがクリックされた時、onStartTimerが呼ばれる', () => {
    render(
      <TaskCard
        task={mockTask}
        onEdit={mockOnEdit}
        onArchive={mockOnArchive}
        onRestore={mockOnRestore}
        onStartTimer={mockOnStartTimer}
      />
    )

    const startTimerButton = screen.getByRole('button', { name: /タイマー開始/i })
    fireEvent.click(startTimerButton)

    expect(mockOnStartTimer).toHaveBeenCalledWith(mockTask)
  })

  it('タスク名が長い場合、適切に表示される', () => {
    const longNameTask = {
      ...mockTask,
      name: 'とても長いタスク名です。このタスク名は非常に長くて、表示時に省略される可能性があります。'
    }

    render(
      <TaskCard
        task={longNameTask}
        onEdit={mockOnEdit}
        onArchive={mockOnArchive}
        onRestore={mockOnRestore}
        onStartTimer={mockOnStartTimer}
      />
    )

    expect(screen.getByText(longNameTask.name)).toBeInTheDocument()
  })

  it('作成日が正しくフォーマットされて表示される', () => {
    render(
      <TaskCard
        task={mockTask}
        onEdit={mockOnEdit}
        onArchive={mockOnArchive}
        onRestore={mockOnRestore}
        onStartTimer={mockOnStartTimer}
      />
    )

    // 作成日が表示されていることを確認（フォーマットは実装に依存）
    expect(screen.getByText(/2025/)).toBeInTheDocument()
  })
})

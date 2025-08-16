import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ProjectSidebar } from './ProjectSidebar'
import { invoke } from '@tauri-apps/api/core'

const mockProjects = [
  {
    id: 1,
    name: 'テストプロジェクト1',
    status: 'active' as const,
    effective_at: '2024-01-01T00:00:00Z'
  },
  {
    id: 2,
    name: 'テストプロジェクト2',
    status: 'active' as const,
    effective_at: '2024-01-01T00:00:00Z'
  }
]

const mockTimeEntries = [
  {
    id: 1,
    task_id: 1,
    start_event_id: 1,
    start_time: '2024-01-01T09:00:00Z',
    end_time: '2024-01-01T10:00:00Z',
    duration_in_seconds: 3600,
    elapsed_duration: '01:00:00',
    is_running: false,
    is_completed: true
  }
]

describe('ProjectSidebar', () => {
  const mockOnProjectSelect = vi.fn()
  const mockOnRefresh = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
    // デフォルトのモック設定
    vi.mocked(invoke).mockImplementation(async (command: string, params?: any) => {
      if (command === 'create_project') {
        return {
          id: 3,
          name: params.request.name,
          status: 'active',
          effective_at: '2024-01-01T00:00:00Z'
        }
      }
      throw new Error(`Unknown command: ${command}`)
    })
  })

  it('プロジェクト一覧を正常に表示する', () => {
    render(
      <ProjectSidebar
        projects={mockProjects}
        timeEntries={mockTimeEntries}
        selectedProject={null}
        onProjectSelect={mockOnProjectSelect}
        onRefresh={mockOnRefresh}
      />
    )

    expect(screen.getByText('Projects')).toBeInTheDocument()
    expect(screen.getByText('テストプロジェクト1')).toBeInTheDocument()
    expect(screen.getByText('テストプロジェクト2')).toBeInTheDocument()
  })

  it('プロジェクトが選択された状態で正しく表示される', () => {
    render(
      <ProjectSidebar
        projects={mockProjects}
        timeEntries={mockTimeEntries}
        selectedProject={mockProjects[0]}
        onProjectSelect={mockOnProjectSelect}
        onRefresh={mockOnRefresh}
      />
    )

    const selectedProject = screen.getByText('テストプロジェクト1').closest('.project-item')
    expect(selectedProject).toHaveClass('selected')
  })

  it('プロジェクトをクリックするとonProjectSelectが呼ばれる', () => {
    render(
      <ProjectSidebar
        projects={mockProjects}
        timeEntries={mockTimeEntries}
        selectedProject={null}
        onProjectSelect={mockOnProjectSelect}
        onRefresh={mockOnRefresh}
      />
    )

    const projectItem = screen.getByText('テストプロジェクト1')
    fireEvent.click(projectItem)

    expect(mockOnProjectSelect).toHaveBeenCalledWith(mockProjects[0])
  })

  it('プロジェクトが存在しない場合に適切なメッセージを表示する', () => {
    render(
      <ProjectSidebar
        projects={[]}
        timeEntries={[]}
        selectedProject={null}
        onProjectSelect={mockOnProjectSelect}
        onRefresh={mockOnRefresh}
      />
    )

    expect(screen.getByText('プロジェクトがありません')).toBeInTheDocument()
    expect(screen.getByText('新しいプロジェクトを作成してください')).toBeInTheDocument()
  })

  it('新しいプロジェクト作成フォームを表示できる', () => {
    render(
      <ProjectSidebar
        projects={mockProjects}
        timeEntries={mockTimeEntries}
        selectedProject={null}
        onProjectSelect={mockOnProjectSelect}
        onRefresh={mockOnRefresh}
      />
    )

    const addButton = screen.getByRole('button', { name: '+' })
    fireEvent.click(addButton)

    expect(screen.getByPlaceholderText('プロジェクト名を入力')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '作成' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'キャンセル' })).toBeInTheDocument()
  })

  it('プロジェクト作成フォームでキャンセルボタンが動作する', () => {
    render(
      <ProjectSidebar
        projects={mockProjects}
        timeEntries={mockTimeEntries}
        selectedProject={null}
        onProjectSelect={mockOnProjectSelect}
        onRefresh={mockOnRefresh}
      />
    )

    // フォームを表示
    const addButton = screen.getByRole('button', { name: '+' })
    fireEvent.click(addButton)

    // キャンセルボタンをクリック
    const cancelButton = screen.getByRole('button', { name: 'キャンセル' })
    fireEvent.click(cancelButton)

    // フォームが非表示になることを確認
    expect(screen.queryByPlaceholderText('プロジェクト名を入力')).not.toBeInTheDocument()
  })

  it('プロジェクト作成フォームでEnterキーでプロジェクトを作成できる', async () => {
    render(
      <ProjectSidebar
        projects={mockProjects}
        timeEntries={mockTimeEntries}
        selectedProject={null}
        onProjectSelect={mockOnProjectSelect}
        onRefresh={mockOnRefresh}
      />
    )

    // フォームを表示
    const addButton = screen.getByRole('button', { name: '+' })
    fireEvent.click(addButton)

    // プロジェクト名を入力
    const input = screen.getByPlaceholderText('プロジェクト名を入力')
    fireEvent.change(input, { target: { value: '新しいプロジェクト' } })

    // Enterキーを押す
    fireEvent.keyDown(input, { key: 'Enter' })

    // プロジェクト作成APIが呼ばれることを確認
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('create_project', {
        request: { name: '新しいプロジェクト' }
      })
    })

    // フォームが閉じられることを確認
    await waitFor(() => {
      expect(screen.queryByPlaceholderText('プロジェクト名を入力')).not.toBeInTheDocument()
    })
  })

  it('プロジェクト作成フォームでEscapeキーでキャンセルできる', () => {
    render(
      <ProjectSidebar
        projects={mockProjects}
        timeEntries={mockTimeEntries}
        selectedProject={null}
        onProjectSelect={mockOnProjectSelect}
        onRefresh={mockOnRefresh}
      />
    )

    // フォームを表示
    const addButton = screen.getByRole('button', { name: '+' })
    fireEvent.click(addButton)

    // Escapeキーを押す
    const input = screen.getByPlaceholderText('プロジェクト名を入力')
    fireEvent.keyDown(input, { key: 'Escape' })

    // フォームが非表示になることを確認
    expect(screen.queryByPlaceholderText('プロジェクト名を入力')).not.toBeInTheDocument()
  })

  it('空のプロジェクト名では作成ボタンが無効化される', () => {
    render(
      <ProjectSidebar
        projects={mockProjects}
        timeEntries={mockTimeEntries}
        selectedProject={null}
        onProjectSelect={mockOnProjectSelect}
        onRefresh={mockOnRefresh}
      />
    )

    // フォームを表示
    const addButton = screen.getByRole('button', { name: '+' })
    fireEvent.click(addButton)

    // 作成ボタンが無効化されていることを確認
    const createButton = screen.getByRole('button', { name: '作成' })
    expect(createButton).toBeDisabled()
  })

  it('プロジェクト作成中は作成ボタンが無効化される', async () => {
    // プロジェクト作成を遅延させる
    vi.mocked(invoke).mockImplementation(async (command: string, params?: any) => {
      if (command === 'create_project') {
        await new Promise(resolve => setTimeout(resolve, 100))
        return {
          id: 3,
          name: params.request.name,
          status: 'active',
          effective_at: '2024-01-01T00:00:00Z'
        }
      }
      throw new Error(`Unknown command: ${command}`)
    })

    render(
      <ProjectSidebar
        projects={mockProjects}
        timeEntries={mockTimeEntries}
        selectedProject={null}
        onProjectSelect={mockOnProjectSelect}
        onRefresh={mockOnRefresh}
      />
    )

    // フォームを表示
    const addButton = screen.getByRole('button', { name: '+' })
    fireEvent.click(addButton)

    // プロジェクト名を入力
    const input = screen.getByPlaceholderText('プロジェクト名を入力')
    fireEvent.change(input, { target: { value: '新しいプロジェクト' } })

    // 作成ボタンをクリック
    const createButton = screen.getByRole('button', { name: '作成' })
    fireEvent.click(createButton)

    // 作成中はボタンが無効化されることを確認
    expect(createButton).toBeDisabled()

    // 作成完了を待つ（onRefreshが呼ばれることを確認）
    await waitFor(() => {
      expect(mockOnRefresh).toHaveBeenCalled()
    })
  })

  it('プロジェクト作成成功後にonRefreshが呼ばれる', async () => {
    render(
      <ProjectSidebar
        projects={mockProjects}
        timeEntries={mockTimeEntries}
        selectedProject={null}
        onProjectSelect={mockOnProjectSelect}
        onRefresh={mockOnRefresh}
      />
    )

    // フォームを表示
    const addButton = screen.getByRole('button', { name: '+' })
    fireEvent.click(addButton)

    // プロジェクト名を入力
    const input = screen.getByPlaceholderText('プロジェクト名を入力')
    fireEvent.change(input, { target: { value: '新しいプロジェクト' } })

    // 作成ボタンをクリック
    const createButton = screen.getByRole('button', { name: '作成' })
    fireEvent.click(createButton)

    // プロジェクト作成成功後にonRefreshが呼ばれることを確認
    await waitFor(() => {
      expect(mockOnRefresh).toHaveBeenCalled()
    })
  })
})

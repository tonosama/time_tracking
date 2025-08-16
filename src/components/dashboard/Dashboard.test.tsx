import { render, screen, waitFor, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { Dashboard } from './Dashboard'
import { invoke } from '@tauri-apps/api/core'

// Tauriのモック
vi.mocked(invoke).mockImplementation(async (command: string) => {
  switch (command) {
    case 'get_all_active_projects':
      return [
        {
          id: 1,
          name: 'テストプロジェクト1',
          status: 'active',
          effective_at: '2024-01-01T00:00:00Z'
        },
        {
          id: 2,
          name: 'テストプロジェクト2',
          status: 'active',
          effective_at: '2024-01-01T00:00:00Z'
        }
      ]
    case 'get_time_entries':
      return []
    case 'get_global_timer_status':
      return {
        task_id: null,
        elapsed_seconds: null
      }
    default:
      throw new Error(`Unknown command: ${command}`)
  }
})

describe('Dashboard', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('プロジェクト一覧を正常に読み込んで表示する', async () => {
    render(<Dashboard />)

    // ローディング状態を確認
    expect(screen.getByText('プロジェクトを読み込み中...')).toBeInTheDocument()

    // プロジェクト一覧が表示されることを確認
    await waitFor(() => {
      expect(screen.getByText('テストプロジェクト1')).toBeInTheDocument()
      expect(screen.getByText('テストプロジェクト2')).toBeInTheDocument()
    })

    // APIが正しく呼び出されていることを確認
    expect(invoke).toHaveBeenCalledWith('get_all_active_projects')
    expect(invoke).toHaveBeenCalledWith('get_time_entries', { start_date: expect.any(String) })
    expect(invoke).toHaveBeenCalledWith('get_global_timer_status')
  })

  it('get_time_entriesが失敗してもプロジェクト一覧は表示される', async () => {
    vi.mocked(invoke).mockImplementation(async (command: string) => {
      if (command === 'get_time_entries') {
        throw new Error('Time entries failed')
      }
      if (command === 'get_all_active_projects') {
        return [
          {
            id: 1,
            name: 'テストプロジェクト1',
            status: 'active',
            effective_at: '2024-01-01T00:00:00Z'
          }
        ]
      }
      if (command === 'get_global_timer_status') {
        return { task_id: null, elapsed_seconds: null }
      }
      throw new Error(`Unknown command: ${command}`)
    })

    render(<Dashboard />)

    // プロジェクト一覧が表示されることを確認
    await waitFor(() => {
      expect(screen.getByText('テストプロジェクト1')).toBeInTheDocument()
    })

    // エラーが発生しても他の機能は正常に動作することを確認
    expect(invoke).toHaveBeenCalledWith('get_all_active_projects')
    expect(invoke).toHaveBeenCalledWith('get_global_timer_status')
  })

  it('プロジェクトが選択されていない状態でタイマー開始を試行するとエラーメッセージが表示される', async () => {
    render(<Dashboard />)

    await waitFor(() => {
      expect(screen.getByText('テストプロジェクト1')).toBeInTheDocument()
    })

    // タイマー開始ボタンをクリック
    const startButton = screen.getByRole('button', { name: /開始/i })
    fireEvent.click(startButton)

    // エラーメッセージが表示されることを確認
    await waitFor(() => {
      expect(screen.getByText('プロジェクトを選択してください')).toBeInTheDocument()
    })
  })

  it('プロジェクトを選択してタイマーを開始できる', async () => {
    // タスク取得とタイマー開始のモック
    vi.mocked(invoke).mockImplementation(async (command: string, params?: any) => {
      if (command === 'get_active_tasks_by_project') {
        return []
      }
      if (command === 'create_task') {
        return { id: 1, name: 'Default Task', project_id: 1 }
      }
      if (command === 'start_timer') {
        return { success: true }
      }
      // 他のコマンドは既存のモックを使用
      return vi.mocked(invoke).mock.results[0]?.value || []
    })

    render(<Dashboard />)

    await waitFor(() => {
      expect(screen.getByText('テストプロジェクト1')).toBeInTheDocument()
    })

    // プロジェクトを選択
    const projectSelect = screen.getByRole('combobox')
    fireEvent.change(projectSelect, { target: { value: '1' } })

    // タスク説明を入力
    const taskInput = screen.getByPlaceholderText(/タスクの説明/i)
    fireEvent.change(taskInput, { target: { value: 'テストタスク' } })

    // タイマー開始ボタンをクリック
    const startButton = screen.getByRole('button', { name: /開始/i })
    fireEvent.click(startButton)

    // タイマー開始のAPIが呼び出されることを確認
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('get_active_tasks_by_project', { project_id: 1 })
      expect(invoke).toHaveBeenCalledWith('create_task', {
        project_id: 1,
        name: 'テストタスク'
      })
      expect(invoke).toHaveBeenCalledWith('start_timer', { task_id: 1 })
    })
  })
})

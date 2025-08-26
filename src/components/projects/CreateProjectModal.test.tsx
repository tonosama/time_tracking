import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { CreateProjectModal } from './CreateProjectModal'
import { invoke } from '@tauri-apps/api/core'

describe('CreateProjectModal', () => {
  const mockOnClose = vi.fn()
  const mockOnProjectCreated = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
    // デフォルトのモック設定
    vi.mocked(invoke).mockImplementation(async (command: string, params?: any) => {
      if (command === 'create_project') {
        return {
          id: 1,
          name: params.request.name,
          status: 'active',
          effective_at: '2024-01-01T00:00:00Z'
        }
      }
      throw new Error(`Unknown command: ${command}`)
    })
  })

  it('モーダルが開いている時にプロジェクト作成フォームを表示する', () => {
    render(
      <CreateProjectModal
        isOpen={true}
        onClose={mockOnClose}
        onProjectCreated={mockOnProjectCreated}
      />
    )

    expect(screen.getByText('新しいプロジェクト')).toBeInTheDocument()
    expect(screen.getByLabelText('プロジェクト名')).toBeInTheDocument()
    expect(screen.getByPlaceholderText('プロジェクト名を入力')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '作成' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'キャンセル' })).toBeInTheDocument()
  })

  it('モーダルが閉じている時は何も表示しない', () => {
    render(
      <CreateProjectModal
        isOpen={false}
        onClose={mockOnClose}
        onProjectCreated={mockOnProjectCreated}
      />
    )

    expect(screen.queryByText('新しいプロジェクト')).not.toBeInTheDocument()
    expect(screen.queryByLabelText('プロジェクト名')).not.toBeInTheDocument()
  })

  it('プロジェクト名を入力して作成ボタンを押すとプロジェクトが作成される', async () => {
    render(
      <CreateProjectModal
        isOpen={true}
        onClose={mockOnClose}
        onProjectCreated={mockOnProjectCreated}
      />
    )

    // プロジェクト名を入力
    const input = screen.getByLabelText('プロジェクト名')
    fireEvent.change(input, { target: { value: 'テストプロジェクト' } })

    // 作成ボタンをクリック
    const createButton = screen.getByRole('button', { name: '作成' })
    fireEvent.click(createButton)

    // プロジェクト作成APIが呼ばれることを確認
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('create_project', {
        request: { name: 'テストプロジェクト' }
      })
    })

    // コールバックが呼ばれることを確認
    await waitFor(() => {
      expect(mockOnProjectCreated).toHaveBeenCalledWith({
        id: 1,
        name: 'テストプロジェクト',
        status: 'active',
        effective_at: '2024-01-01T00:00:00Z'
      })
    })

    // モーダルが閉じられることを確認
    expect(mockOnClose).toHaveBeenCalled()
  })

  it('空のプロジェクト名では作成ボタンが無効化される', () => {
    render(
      <CreateProjectModal
        isOpen={true}
        onClose={mockOnClose}
        onProjectCreated={mockOnProjectCreated}
      />
    )

    // 作成ボタンが無効化されていることを確認
    const createButton = screen.getByRole('button', { name: '作成' })
    expect(createButton).toBeDisabled()

    // APIは呼ばれないことを確認
    expect(invoke).not.toHaveBeenCalled()
  })

  it('空白のみのプロジェクト名では作成ボタンが無効化される', () => {
    render(
      <CreateProjectModal
        isOpen={true}
        onClose={mockOnClose}
        onProjectCreated={mockOnProjectCreated}
      />
    )

    // 空白のみのプロジェクト名を入力
    const input = screen.getByLabelText('プロジェクト名')
    fireEvent.change(input, { target: { value: '   ' } })

    // 作成ボタンが無効化されていることを確認
    const createButton = screen.getByRole('button', { name: '作成' })
    expect(createButton).toBeDisabled()

    // APIは呼ばれないことを確認
    expect(invoke).not.toHaveBeenCalled()
  })

  it('キャンセルボタンを押すとモーダルが閉じられる', () => {
    render(
      <CreateProjectModal
        isOpen={true}
        onClose={mockOnClose}
        onProjectCreated={mockOnProjectCreated}
      />
    )

    const cancelButton = screen.getByRole('button', { name: 'キャンセル' })
    fireEvent.click(cancelButton)

    expect(mockOnClose).toHaveBeenCalled()
  })

  it('プロジェクト作成中は作成ボタンが無効化される', async () => {
    // プロジェクト作成を遅延させる
    vi.mocked(invoke).mockImplementation(async (command: string, params?: any) => {
      if (command === 'create_project') {
        await new Promise(resolve => setTimeout(resolve, 100))
        return {
          id: 1,
          name: params.request.name,
          status: 'active',
          effective_at: '2024-01-01T00:00:00Z'
        }
      }
      throw new Error(`Unknown command: ${command}`)
    })

    render(
      <CreateProjectModal
        isOpen={true}
        onClose={mockOnClose}
        onProjectCreated={mockOnProjectCreated}
      />
    )

    // プロジェクト名を入力
    const input = screen.getByLabelText('プロジェクト名')
    fireEvent.change(input, { target: { value: 'テストプロジェクト' } })

    // 作成ボタンをクリック
    const createButton = screen.getByRole('button', { name: '作成' })
    fireEvent.click(createButton)

    // 作成中はボタンが無効化されることを確認
    expect(createButton).toBeDisabled()

    // 作成完了を待つ（コールバックが呼ばれることを確認）
    await waitFor(() => {
      expect(mockOnProjectCreated).toHaveBeenCalled()
    })

    // モーダルが閉じられることを確認
    expect(mockOnClose).toHaveBeenCalled()
  })

  it('プロジェクト作成中はキャンセルボタンも無効化される', async () => {
    // プロジェクト作成を遅延させる
    vi.mocked(invoke).mockImplementation(async (command: string, params?: any) => {
      if (command === 'create_project') {
        await new Promise(resolve => setTimeout(resolve, 100))
        return {
          id: 1,
          name: params.request.name,
          status: 'active',
          effective_at: '2024-01-01T00:00:00Z'
        }
      }
      throw new Error(`Unknown command: ${command}`)
    })

    render(
      <CreateProjectModal
        isOpen={true}
        onClose={mockOnClose}
        onProjectCreated={mockOnProjectCreated}
      />
    )

    // プロジェクト名を入力
    const input = screen.getByLabelText('プロジェクト名')
    fireEvent.change(input, { target: { value: 'テストプロジェクト' } })

    // 作成ボタンをクリック
    const createButton = screen.getByRole('button', { name: '作成' })
    fireEvent.click(createButton)

    // 作成中はキャンセルボタンも無効化されることを確認
    const cancelButton = screen.getByRole('button', { name: 'キャンセル' })
    expect(cancelButton).toBeDisabled()

    // 作成完了を待つ（コールバックが呼ばれることを確認）
    await waitFor(() => {
      expect(mockOnProjectCreated).toHaveBeenCalled()
    })

    // モーダルが閉じられることを確認
    expect(mockOnClose).toHaveBeenCalled()
  })

  it('プロジェクト作成エラー時にエラーメッセージが表示される', async () => {
    // プロジェクト作成でエラーを発生させる
    vi.mocked(invoke).mockImplementation(async (command: string, _params?: any) => {  // paramsに_プレフィックス追加
      if (command === 'create_project') {
        throw new Error('プロジェクト作成に失敗しました')
      }
      throw new Error(`Unknown command: ${command}`)
    })

    render(
      <CreateProjectModal
        isOpen={true}
        onClose={mockOnClose}
        onProjectCreated={mockOnProjectCreated}
      />
    )

    // プロジェクト名を入力
    const input = screen.getByLabelText('プロジェクト名')
    fireEvent.change(input, { target: { value: 'テストプロジェクト' } })

    // 作成ボタンをクリック
    const createButton = screen.getByRole('button', { name: '作成' })
    fireEvent.click(createButton)

    // エラーメッセージが表示されることを確認
    await waitFor(() => {
      expect(screen.getByText('プロジェクト作成に失敗しました')).toBeInTheDocument()
    })

    // モーダルは閉じられないことを確認
    expect(mockOnClose).not.toHaveBeenCalled()
  })

  it('プロジェクト作成成功後にフォームがリセットされる', async () => {
    const { unmount } = render(
      <CreateProjectModal
        isOpen={true}
        onClose={mockOnClose}
        onProjectCreated={mockOnProjectCreated}
      />
    )

    // プロジェクト名を入力
    const input = screen.getByLabelText('プロジェクト名')
    fireEvent.change(input, { target: { value: 'テストプロジェクト' } })

    // 作成ボタンをクリック
    const createButton = screen.getByRole('button', { name: '作成' })
    fireEvent.click(createButton)

    // プロジェクト作成完了を待つ
    await waitFor(() => {
      expect(mockOnProjectCreated).toHaveBeenCalled()
    })

    // モーダルを閉じて再開
    unmount()
    
    render(
      <CreateProjectModal
        isOpen={true}
        onClose={mockOnClose}
        onProjectCreated={mockOnProjectCreated}
      />
    )

    // フォームがリセットされていることを確認
    const resetInput = screen.getByLabelText('プロジェクト名')
    expect(resetInput).toHaveValue('')
  })

  it('Enterキーでプロジェクトを作成できる', async () => {
    render(
      <CreateProjectModal
        isOpen={true}
        onClose={mockOnClose}
        onProjectCreated={mockOnProjectCreated}
      />
    )

    // プロジェクト名を入力
    const input = screen.getByLabelText('プロジェクト名')
    fireEvent.change(input, { target: { value: 'テストプロジェクト' } })

    // Enterキーを押す（フォームのsubmitイベントが発生）
    const form = screen.getByRole('form')
    fireEvent.submit(form)

    // プロジェクト作成APIが呼ばれることを確認
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('create_project', {
        request: { name: 'テストプロジェクト' }
      })
    })
  })

  it('プロジェクト名が空の時は作成ボタンが無効化される', () => {
    render(
      <CreateProjectModal
        isOpen={true}
        onClose={mockOnClose}
        onProjectCreated={mockOnProjectCreated}
      />
    )

    const createButton = screen.getByRole('button', { name: '作成' })
    expect(createButton).toBeDisabled()
  })

  it('プロジェクト名が入力されると作成ボタンが有効化される', () => {
    render(
      <CreateProjectModal
        isOpen={true}
        onClose={mockOnClose}
        onProjectCreated={mockOnProjectCreated}
      />
    )

    const input = screen.getByLabelText('プロジェクト名')
    const createButton = screen.getByRole('button', { name: '作成' })

    // 初期状態では無効
    expect(createButton).toBeDisabled()

    // プロジェクト名を入力
    fireEvent.change(input, { target: { value: 'テストプロジェクト' } })

    // 有効化される
    expect(createButton).not.toBeDisabled()
  })
})

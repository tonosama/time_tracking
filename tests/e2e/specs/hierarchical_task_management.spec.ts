import { test, expect } from '@playwright/test'

test.describe('US-001: タスクを作成・編集・削除し、階層で管理できる', () => {
  test.beforeEach(async ({ page }) => {
    // Tauriアプリケーションの起動を待機
    await page.goto('http://localhost:1420')
    await page.waitForLoadState('networkidle')
  })

  test('プロジェクト配下にタスクを作成できる（名前・状態）', async ({ page }) => {
    // 受け入れ基準1: プロジェクト配下にタスクを作成できる（名前・状態）
    
    // Step 1: 新しいプロジェクトを作成
    await page.getByRole('button', { name: '新しいプロジェクト' }).click()
    await page.getByPlaceholder('プロジェクト名').fill('テストプロジェクト')
    await page.getByRole('button', { name: '作成' }).click()
    
    // プロジェクトが作成されたことを確認
    await expect(page.getByTestId('project-card')).toContainText('テストプロジェクト')
    
    // Step 2: プロジェクトを選択してタスクを作成
    await page.getByTestId('project-card').click()
    await page.getByRole('button', { name: '新しいタスク' }).click()
    await page.getByPlaceholder('タスク名').fill('重要なタスク')
    await page.getByRole('button', { name: '作成' }).click()
    
    // タスクが作成されたことを確認
    await expect(page.getByTestId('task-item')).toContainText('重要なタスク')
    await expect(page.getByTestId('task-status')).toContainText('アクティブ')
  })

  test('タスクの状態をactive/archivedで切替できる', async ({ page }) => {
    // 受け入れ基準2: タスクの状態をactive/archivedで切替できる
    
    // 前提: プロジェクトとタスクを作成
    await page.getByRole('button', { name: '新しいプロジェクト' }).click()
    await page.getByPlaceholder('プロジェクト名').fill('状態管理プロジェクト')
    await page.getByRole('button', { name: '作成' }).click()
    
    await page.getByTestId('project-card').click()
    await page.getByRole('button', { name: '新しいタスク' }).click()
    await page.getByPlaceholder('タスク名').fill('状態切替タスク')
    await page.getByRole('button', { name: '作成' }).click()
    
    // Step 1: タスクをアーカイブ
    await page.getByTestId('task-item').getByRole('button', { name: 'アーカイブ' }).click()
    await expect(page.getByTestId('task-status')).toContainText('アーカイブ済み')
    
    // Step 2: タスクを復元
    await page.getByTestId('task-item').getByRole('button', { name: '復元' }).click()
    await expect(page.getByTestId('task-status')).toContainText('アクティブ')
  })

  test('階層（プロジェクト→タスク）がUIとデータで一貫している', async ({ page }) => {
    // 受け入れ基準3: 階層（プロジェクト→タスク）がUIとデータで一貫している
    
    // Step 1: 複数のプロジェクトとタスクを作成
    
    // プロジェクト1
    await page.getByRole('button', { name: '新しいプロジェクト' }).click()
    await page.getByPlaceholder('プロジェクト名').fill('プロジェクトA')
    await page.getByRole('button', { name: '作成' }).click()
    
    await page.getByTestId('project-card').filter({ hasText: 'プロジェクトA' }).click()
    await page.getByRole('button', { name: '新しいタスク' }).click()
    await page.getByPlaceholder('タスク名').fill('タスクA1')
    await page.getByRole('button', { name: '作成' }).click()
    
    // プロジェクト一覧に戻る
    await page.getByRole('button', { name: 'プロジェクト一覧に戻る' }).click()
    
    // プロジェクト2
    await page.getByRole('button', { name: '新しいプロジェクト' }).click()
    await page.getByPlaceholder('プロジェクト名').fill('プロジェクトB')
    await page.getByRole('button', { name: '作成' }).click()
    
    await page.getByTestId('project-card').filter({ hasText: 'プロジェクトB' }).click()
    await page.getByRole('button', { name: '新しいタスク' }).click()
    await page.getByPlaceholder('タスク名').fill('タスクB1')
    await page.getByRole('button', { name: '作成' }).click()
    
    // Step 2: 階層関係を確認
    
    // プロジェクトAの詳細を確認
    await page.getByRole('button', { name: 'プロジェクト一覧に戻る' }).click()
    await page.getByTestId('project-card').filter({ hasText: 'プロジェクトA' }).click()
    
    // プロジェクトAのタスクのみ表示されることを確認
    await expect(page.getByTestId('task-item')).toHaveCount(1)
    await expect(page.getByTestId('task-item')).toContainText('タスクA1')
    
    // プロジェクトBの詳細を確認
    await page.getByRole('button', { name: 'プロジェクト一覧に戻る' }).click()
    await page.getByTestId('project-card').filter({ hasText: 'プロジェクトB' }).click()
    
    // プロジェクトBのタスクのみ表示されることを確認
    await expect(page.getByTestId('task-item')).toHaveCount(1)
    await expect(page.getByTestId('task-item')).toContainText('タスクB1')
    
    // Step 3: タスクの移動機能をテスト
    await page.getByTestId('task-item').getByRole('button', { name: '編集' }).click()
    await page.getByLabel('プロジェクト').selectOption({ label: 'プロジェクトA' })
    await page.getByRole('button', { name: '保存' }).click()
    
    // タスクB1がプロジェクトBから消えたことを確認
    await expect(page.getByText('タスクがありません')).toBeVisible()
    
    // プロジェクトAに移動してタスクB1があることを確認
    await page.getByRole('button', { name: 'プロジェクト一覧に戻る' }).click()
    await page.getByTestId('project-card').filter({ hasText: 'プロジェクトA' }).click()
    await expect(page.getByTestId('task-item')).toHaveCount(2)
    await expect(page.locator('[data-testid="task-item"]')).toContainText(['タスクA1', 'タスクB1'])
  })

  test('エラーハンドリング: アーカイブ済みプロジェクトにタスクを作成できない', async ({ page }) => {
    // プロジェクトを作成してアーカイブ
    await page.getByRole('button', { name: '新しいプロジェクト' }).click()
    await page.getByPlaceholder('プロジェクト名').fill('アーカイブ対象プロジェクト')
    await page.getByRole('button', { name: '作成' }).click()
    
    await page.getByTestId('project-card').getByRole('button', { name: 'アーカイブ' }).click()
    
    // アーカイブ済みを表示
    await page.getByRole('button', { name: 'アーカイブ済みを表示' }).click()
    
    // アーカイブ済みプロジェクトをクリック
    await page.getByTestId('project-card').filter({ hasText: 'アーカイブ対象プロジェクト' }).click()
    
    // 新しいタスクボタンが無効化されていることを確認
    await expect(page.getByRole('button', { name: '新しいタスク' })).toBeDisabled()
    
    // または、エラーメッセージが表示されることを確認
    await expect(page.getByText('アーカイブ済みプロジェクトではタスクを作成できません')).toBeVisible()
  })
})

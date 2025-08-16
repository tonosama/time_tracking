import { test, expect } from '@playwright/test'

test.describe('タスク管理機能 - データベース統合テスト', () => {
  test.beforeEach(async ({ page }) => {
    // Tauriアプリケーションの起動を待機
    await page.goto('http://localhost:1420')
    await page.waitForLoadState('networkidle')
  })

  test('タスク作成からデータベース登録までの完全な流れ', async ({ page }) => {
    // Step 1: プロジェクトを作成
    await page.getByRole('button', { name: '+' }).click()
    await page.getByPlaceholderText('プロジェクト名を入力').fill('DB統合テストプロジェクト')
    await page.getByRole('button', { name: '作成' }).click()
    
    // プロジェクトが作成されたことを確認
    await expect(page.getByText('DB統合テストプロジェクト')).toBeInTheDocument()
    
    // Step 2: プロジェクトを選択してタスク作成画面に移動
    await page.getByText('DB統合テストプロジェクト').click()
    
    // Step 3: タスクを作成
    await page.getByRole('button', { name: '新しいタスク' }).click()
    await page.getByPlaceholderText('タスク名を入力').fill('DB統合テストタスク')
    await page.getByRole('button', { name: '作成' }).click()
    
    // タスクが作成されたことを確認
    await expect(page.getByText('DB統合テストタスク')).toBeInTheDocument()
    await expect(page.getByText('アクティブ')).toBeInTheDocument()
    
    // Step 4: ページをリロードしてデータベースからの読み込みを確認
    await page.reload()
    await page.waitForLoadState('networkidle')
    
    // プロジェクトを再選択
    await page.getByText('DB統合テストプロジェクト').click()
    
    // タスクが永続化されていることを確認
    await expect(page.getByText('DB統合テストタスク')).toBeInTheDocument()
    
    // クリーンアップ: タスクをアーカイブ
    await page.getByRole('button', { name: 'アーカイブ' }).first().click()
    
    // クリーンアップ: プロジェクトをアーカイブ
    await page.goBack()
    await page.getByRole('button', { name: 'アーカイブ' }).first().click()
  })

  test('タスク編集機能のデータベース永続化', async ({ page }) => {
    // 前提: プロジェクトとタスクを作成
    await page.getByRole('button', { name: '+' }).click()
    await page.getByPlaceholderText('プロジェクト名を入力').fill('編集テストプロジェクト')
    await page.getByRole('button', { name: '作成' }).click()
    
    await page.getByText('編集テストプロジェクト').click()
    await page.getByRole('button', { name: '新しいタスク' }).click()
    await page.getByPlaceholderText('タスク名を入力').fill('編集前のタスク名')
    await page.getByRole('button', { name: '作成' }).click()
    
    // Step 1: タスクを編集
    await page.getByRole('button', { name: '編集' }).click()
    await page.getByDisplayValue('編集前のタスク名').fill('編集後のタスク名')
    await page.getByRole('button', { name: '保存' }).click()
    
    // 編集が反映されたことを確認
    await expect(page.getByText('編集後のタスク名')).toBeInTheDocument()
    
    // Step 2: ページをリロードしてデータベースからの読み込みを確認
    await page.reload()
    await page.waitForLoadState('networkidle')
    
    await page.getByText('編集テストプロジェクト').click()
    
    // 編集内容が永続化されていることを確認
    await expect(page.getByText('編集後のタスク名')).toBeInTheDocument()
    await expect(page.getByText('編集前のタスク名')).not.toBeInTheDocument()
    
    // クリーンアップ
    await page.getByRole('button', { name: 'アーカイブ' }).first().click()
    await page.goBack()
    await page.getByRole('button', { name: 'アーカイブ' }).first().click()
  })

  test('タスクのアーカイブ・復元機能のデータベース永続化', async ({ page }) => {
    // 前提: プロジェクトとタスクを作成
    await page.getByRole('button', { name: '+' }).click()
    await page.getByPlaceholderText('プロジェクト名を入力').fill('アーカイブテストプロジェクト')
    await page.getByRole('button', { name: '作成' }).click()
    
    await page.getByText('アーカイブテストプロジェクト').click()
    await page.getByRole('button', { name: '新しいタスク' }).click()
    await page.getByPlaceholderText('タスク名を入力').fill('アーカイブ対象タスク')
    await page.getByRole('button', { name: '作成' }).click()
    
    // Step 1: タスクをアーカイブ
    await page.getByRole('button', { name: 'アーカイブ' }).click()
    await expect(page.getByText('アーカイブ済み')).toBeInTheDocument()
    
    // Step 2: ページをリロードしてアーカイブ状態が永続化されていることを確認
    await page.reload()
    await page.waitForLoadState('networkidle')
    
    await page.getByText('アーカイブテストプロジェクト').click()
    await expect(page.getByText('アーカイブ済み')).toBeInTheDocument()
    
    // Step 3: タスクを復元
    await page.getByRole('button', { name: '復元' }).click()
    await expect(page.getByText('アクティブ')).toBeInTheDocument()
    
    // Step 4: ページをリロードして復元状態が永続化されていることを確認
    await page.reload()
    await page.waitForLoadState('networkidle')
    
    await page.getByText('アーカイブテストプロジェクト').click()
    await expect(page.getByText('アクティブ')).toBeInTheDocument()
    
    // クリーンアップ
    await page.getByRole('button', { name: 'アーカイブ' }).first().click()
    await page.goBack()
    await page.getByRole('button', { name: 'アーカイブ' }).first().click()
  })

  test('タスクの階層管理とプロジェクト間移動のデータベース永続化', async ({ page }) => {
    // 前提: 複数のプロジェクトを作成
    await page.getByRole('button', { name: '+' }).click()
    await page.getByPlaceholderText('プロジェクト名を入力').fill('プロジェクトA')
    await page.getByRole('button', { name: '作成' }).click()
    
    await page.goBack()
    await page.getByRole('button', { name: '+' }).click()
    await page.getByPlaceholderText('プロジェクト名を入力').fill('プロジェクトB')
    await page.getByRole('button', { name: '作成' }).click()
    
    // Step 1: プロジェクトAにタスクを作成
    await page.getByText('プロジェクトA').click()
    await page.getByRole('button', { name: '新しいタスク' }).click()
    await page.getByPlaceholderText('タスク名を入力').fill('移動対象タスク')
    await page.getByRole('button', { name: '作成' }).click()
    
    // Step 2: タスクをプロジェクトBに移動
    await page.getByRole('button', { name: '編集' }).click()
    await page.getByLabel('プロジェクト').selectOption({ label: 'プロジェクトB' })
    await page.getByRole('button', { name: '保存' }).click()
    
    // プロジェクトAからタスクが消えたことを確認
    await expect(page.getByText('まだタスクがありません')).toBeInTheDocument()
    
    // Step 3: プロジェクトBに移動してタスクがあることを確認
    await page.goBack()
    await page.getByText('プロジェクトB').click()
    await expect(page.getByText('移動対象タスク')).toBeInTheDocument()
    
    // Step 4: ページをリロードして移動が永続化されていることを確認
    await page.reload()
    await page.waitForLoadState('networkidle')
    
    await page.getByText('プロジェクトA').click()
    await expect(page.getByText('まだタスクがありません')).toBeInTheDocument()
    
    await page.goBack()
    await page.getByText('プロジェクトB').click()
    await expect(page.getByText('移動対象タスク')).toBeInTheDocument()
    
    // クリーンアップ
    await page.getByRole('button', { name: 'アーカイブ' }).first().click()
    await page.goBack()
    await page.getByRole('button', { name: 'アーカイブ' }).first().click()
    await page.goBack()
    await page.getByRole('button', { name: 'アーカイブ' }).first().click()
  })

  test('バリデーション機能の動作確認', async ({ page }) => {
    // 前提: プロジェクトを作成
    await page.getByRole('button', { name: '+' }).click()
    await page.getByPlaceholderText('プロジェクト名を入力').fill('バリデーションテストプロジェクト')
    await page.getByRole('button', { name: '作成' }).click()
    
    await page.getByText('バリデーションテストプロジェクト').click()
    
    // Step 1: 空のタスク名で作成を試行
    await page.getByRole('button', { name: '新しいタスク' }).click()
    await page.getByRole('button', { name: '作成' }).click()
    
    // エラーメッセージが表示されることを確認
    await expect(page.getByText('タスク名を入力してください')).toBeInTheDocument()
    
    // Step 2: 空白のみのタスク名で作成を試行
    await page.getByPlaceholderText('タスク名を入力').fill('   ')
    await page.getByRole('button', { name: '作成' }).click()
    
    // エラーメッセージが表示されることを確認
    await expect(page.getByText('タスク名を入力してください')).toBeInTheDocument()
    
    // Step 3: 正常なタスク名で作成
    await page.getByPlaceholderText('タスク名を入力').fill('正常なタスク')
    await page.getByRole('button', { name: '作成' }).click()
    
    // タスクが作成されることを確認
    await expect(page.getByText('正常なタスク')).toBeInTheDocument()
    
    // クリーンアップ
    await page.getByRole('button', { name: 'アーカイブ' }).first().click()
    await page.goBack()
    await page.getByRole('button', { name: 'アーカイブ' }).first().click()
  })

  test('アーカイブ済みプロジェクトでのタスク作成制限', async ({ page }) => {
    // 前提: プロジェクトを作成してアーカイブ
    await page.getByRole('button', { name: '+' }).click()
    await page.getByPlaceholderText('プロジェクト名を入力').fill('アーカイブ済みプロジェクト')
    await page.getByRole('button', { name: '作成' }).click()
    
    await page.getByRole('button', { name: 'アーカイブ' }).first().click()
    
    // アーカイブ済みプロジェクトをクリック
    await page.getByText('アーカイブ済みプロジェクト').click()
    
    // 新しいタスクボタンが無効化されていることを確認
    await expect(page.getByRole('button', { name: '新しいタスク' })).toBeDisabled()
    
    // または、エラーメッセージが表示されることを確認
    await expect(page.getByText('アーカイブ済みプロジェクトではタスクを作成できません')).toBeVisible()
  })

  test('複数タスクの一括操作とデータベース整合性', async ({ page }) => {
    // 前提: プロジェクトを作成
    await page.getByRole('button', { name: '+' }).click()
    await page.getByPlaceholderText('プロジェクト名を入力').fill('一括操作テストプロジェクト')
    await page.getByRole('button', { name: '作成' }).click()
    
    // 複数のタスクを作成
    const taskNames = ['タスク1', 'タスク2', 'タスク3', 'タスク4', 'タスク5']
    
    for (const taskName of taskNames) {
      await page.getByRole('button', { name: '新しいタスク' }).click()
      await page.getByPlaceholderText('タスク名を入力').fill(taskName)
      await page.getByRole('button', { name: '作成' }).click()
    }
    
    // すべてのタスクが表示されることを確認
    for (const taskName of taskNames) {
      await expect(page.getByText(taskName)).toBeInTheDocument()
    }
    
    // ページをリロードしてデータベースからの読み込みを確認
    await page.reload()
    await page.waitForLoadState('networkidle')
    
    await page.getByText('一括操作テストプロジェクト').click()
    
    // すべてのタスクが永続化されていることを確認
    for (const taskName of taskNames) {
      await expect(page.getByText(taskName)).toBeInTheDocument()
    }
    
    // 複数のタスクをアーカイブ
    const archiveButtons = page.getByRole('button', { name: 'アーカイブ' })
    const count = await archiveButtons.count()
    
    for (let i = 0; i < Math.min(count, 3); i++) {
      await archiveButtons.nth(i).click()
    }
    
    // ページをリロードしてアーカイブ状態が永続化されていることを確認
    await page.reload()
    await page.waitForLoadState('networkidle')
    
    await page.getByText('一括操作テストプロジェクト').click()
    
    // アーカイブ済みタスクが表示されることを確認
    await expect(page.getByText('アーカイブ済み')).toBeInTheDocument()
    
    // クリーンアップ: 残りのタスクをアーカイブ
    const remainingArchiveButtons = page.getByRole('button', { name: 'アーカイブ' })
    const remainingCount = await remainingArchiveButtons.count()
    
    for (let i = 0; i < remainingCount; i++) {
      await remainingArchiveButtons.nth(0).click()
    }
    
    await page.goBack()
    await page.getByRole('button', { name: 'アーカイブ' }).first().click()
  })

  test('タスク履歴のデータベース永続化', async ({ page }) => {
    // 前提: プロジェクトとタスクを作成
    await page.getByRole('button', { name: '+' }).click()
    await page.getByPlaceholderText('プロジェクト名を入力').fill('履歴テストプロジェクト')
    await page.getByRole('button', { name: '作成' }).click()
    
    await page.getByText('履歴テストプロジェクト').click()
    await page.getByRole('button', { name: '新しいタスク' }).click()
    await page.getByPlaceholderText('タスク名を入力').fill('履歴テストタスク')
    await page.getByRole('button', { name: '作成' }).click()
    
    // タスクを複数回編集
    const editNames = ['編集1', '編集2', '編集3', '最終編集']
    
    for (const editName of editNames) {
      await page.getByRole('button', { name: '編集' }).click()
      await page.getByDisplayValue(/.*/).fill(editName)
      await page.getByRole('button', { name: '保存' }).click()
      
      // 編集が反映されたことを確認
      await expect(page.getByText(editName)).toBeInTheDocument()
    }
    
    // ページをリロードして最終状態が永続化されていることを確認
    await page.reload()
    await page.waitForLoadState('networkidle')
    
    await page.getByText('履歴テストプロジェクト').click()
    await expect(page.getByText('最終編集')).toBeInTheDocument()
    
    // クリーンアップ
    await page.getByRole('button', { name: 'アーカイブ' }).first().click()
    await page.goBack()
    await page.getByRole('button', { name: 'アーカイブ' }).first().click()
  })
})

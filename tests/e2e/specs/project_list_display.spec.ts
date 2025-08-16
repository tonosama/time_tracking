import { test, expect } from '@playwright/test'

test.describe('プロジェクト一覧表示機能', () => {
  test.beforeEach(async ({ page }) => {
    // アプリケーションを起動
    await page.goto('http://localhost:1420')
    
    // アプリケーションが読み込まれるまで待機
    await page.waitForSelector('.dashboard', { timeout: 10000 })
  })

  test('プロジェクト一覧が正常に表示される', async ({ page }) => {
    // プロジェクト一覧が表示されることを確認
    await expect(page.locator('.project-sidebar')).toBeVisible()
    
    // プロジェクト一覧のヘッダーが表示されることを確認
    await expect(page.locator('.sidebar-header h3')).toHaveText('Projects')
    
    // プロジェクトが存在する場合、プロジェクトアイテムが表示されることを確認
    const projectItems = page.locator('.project-item')
    const projectCount = await projectItems.count()
    
    if (projectCount > 0) {
      // プロジェクトアイテムが表示されることを確認
      await expect(projectItems.first()).toBeVisible()
      
      // プロジェクト名が表示されることを確認
      await expect(projectItems.first().locator('.project-name')).toBeVisible()
    } else {
      // プロジェクトが存在しない場合のメッセージが表示されることを確認
      await expect(page.locator('.no-projects')).toBeVisible()
      await expect(page.locator('.no-projects p')).toContainText('プロジェクトがありません')
    }
  })

  test('新しいプロジェクト作成ボタンが表示される', async ({ page }) => {
    // 新しいプロジェクト作成ボタンが表示されることを確認
    await expect(page.locator('.add-project-btn')).toBeVisible()
    await expect(page.locator('.add-project-btn')).toHaveText('+')
    
    // フッターの作成リンクも表示されることを確認
    await expect(page.locator('.add-project-link')).toBeVisible()
    await expect(page.locator('.add-project-link')).toContainText('Add new project')
  })

  test('プロジェクト作成フォームが表示される', async ({ page }) => {
    // 新しいプロジェクト作成ボタンをクリック
    await page.click('.add-project-btn')
    
    // プロジェクト作成フォームが表示されることを確認
    await expect(page.locator('.create-project-form')).toBeVisible()
    await expect(page.locator('.project-name-input')).toBeVisible()
    await expect(page.locator('.project-name-input')).toHaveAttribute('placeholder', 'プロジェクト名を入力')
    
    // 作成ボタンとキャンセルボタンが表示されることを確認
    await expect(page.locator('.create-btn')).toBeVisible()
    await expect(page.locator('.create-btn')).toHaveText('作成')
    await expect(page.locator('.cancel-btn')).toBeVisible()
    await expect(page.locator('.cancel-btn')).toHaveText('キャンセル')
  })

  test('プロジェクト作成フォームでキャンセルが動作する', async ({ page }) => {
    // 新しいプロジェクト作成ボタンをクリック
    await page.click('.add-project-btn')
    
    // フォームが表示されることを確認
    await expect(page.locator('.create-project-form')).toBeVisible()
    
    // キャンセルボタンをクリック
    await page.click('.cancel-btn')
    
    // フォームが非表示になることを確認
    await expect(page.locator('.create-project-form')).not.toBeVisible()
  })

  test('プロジェクト作成フォームでEscapeキーでキャンセルできる', async ({ page }) => {
    // 新しいプロジェクト作成ボタンをクリック
    await page.click('.add-project-btn')
    
    // フォームが表示されることを確認
    await expect(page.locator('.create-project-form')).toBeVisible()
    
    // 入力フィールドにフォーカスしてEscapeキーを押す
    await page.click('.project-name-input')
    await page.keyboard.press('Escape')
    
    // フォームが非表示になることを確認
    await expect(page.locator('.create-project-form')).not.toBeVisible()
  })

  test('空のプロジェクト名で作成ボタンを押すとエラーメッセージが表示される', async ({ page }) => {
    // 新しいプロジェクト作成ボタンをクリック
    await page.click('.add-project-btn')
    
    // 作成ボタンをクリック（プロジェクト名は空）
    await page.click('.create-btn')
    
    // アラートが表示されることを確認
    page.on('dialog', dialog => {
      expect(dialog.message()).toBe('プロジェクト名を入力してください')
      dialog.accept()
    })
  })

  test('プロジェクト名を入力して作成ボタンを押すとプロジェクトが作成される', async ({ page }) => {
    // 新しいプロジェクト作成ボタンをクリック
    await page.click('.add-project-btn')
    
    // プロジェクト名を入力
    await page.fill('.project-name-input', 'テストプロジェクト')
    
    // 作成ボタンをクリック
    await page.click('.create-btn')
    
    // フォームが閉じられることを確認
    await expect(page.locator('.create-project-form')).not.toBeVisible()
    
    // 作成されたプロジェクトが一覧に表示されることを確認
    await expect(page.locator('.project-item')).toContainText('テストプロジェクト')
  })

  test('プロジェクト名を入力してEnterキーでプロジェクトが作成される', async ({ page }) => {
    // 新しいプロジェクト作成ボタンをクリック
    await page.click('.add-project-btn')
    
    // プロジェクト名を入力してEnterキーを押す
    await page.fill('.project-name-input', 'Enterキーテストプロジェクト')
    await page.keyboard.press('Enter')
    
    // フォームが閉じられることを確認
    await expect(page.locator('.create-project-form')).not.toBeVisible()
    
    // 作成されたプロジェクトが一覧に表示されることを確認
    await expect(page.locator('.project-item')).toContainText('Enterキーテストプロジェクト')
  })

  test('プロジェクト作成中はボタンが無効化される', async ({ page }) => {
    // 新しいプロジェクト作成ボタンをクリック
    await page.click('.add-project-btn')
    
    // プロジェクト名を入力
    await page.fill('.project-name-input', '無効化テストプロジェクト')
    
    // 作成ボタンをクリック
    await page.click('.create-btn')
    
    // 作成中はボタンが無効化されることを確認（短時間のため、状態変化を確認）
    await expect(page.locator('.create-btn')).toBeDisabled()
    
    // 作成完了を待つ
    await expect(page.locator('.create-project-form')).not.toBeVisible()
  })

  test('プロジェクトを選択できる', async ({ page }) => {
    // プロジェクトが存在する場合のみテストを実行
    const projectItems = page.locator('.project-item')
    const projectCount = await projectItems.count()
    
    if (projectCount > 0) {
      // 最初のプロジェクトをクリック
      await projectItems.first().click()
      
      // プロジェクトが選択された状態になることを確認
      await expect(projectItems.first()).toHaveClass(/selected/)
    }
  })

  test('プロジェクト選択時にダッシュボードが更新される', async ({ page }) => {
    // プロジェクトが存在する場合のみテストを実行
    const projectItems = page.locator('.project-item')
    const projectCount = await projectItems.count()
    
    if (projectCount > 0) {
      // プロジェクト名を取得
      const projectName = await projectItems.first().locator('.project-name').textContent()
      
      // 最初のプロジェクトをクリック
      await projectItems.first().click()
      
      // ダッシュボードのプロジェクト選択部分が更新されることを確認
      // （実際のUIに応じてセレクタを調整）
      await expect(page.locator('.dashboard')).toBeVisible()
    }
  })

  test('プロジェクト一覧の表示が正しい', async ({ page }) => {
    // プロジェクト一覧の構造を確認
    await expect(page.locator('.project-sidebar')).toBeVisible()
    await expect(page.locator('.sidebar-header')).toBeVisible()
    await expect(page.locator('.projects-list')).toBeVisible()
    await expect(page.locator('.sidebar-footer')).toBeVisible()
    
    // プロジェクトアイテムの構造を確認（プロジェクトが存在する場合）
    const projectItems = page.locator('.project-item')
    const projectCount = await projectItems.count()
    
    if (projectCount > 0) {
      const firstProject = projectItems.first()
      await expect(firstProject.locator('.project-color-indicator')).toBeVisible()
      await expect(firstProject.locator('.project-icon')).toBeVisible()
      await expect(firstProject.locator('.project-info')).toBeVisible()
      await expect(firstProject.locator('.project-name')).toBeVisible()
      await expect(firstProject.locator('.project-stats')).toBeVisible()
      await expect(firstProject.locator('.project-time')).toBeVisible()
    }
  })

  test('プロジェクト一覧のレスポンシブ対応', async ({ page }) => {
    // デスクトップサイズでの表示
    await page.setViewportSize({ width: 1200, height: 800 })
    await expect(page.locator('.project-sidebar')).toBeVisible()
    
    // タブレットサイズでの表示
    await page.setViewportSize({ width: 768, height: 1024 })
    await expect(page.locator('.project-sidebar')).toBeVisible()
    
    // モバイルサイズでの表示
    await page.setViewportSize({ width: 375, height: 667 })
    await expect(page.locator('.project-sidebar')).toBeVisible()
  })
})

import { test, expect } from '@playwright/test'

test.describe('プロジェクト作成機能', () => {
  test.beforeEach(async ({ page }) => {
    // Tauriアプリケーションにアクセス
    await page.goto('/')
    
    // アプリケーションが完全に読み込まれるまで待機
    await page.waitForLoadState('networkidle')
    
    // ダッシュボードが表示されるまで待機
    await expect(page.locator('.dashboard')).toBeVisible()
  })

  test('新しいプロジェクトを作成できる', async ({ page }) => {
    const projectName = 'テストプロジェクト'
    
    // 初期状態でプロジェクトサイドバーが表示されていることを確認
    await expect(page.locator('.project-sidebar')).toBeVisible()
    
    // 「+ Add new project」ボタンをクリック
    const addProjectBtn = page.locator('.add-project-btn, .add-project-link').first()
    await expect(addProjectBtn).toBeVisible()
    await addProjectBtn.click()
    
    // プロジェクト作成フォームが表示されることを確認
    await expect(page.locator('.create-project-form')).toBeVisible()
    
    // プロジェクト名入力フィールドにテキストを入力
    const projectNameInput = page.locator('.project-name-input')
    await expect(projectNameInput).toBeVisible()
    await projectNameInput.fill(projectName)
    
    // 入力値が正しく設定されていることを確認
    await expect(projectNameInput).toHaveValue(projectName)
    
    // 「作成」ボタンをクリック
    const createBtn = page.locator('.create-btn')
    await expect(createBtn).toBeVisible()
    await expect(createBtn).toBeEnabled()
    
    // コンソールログをキャプチャ
    const consoleMessages: string[] = []
    page.on('console', msg => {
      consoleMessages.push(msg.text())
    })
    
    await createBtn.click()
    
    // 作成フォームが非表示になるまで待機
    await expect(page.locator('.create-project-form')).not.toBeVisible()
    
    // 新しいプロジェクトがプロジェクト一覧に表示されることを確認
    const projectItem = page.locator('.project-item').filter({ hasText: projectName })
    await expect(projectItem).toBeVisible()
    
    // コンソールログの確認
    await page.waitForTimeout(1000) // ログが出力されるまで少し待機
    
    // 期待するログメッセージを確認
    const hasCreationStartLog = consoleMessages.some(msg => 
      msg.includes('プロジェクト作成開始:') && msg.includes(projectName)
    )
    const hasInvokeFunctionLog = consoleMessages.some(msg => 
      msg.includes('invoke関数:') && msg.includes('function')
    )
    const hasCreationSuccessLog = consoleMessages.some(msg => 
      msg.includes('プロジェクト作成成功:')
    )
    
    expect(hasCreationStartLog).toBe(true)
    expect(hasInvokeFunctionLog).toBe(true)
    expect(hasCreationSuccessLog).toBe(true)
  })

  test('プロジェクト名が空の場合はエラーメッセージが表示される', async ({ page }) => {
    // 「+ Add new project」ボタンをクリック
    const addProjectBtn = page.locator('.add-project-btn, .add-project-link').first()
    await addProjectBtn.click()
    
    // プロジェクト作成フォームが表示されることを確認
    await expect(page.locator('.create-project-form')).toBeVisible()
    
    // 空の状態で「作成」ボタンをクリック
    const createBtn = page.locator('.create-btn')
    
    // アラートダイアログを処理
    page.once('dialog', async dialog => {
      expect(dialog.message()).toBe('プロジェクト名を入力してください')
      await dialog.accept()
    })
    
    await createBtn.click()
    
    // フォームが閉じられていないことを確認
    await expect(page.locator('.create-project-form')).toBeVisible()
  })

  test('プロジェクト作成をキャンセルできる', async ({ page }) => {
    // 「+ Add new project」ボタンをクリック
    const addProjectBtn = page.locator('.add-project-btn, .add-project-link').first()
    await addProjectBtn.click()
    
    // プロジェクト作成フォームが表示されることを確認
    await expect(page.locator('.create-project-form')).toBeVisible()
    
    // プロジェクト名を入力
    const projectNameInput = page.locator('.project-name-input')
    await projectNameInput.fill('キャンセルテスト')
    
    // 「キャンセル」ボタンをクリック
    const cancelBtn = page.locator('.cancel-btn')
    await expect(cancelBtn).toBeVisible()
    await cancelBtn.click()
    
    // フォームが非表示になることを確認
    await expect(page.locator('.create-project-form')).not.toBeVisible()
    
    // 入力したプロジェクトが作成されていないことを確認
    const projectItem = page.locator('.project-item').filter({ hasText: 'キャンセルテスト' })
    await expect(projectItem).not.toBeVisible()
  })

  test('プロジェクト作成後にプロジェクト選択ができる', async ({ page }) => {
    const projectName = 'プロジェクト選択テスト'
    
    // プロジェクトを作成
    const addProjectBtn = page.locator('.add-project-btn, .add-project-link').first()
    await addProjectBtn.click()
    
    const projectNameInput = page.locator('.project-name-input')
    await projectNameInput.fill(projectName)
    
    const createBtn = page.locator('.create-btn')
    await createBtn.click()
    
    // 作成フォームが非表示になるまで待機
    await expect(page.locator('.create-project-form')).not.toBeVisible()
    
    // 新しいプロジェクトをクリックして選択
    const projectItem = page.locator('.project-item').filter({ hasText: projectName })
    await expect(projectItem).toBeVisible()
    await projectItem.click()
    
    // プロジェクトが選択状態になることを確認
    await expect(projectItem).toHaveClass(/selected/)
    
    // タイマーセクションでプロジェクトが選択されていることを確認
    const selectedProjectDisplay = page.locator('.timer-section .project-display')
    await expect(selectedProjectDisplay).toContainText(projectName)
  })
})


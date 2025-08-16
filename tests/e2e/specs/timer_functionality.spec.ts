import { test, expect } from '@playwright/test'

test.describe('タイマー機能', () => {
  test.beforeEach(async ({ page }) => {
    // Tauriアプリケーションにアクセス
    await page.goto('/')
    
    // アプリケーションが完全に読み込まれるまで待機
    await page.waitForLoadState('networkidle')
    
    // ダッシュボードが表示されるまで待機
    await expect(page.locator('.dashboard')).toBeVisible()
  })

  test('タイマーを開始・停止できる', async ({ page }) => {
    const projectName = 'タイマーテストプロジェクト'
    const taskDescription = 'テストタスクの作業'
    
    // まずプロジェクトを作成
    const addProjectBtn = page.locator('.add-project-btn, .add-project-link').first()
    await addProjectBtn.click()
    
    const projectNameInput = page.locator('.project-name-input')
    await projectNameInput.fill(projectName)
    
    const createBtn = page.locator('.create-btn')
    await createBtn.click()
    
    // プロジェクト作成完了まで待機
    await expect(page.locator('.create-project-form')).not.toBeVisible()
    
    // 作成したプロジェクトを選択
    const projectItem = page.locator('.project-item').filter({ hasText: projectName })
    await expect(projectItem).toBeVisible()
    await projectItem.click()
    
    // プロジェクトが選択状態になることを確認
    await expect(projectItem).toHaveClass(/selected/)
    
    // タスク説明を入力
    const taskInput = page.locator('input[placeholder*="What are you working on"]')
    await expect(taskInput).toBeVisible()
    await taskInput.fill(taskDescription)
    
    // コンソールログをキャプチャ
    const consoleMessages: string[] = []
    page.on('console', msg => {
      consoleMessages.push(msg.text())
    })
    
    // タイマー開始ボタンをクリック
    const startBtn = page.locator('.start-btn, .timer-start-btn').first()
    await expect(startBtn).toBeVisible()
    await expect(startBtn).toBeEnabled()
    await startBtn.click()
    
    // タイマーが開始されたことを確認（ヘッダーに実行中タイマーが表示される）
    await expect(page.locator('.running-timer-header')).toBeVisible()
    
    // 実行中タイマーにタスク情報が表示されることを確認
    await expect(page.locator('.running-timer-header')).toContainText(taskDescription)
    
    // コンソールログの確認
    await page.waitForTimeout(1000) // ログが出力されるまで少し待機
    
    const hasTimerStartLog = consoleMessages.some(msg => 
      msg.includes('タイマー開始ボタンがクリックされました')
    )
    const hasProjectSelectionLog = consoleMessages.some(msg => 
      msg.includes('選択されたプロジェクト:')
    )
    
    expect(hasTimerStartLog).toBe(true)
    expect(hasProjectSelectionLog).toBe(true)
    
    // タイマーを停止
    const stopBtn = page.locator('.stop-btn, .timer-stop-btn').first()
    await expect(stopBtn).toBeVisible()
    await stopBtn.click()
    
    // 実行中タイマーが非表示になることを確認
    await expect(page.locator('.running-timer-header')).not.toBeVisible()
  })

  test('プロジェクト未選択時にタイマー開始するとアラートが表示される', async ({ page }) => {
    // タスク説明のみ入力
    const taskInput = page.locator('input[placeholder*="What are you working on"]')
    await taskInput.fill('プロジェクト未選択テスト')
    
    // アラートダイアログを処理
    page.once('dialog', async dialog => {
      expect(dialog.message()).toBe('プロジェクトを選択してください')
      await dialog.accept()
    })
    
    // タイマー開始ボタンをクリック
    const startBtn = page.locator('.start-btn, .timer-start-btn').first()
    await startBtn.click()
    
    // 実行中タイマーが表示されないことを確認
    await expect(page.locator('.running-timer-header')).not.toBeVisible()
  })

  test('タイマー実行中に別のタスクを開始すると前のタイマーが停止される', async ({ page }) => {
    const project1Name = 'プロジェクト1'
    const project2Name = 'プロジェクト2'
    const task1Description = 'タスク1の作業'
    const task2Description = 'タスク2の作業'
    
    // プロジェクト1を作成
    let addProjectBtn = page.locator('.add-project-btn, .add-project-link').first()
    await addProjectBtn.click()
    
    let projectNameInput = page.locator('.project-name-input')
    await projectNameInput.fill(project1Name)
    
    let createBtn = page.locator('.create-btn')
    await createBtn.click()
    
    await expect(page.locator('.create-project-form')).not.toBeVisible()
    
    // プロジェクト2を作成
    addProjectBtn = page.locator('.add-project-btn, .add-project-link').first()
    await addProjectBtn.click()
    
    projectNameInput = page.locator('.project-name-input')
    await projectNameInput.fill(project2Name)
    
    createBtn = page.locator('.create-btn')
    await createBtn.click()
    
    await expect(page.locator('.create-project-form')).not.toBeVisible()
    
    // プロジェクト1でタイマー開始
    const project1Item = page.locator('.project-item').filter({ hasText: project1Name })
    await project1Item.click()
    
    const taskInput = page.locator('input[placeholder*="What are you working on"]')
    await taskInput.fill(task1Description)
    
    const startBtn = page.locator('.start-btn, .timer-start-btn').first()
    await startBtn.click()
    
    // タイマーが開始されたことを確認
    await expect(page.locator('.running-timer-header')).toBeVisible()
    await expect(page.locator('.running-timer-header')).toContainText(task1Description)
    
    // プロジェクト2でタイマー開始
    const project2Item = page.locator('.project-item').filter({ hasText: project2Name })
    await project2Item.click()
    
    await taskInput.fill(task2Description)
    await startBtn.click()
    
    // 新しいタイマーが開始され、タスク2の情報が表示されることを確認
    await expect(page.locator('.running-timer-header')).toBeVisible()
    await expect(page.locator('.running-timer-header')).toContainText(task2Description)
    await expect(page.locator('.running-timer-header')).not.toContainText(task1Description)
  })

  test('時間エントリーが記録される', async ({ page }) => {
    const projectName = '時間記録テストプロジェクト'
    const taskDescription = '時間記録テストタスク'
    
    // プロジェクトを作成
    const addProjectBtn = page.locator('.add-project-btn, .add-project-link').first()
    await addProjectBtn.click()
    
    const projectNameInput = page.locator('.project-name-input')
    await projectNameInput.fill(projectName)
    
    const createBtn = page.locator('.create-btn')
    await createBtn.click()
    
    await expect(page.locator('.create-project-form')).not.toBeVisible()
    
    // プロジェクトを選択してタイマーを開始
    const projectItem = page.locator('.project-item').filter({ hasText: projectName })
    await projectItem.click()
    
    const taskInput = page.locator('input[placeholder*="What are you working on"]')
    await taskInput.fill(taskDescription)
    
    const startBtn = page.locator('.start-btn, .timer-start-btn').first()
    await startBtn.click()
    
    // タイマーが開始されたことを確認
    await expect(page.locator('.running-timer-header')).toBeVisible()
    
    // 少し待ってからタイマーを停止
    await page.waitForTimeout(2000)
    
    const stopBtn = page.locator('.stop-btn, .timer-stop-btn').first()
    await stopBtn.click()
    
    // 時間エントリー一覧で記録が確認できることを確認
    await expect(page.locator('.time-entries-section')).toBeVisible()
    
    // 今日の時間エントリーが表示されることを確認
    const todaySection = page.locator('.time-entry-group').first()
    await expect(todaySection).toBeVisible()
    
    // 作成したタスクのエントリーが表示されることを確認
    const timeEntry = todaySection.locator('.time-entry-item').filter({ hasText: taskDescription })
    await expect(timeEntry).toBeVisible()
    
    // プロジェクト名も表示されることを確認
    await expect(timeEntry).toContainText(projectName)
  })
})


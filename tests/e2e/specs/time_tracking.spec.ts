import { test, expect } from '@playwright/test'

test.describe('US-002: タスクごとにタイマーを開始/停止できる', () => {
  test.beforeEach(async ({ page }) => {
    // Tauriアプリケーションの起動を待機
    await page.goto('http://localhost:1420')
    await page.waitForLoadState('networkidle')
    
    // テスト用プロジェクトとタスクを作成
    await page.getByRole('button', { name: '新しいプロジェクト' }).click()
    await page.getByPlaceholder('プロジェクト名').fill('タイムトラッキングテスト')
    await page.getByRole('button', { name: '作成' }).click()
    
    await page.getByTestId('project-card').click()
    await page.getByRole('button', { name: '新しいタスク' }).click()
    await page.getByPlaceholder('タスク名').fill('開発作業')
    await page.getByRole('button', { name: '作成' }).click()
  })

  test('start/stop操作がイベントとして保存される', async ({ page }) => {
    // 受け入れ基準1: start/stop操作がイベントとして保存される（at, task_id, event_type）
    
    // Step 1: タイマーを開始
    await page.getByTestId('task-item').getByRole('button', { name: 'タイマー開始' }).click()
    
    // タイマーが開始されたことを確認
    await expect(page.getByTestId('timer-status')).toContainText('実行中')
    await expect(page.getByTestId('task-item')).toHaveClass(/.*task-running.*/)
    
    // 少し待ってからタイマーを停止
    await page.waitForTimeout(2000)
    await page.getByTestId('task-item').getByRole('button', { name: 'タイマー停止' }).click()
    
    // タイマーが停止されたことを確認
    await expect(page.getByTestId('timer-status')).toContainText('停止中')
    await expect(page.getByTestId('task-item')).not.toHaveClass(/.*task-running.*/)
    
    // タイム履歴が記録されていることを確認
    await page.getByRole('button', { name: 'タイム履歴' }).click()
    await expect(page.getByTestId('time-entry')).toBeVisible()
    
    // 開始・停止イベントが保存されていることを確認
    const timeEntries = page.getByTestId('time-entry')
    await expect(timeEntries).toHaveCount(1)
    await expect(timeEntries.first()).toContainText('開発作業')
    await expect(timeEntries.first()).toContainText(/\d{2}:\d{2}:\d{2}/) // 時間が表示されている
  })

  test('同一タスクの連続start時は暗黙stopで直前区間が閉じる', async ({ page }) => {
    // 受け入れ基準2: 同一タスクの連続start時は暗黙stopで直前区間が閉じる
    
    // Step 1: 最初のタイマーを開始
    await page.getByTestId('task-item').getByRole('button', { name: 'タイマー開始' }).click()
    await expect(page.getByTestId('timer-status')).toContainText('実行中')
    
    // Step 2: 少し待ってから再度開始ボタンを押す（暗黙stop + start）
    await page.waitForTimeout(1000)
    await page.getByTestId('task-item').getByRole('button', { name: 'タイマー開始' }).click()
    
    // まだ実行中状態であることを確認
    await expect(page.getByTestId('timer-status')).toContainText('実行中')
    
    // Step 3: タイマーを停止
    await page.waitForTimeout(1000)
    await page.getByTestId('task-item').getByRole('button', { name: 'タイマー停止' }).click()
    
    // Step 4: タイム履歴を確認 - 2つの区間が記録されているはず
    await page.getByRole('button', { name: 'タイム履歴' }).click()
    const timeEntries = page.getByTestId('time-entry')
    await expect(timeEntries).toHaveCount(2)
    
    // 両方の区間に終了時間が記録されていることを確認
    for (let i = 0; i < 2; i++) {
      await expect(timeEntries.nth(i)).toContainText(/\d{2}:\d{2}:\d{2}/)
      await expect(timeEntries.nth(i)).toContainText('完了')
    }
  })

  test('実行中のタスクがUIで視覚的に分かる', async ({ page }) => {
    // 受け入れ基準3: 実行中のタスクがUIで視覚的に分かる
    
    // Step 1: 初期状態では実行中のタスクがないことを確認
    await expect(page.getByTestId('task-item')).not.toHaveClass(/.*task-running.*/)
    await expect(page.getByTestId('timer-status')).toContainText('停止中')
    
    // Step 2: タイマーを開始
    await page.getByTestId('task-item').getByRole('button', { name: 'タイマー開始' }).click()
    
    // Step 3: 視覚的な変化を確認
    await expect(page.getByTestId('task-item')).toHaveClass(/.*task-running.*/)
    await expect(page.getByTestId('timer-status')).toContainText('実行中')
    await expect(page.getByTestId('timer-display')).toBeVisible()
    await expect(page.getByTestId('timer-display')).toContainText(/\d{2}:\d{2}:\d{2}/)
    
    // タイマーボタンが「停止」に変わっていることを確認
    await expect(page.getByTestId('task-item').getByRole('button', { name: 'タイマー停止' })).toBeVisible()
    await expect(page.getByTestId('task-item').getByRole('button', { name: 'タイマー開始' })).not.toBeVisible()
    
    // Step 4: グローバルタイマー表示の確認
    await expect(page.getByTestId('global-timer')).toBeVisible()
    await expect(page.getByTestId('global-timer')).toContainText('開発作業')
    await expect(page.getByTestId('global-timer')).toContainText(/\d{2}:\d{2}:\d{2}/)
    
    // Step 5: タイマーを停止
    await page.getByTestId('task-item').getByRole('button', { name: 'タイマー停止' }).click()
    
    // Step 6: 停止後の視覚的状態を確認
    await expect(page.getByTestId('task-item')).not.toHaveClass(/.*task-running.*/)
    await expect(page.getByTestId('timer-status')).toContainText('停止中')
    await expect(page.getByTestId('global-timer')).not.toBeVisible()
    
    // タイマーボタンが「開始」に戻っていることを確認
    await expect(page.getByTestId('task-item').getByRole('button', { name: 'タイマー開始' })).toBeVisible()
    await expect(page.getByTestId('task-item').getByRole('button', { name: 'タイマー停止' })).not.toBeVisible()
  })

  test('複数タスクでの排他制御が機能する', async ({ page }) => {
    // 追加テストケース: 複数タスクの排他制御
    
    // Step 1: 2つ目のタスクを作成
    await page.getByRole('button', { name: '新しいタスク' }).click()
    await page.getByPlaceholder('タスク名').fill('設計作業')
    await page.getByRole('button', { name: '作成' }).click()
    
    // Step 2: 最初のタスクでタイマーを開始
    const firstTask = page.getByTestId('task-item').filter({ hasText: '開発作業' })
    const secondTask = page.getByTestId('task-item').filter({ hasText: '設計作業' })
    
    await firstTask.getByRole('button', { name: 'タイマー開始' }).click()
    await expect(firstTask).toHaveClass(/.*task-running.*/)
    
    // Step 3: 2つ目のタスクでタイマーを開始（1つ目は自動停止されるはず）
    await secondTask.getByRole('button', { name: 'タイマー開始' }).click()
    
    // Step 4: 状態を確認
    await expect(firstTask).not.toHaveClass(/.*task-running.*/)
    await expect(secondTask).toHaveClass(/.*task-running.*/)
    await expect(page.getByTestId('global-timer')).toContainText('設計作業')
    
    // Step 5: 最終停止
    await secondTask.getByRole('button', { name: 'タイマー停止' }).click()
    await expect(secondTask).not.toHaveClass(/.*task-running.*/)
    await expect(page.getByTestId('global-timer')).not.toBeVisible()
  })

  test('タイマー実行中のタスク編集・削除が制限される', async ({ page }) => {
    // エラーハンドリングテスト
    
    // Step 1: タイマーを開始
    await page.getByTestId('task-item').getByRole('button', { name: 'タイマー開始' }).click()
    await expect(page.getByTestId('task-item')).toHaveClass(/.*task-running.*/)
    
    // Step 2: 実行中タスクの編集ボタンが無効化されていることを確認
    await expect(page.getByTestId('task-item').getByRole('button', { name: '編集' })).toBeDisabled()
    await expect(page.getByTestId('task-item').getByRole('button', { name: 'アーカイブ' })).toBeDisabled()
    
    // Step 3: タイマーを停止
    await page.getByTestId('task-item').getByRole('button', { name: 'タイマー停止' }).click()
    
    // Step 4: 停止後は編集可能になることを確認
    await expect(page.getByTestId('task-item').getByRole('button', { name: '編集' })).toBeEnabled()
    await expect(page.getByTestId('task-item').getByRole('button', { name: 'アーカイブ' })).toBeEnabled()
  })

  test('ページリロード後もタイマー状態が保持される', async ({ page }) => {
    // 永続化テスト
    
    // Step 1: タイマーを開始
    await page.getByTestId('task-item').getByRole('button', { name: 'タイマー開始' }).click()
    await expect(page.getByTestId('task-item')).toHaveClass(/.*task-running.*/)
    
    // Step 2: ページをリロード
    await page.reload()
    await page.waitForLoadState('networkidle')
    
    // プロジェクトを再選択
    await page.getByTestId('project-card').click()
    
    // Step 3: タイマー状態が保持されていることを確認
    await expect(page.getByTestId('task-item')).toHaveClass(/.*task-running.*/)
    await expect(page.getByTestId('timer-status')).toContainText('実行中')
    await expect(page.getByTestId('global-timer')).toBeVisible()
    await expect(page.getByTestId('global-timer')).toContainText('開発作業')
    
    // Step 4: 正常に停止できることを確認
    await page.getByTestId('task-item').getByRole('button', { name: 'タイマー停止' }).click()
    await expect(page.getByTestId('task-item')).not.toHaveClass(/.*task-running.*/)
  })
})
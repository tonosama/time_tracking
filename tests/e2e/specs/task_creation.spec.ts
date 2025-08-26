import { test, expect } from '@playwright/test';

test.describe('タスク作成機能のE2Eテスト', () => {
  test.beforeEach(async ({ page }) => {
    // ページにアクセス
    await page.goto('/');
    
    // アプリケーションの読み込みを待つ
    await page.waitForSelector('[data-testid="dashboard"]', { timeout: 30000 });
    
    // デバッグ用：テスト開始時に一時停止（開発時のみ）
    if (process.env.NODE_ENV === 'development') {
      await page.pause();
    }
  });

  test('基本的なタスク作成フロー', async ({ page }) => {
    // プロジェクトサイドバーが表示されることを確認
    await expect(page.locator('[data-testid="project-sidebar"]')).toBeVisible();
    
    // 最初のプロジェクトを選択
    const firstProject = page.locator('[data-testid="project-item"]').first();
    await firstProject.click();
    
    // プロジェクトが選択された状態になることを確認
    await expect(firstProject).toHaveClass(/selected/);
    
    // タスク作成ボタンをクリック
    await page.click('[data-testid="create-task-button"]');
    
    // タスク作成モーダルが表示されることを確認
    await expect(page.locator('[data-testid="create-task-modal"]')).toBeVisible();
    
    // タスク名を入力
    await page.fill('[data-testid="task-name-input"]', 'テストタスク');
    
    // 作成ボタンをクリック
    await page.click('[data-testid="create-task-submit"]');
    
    // モーダルが閉じることを確認
    await expect(page.locator('[data-testid="create-task-modal"]')).not.toBeVisible();
    
    // 作成されたタスクが表示されることを確認
    await expect(page.locator('text=テストタスク')).toBeVisible();
  });

  test('タスク名のバリデーション', async ({ page }) => {
    // プロジェクトを選択
    await page.locator('[data-testid="project-item"]').first().click();
    
    // タスク作成モーダルを開く
    await page.click('[data-testid="create-task-button"]');
    
    // 空のタスク名で作成を試す
    await page.click('[data-testid="create-task-submit"]');
    
    // エラーメッセージが表示されることを確認
    await expect(page.locator('text=タスク名を入力してください')).toBeVisible();
    
    // 空白のみのタスク名で作成を試す
    await page.fill('[data-testid="task-name-input"]', '   ');
    await page.click('[data-testid="create-task-submit"]');
    
    // エラーメッセージが表示されることを確認
    await expect(page.locator('text=タスク名を入力してください')).toBeVisible();
  });

  test('モーダルの操作（キャンセル）', async ({ page }) => {
    // プロジェクトを選択
    await page.locator('[data-testid="project-item"]').first().click();
    
    // タスク作成モーダルを開く
    await page.click('[data-testid="create-task-button"]');
    
    // キャンセルボタンをクリック
    await page.click('[data-testid="create-task-cancel"]');
    
    // モーダルが閉じることを確認
    await expect(page.locator('[data-testid="create-task-modal"]')).not.toBeVisible();
  });

  test('モーダルの操作（ESCキー）', async ({ page }) => {
    // プロジェクトを選択
    await page.locator('[data-testid="project-item"]').first().click();
    
    // タスク作成モーダルを開く
    await page.click('[data-testid="create-task-button"]');
    
    // ESCキーを押す
    await page.keyboard.press('Escape');
    
    // モーダルが閉じることを確認
    await expect(page.locator('[data-testid="create-task-modal"]')).not.toBeVisible();
  });

  test('モーダルの操作（Enterキー）', async ({ page }) => {
    // プロジェクトを選択
    await page.locator('[data-testid="project-item"]').first().click();
    
    // タスク作成モーダルを開く
    await page.click('[data-testid="create-task-button"]');
    
    // タスク名を入力
    await page.fill('[data-testid="task-name-input"]', 'Enterキーで作成');
    
    // Enterキーを押す
    await page.keyboard.press('Enter');
    
    // モーダルが閉じることを確認
    await expect(page.locator('[data-testid="create-task-modal"]')).not.toBeVisible();
    
    // 作成されたタスクが表示されることを確認
    await expect(page.locator('text=Enterキーで作成')).toBeVisible();
  });

  test('フォーカス管理', async ({ page }) => {
    // プロジェクトを選択
    await page.locator('[data-testid="project-item"]').first().click();
    
    // タスク作成モーダルを開く
    await page.click('[data-testid="create-task-button"]');
    
    // タスク名入力フィールドにフォーカスが当たることを確認
    const taskNameInput = page.locator('[data-testid="task-name-input"]');
    await expect(taskNameInput).toBeFocused();
  });

  test('複数タスクの作成', async ({ page }) => {
    // プロジェクトを選択
    await page.locator('[data-testid="project-item"]').first().click();
    
    // 複数のタスクを作成
    const taskNames = ['タスク1', 'タスク2', 'タスク3'];
    
    for (const taskName of taskNames) {
      // タスク作成モーダルを開く
      await page.click('[data-testid="create-task-button"]');
      
      // タスク名を入力
      await page.fill('[data-testid="task-name-input"]', taskName);
      
      // 作成ボタンをクリック
      await page.click('[data-testid="create-task-submit"]');
      
      // モーダルが閉じることを確認
      await expect(page.locator('[data-testid="create-task-modal"]')).not.toBeVisible();
    }
    
    // すべてのタスクが表示されることを確認
    for (const taskName of taskNames) {
      await expect(page.locator(`text=${taskName}`)).toBeVisible();
    }
  });

  test('ローディング状態の確認', async ({ page }) => {
    // プロジェクトを選択
    await page.locator('[data-testid="project-item"]').first().click();
    
    // タスク作成モーダルを開く
    await page.click('[data-testid="create-task-button"]');
    
    // タスク名を入力
    await page.fill('[data-testid="task-name-input"]', 'ローディングテスト');
    
    // 作成ボタンをクリック
    await page.click('[data-testid="create-task-submit"]');
    
    // ローディング状態が表示されることを確認（短時間）
    await expect(page.locator('[data-testid="create-task-submit"]')).toBeDisabled();
  });

  test('データ永続化の確認', async ({ page }) => {
    // プロジェクトを選択
    await page.locator('[data-testid="project-item"]').first().click();
    
    // タスクを作成
    await page.click('[data-testid="create-task-button"]');
    await page.fill('[data-testid="task-name-input"]', '永続化テストタスク');
    await page.click('[data-testid="create-task-submit"]');
    
    // ページをリロード
    await page.reload();
    
    // アプリケーションの読み込みを待つ
    await page.waitForSelector('[data-testid="dashboard"]', { timeout: 30000 });
    
    // 同じプロジェクトを選択
    await page.locator('[data-testid="project-item"]').first().click();
    
    // 作成したタスクが表示されることを確認
    await expect(page.locator('text=永続化テストタスク')).toBeVisible();
  });

  test('エラーハンドリング（長いタスク名）', async ({ page }) => {
    // プロジェクトを選択
    await page.locator('[data-testid="project-item"]').first().click();
    
    // タスク作成モーダルを開く
    await page.click('[data-testid="create-task-button"]');
    
    // 非常に長いタスク名を入力
    const longTaskName = 'a'.repeat(1000);
    await page.fill('[data-testid="task-name-input"]', longTaskName);
    
    // 作成ボタンをクリック
    await page.click('[data-testid="create-task-submit"]');
    
    // エラーメッセージが表示されるか、モーダルが閉じることを確認
    await expect(page.locator('[data-testid="create-task-modal"]')).not.toBeVisible();
  });

  test('アクセシビリティ（キーボードナビゲーション）', async ({ page }) => {
    // プロジェクトを選択
    await page.locator('[data-testid="project-item"]').first().click();
    
    // Tabキーでタスク作成ボタンにフォーカスを移動
    await page.keyboard.press('Tab');
    
    // タスク作成ボタンにフォーカスが当たることを確認
    await expect(page.locator('[data-testid="create-task-button"]')).toBeFocused();
    
    // Enterキーでモーダルを開く
    await page.keyboard.press('Enter');
    
    // モーダルが開くことを確認
    await expect(page.locator('[data-testid="create-task-modal"]')).toBeVisible();
  });
});

// E2Eテスト - タイムトラッキング機能のテスト

import { test, expect } from '@playwright/test';

test.describe('Time Tracking', () => {
  test('should create project and start time tracking', async ({ page }) => {
    await page.goto('/');
    
    // プロジェクト作成
    await page.click('[data-testid="create-project-button"]');
    await page.fill('[data-testid="project-name-input"]', 'Test Project');
    await page.click('[data-testid="save-project-button"]');
    
    // タスク作成
    await page.click('[data-testid="create-task-button"]');
    await page.fill('[data-testid="task-name-input"]', 'Test Task');
    await page.click('[data-testid="save-task-button"]');
    
    // タイマー開始
    await page.click('[data-testid="start-timer-button"]');
    
    // タイマーが実行中であることを確認
    await expect(page.locator('[data-testid="timer-status"]')).toContainText('実行中');
    
    // タイマー停止
    await page.click('[data-testid="stop-timer-button"]');
    
    // 時間エントリが記録されていることを確認
    await expect(page.locator('[data-testid="time-entry-list"]')).toBeVisible();
  });
});


// E2Eテスト - プロジェクト管理機能のテスト

import { test, expect } from '@playwright/test';

test.describe('Project Management', () => {
  test('プロジェクト一覧が表示されること', async ({ page }) => {
    await page.goto('/');
    
    // プロジェクト一覧のヘディングが表示されることを確認
    await expect(page.locator('h2')).toContainText('プロジェクト一覧');
    
    // 新しいプロジェクトボタンが表示されることを確認
    await expect(page.getByRole('button', { name: '新しいプロジェクト' })).toBeVisible();
    
    // アーカイブ済みを表示ボタンが表示されることを確認
    await expect(page.getByRole('button', { name: 'アーカイブ済みを表示' })).toBeVisible();
  });

  test('アーカイブ表示の切り替えができること', async ({ page }) => {
    await page.goto('/');
    
    // 初期状態でアーカイブ済みを表示ボタンがあることを確認
    const toggleButton = page.getByRole('button', { name: 'アーカイブ済みを表示' });
    await expect(toggleButton).toBeVisible();
    
    // ボタンをクリック
    await toggleButton.click();
    
    // ボタンのテキストが変更されることを確認
    await expect(page.getByRole('button', { name: 'アクティブのみ表示' })).toBeVisible();
  });

  // 注：このテストはバックエンドのモックが必要なため、現在はスキップ
  test.skip('プロジェクトカードが表示されること', async ({ page }) => {
    await page.goto('/');
    
    // プロジェクトカードが表示されることを確認
    await expect(page.locator('[data-testid="project-card"]')).toBeVisible();
  });
});


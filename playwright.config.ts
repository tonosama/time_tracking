import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright設定ファイル
 * E2Eテストの実行環境を定義
 */
export default defineConfig({
  // テストディレクトリ
  testDir: './tests/e2e',
  
  // 並行実行の設定
  fullyParallel: true,
  
  // CI環境でのfail fast
  forbidOnly: !!process.env.CI,
  
  // リトライ設定
  retries: process.env.CI ? 2 : 0,
  
  // ワーカー数（CI環境では1、ローカルでは未定義=自動）
  workers: process.env.CI ? 1 : undefined,
  
  // レポーター設定
  reporter: 'html',
  
  // 全テストの共通設定
  use: {
    // ベースURL
    baseURL: 'http://127.0.0.1:1420',
    
    // トレース設定（失敗時のみ）
    trace: 'on-first-retry',
  },

  // プロジェクト設定（ブラウザ別）
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },

    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },

    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },

    // モバイルブラウザ（必要に応じて有効化）
    // {
    //   name: 'Mobile Chrome',
    //   use: { ...devices['Pixel 5'] },
    // },
    // {
    //   name: 'Mobile Safari',
    //   use: { ...devices['iPhone 12'] },
    // },
  ],

  // ローカル開発サーバーの設定
  webServer: {
    command: 'npm run tauri:dev',
    url: 'http://127.0.0.1:1420',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000, // 2分のタイムアウト
  },
});

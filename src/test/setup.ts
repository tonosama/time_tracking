import '@testing-library/jest-dom'
import { afterEach, vi } from 'vitest'
import { cleanup } from '@testing-library/react'

// 各テスト後にクリーンアップ
afterEach(() => {
  cleanup()
})

// Tauriのモック
const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}))

// window.__TAURI__のモック
Object.defineProperty(window, '__TAURI__', {
  writable: true,
  value: {
    invoke: mockInvoke,
  },
})

// グローバル設定
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})

// Loggerのモック
vi.mock('@/utils', () => ({
  Logger: {
    debug: vi.fn(),
    error: vi.fn(),
    warn: vi.fn(),
    info: vi.fn(),
    userAction: vi.fn(),
    apiCall: vi.fn(),
    apiSuccess: vi.fn(),
    apiError: vi.fn(),
    performance: vi.fn(),
  },
  groupEntriesByDate: vi.fn(() => ({})),
  formatDuration: vi.fn(() => '00:00:00'),
}));

// グローバルでモックを利用可能にする
(global as any).mockInvoke = mockInvoke  // 型アサーションを追加

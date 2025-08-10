import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  
  // Tauri用の設定
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },

  // テスト設定
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    // E2Eテストのみを除外（Playwrightで実行）
    exclude: [
      '**/tests/e2e/**'
    ],
  },

  // パスエイリアス設定
  resolve: {
    alias: {
      '@': '/src'
    }
  },
})

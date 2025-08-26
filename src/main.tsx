import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'

console.log('[main.tsx] Application starting...')

// Tauriの初期化を待ってからアプリケーションをレンダリング
async function startApp() {
  console.log('[main.tsx] startApp called')
  
  // Tauriの初期化を待つ
  if (window.__TAURI__) {
    console.log('[main.tsx] Tauri detected, waiting for init...')
    await window.__TAURI__.init?.()  // オプショナルチェーンを追加
    console.log('[main.tsx] Tauri init completed')
  } else {
    console.log('[main.tsx] Tauri not detected')
  }

  console.log('[main.tsx] Rendering React app...')
  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  )
  console.log('[main.tsx] React app rendered')
}

startApp()

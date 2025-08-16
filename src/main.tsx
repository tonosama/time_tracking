import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'

// Tauriの初期化を待ってからアプリケーションをレンダリング
async function startApp() {
  // Tauriの初期化を待つ
  if (window.__TAURI__) {
    await window.__TAURI__.init()
  }

  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  )
}

startApp()

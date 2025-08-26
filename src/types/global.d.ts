// Tauri APIの型定義
declare global {
  interface Window {
    __TAURI__?: {
      invoke: (command: string, args?: any) => Promise<any>
      init?: () => Promise<void>
    }
  }
}

export {}

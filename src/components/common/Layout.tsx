import { ReactNode } from 'react'
import { GlobalTimer } from '@/components/time_tracking'

interface LayoutProps {
  children: ReactNode
  title?: string
  onTimerStopped?: () => void
}

export function Layout({ children, title, onTimerStopped }: LayoutProps) {
  console.log('[Layout] Component rendering')
  console.log('[Layout] Props:', { title, onTimerStopped: !!onTimerStopped })
  
  return (
    <div className="layout">
      <header className="layout-header">
        <div className="header-content">
          <h1 className="layout-title">{title || 'Time Tracker'}</h1>
          <GlobalTimer onTimerStopped={onTimerStopped} />
        </div>
      </header>
      <main className="layout-main">
        {children}
      </main>
    </div>
  )
}

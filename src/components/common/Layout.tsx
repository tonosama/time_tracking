import { ReactNode } from 'react'
import { GlobalTimer } from '@/components/time_tracking'

interface LayoutProps {
  children: ReactNode
  title?: string
}

export function Layout({ children, title }: LayoutProps) {
  return (
    <div className="layout">
      <header className="layout-header">
        <div className="header-content">
          <h1 className="layout-title">{title || 'Time Tracker'}</h1>
          <GlobalTimer />
        </div>
      </header>
      <main className="layout-main">
        {children}
      </main>
    </div>
  )
}

import React from 'react'
import { ProjectList } from '@/components/projects'
import './styles/components.css'

function App() {
  return (
    <div className="app">
      <header className="app-header">
        <h1>Time Tracker Go</h1>
      </header>
      <main className="app-main">
        <ProjectList />
      </main>
    </div>
  )
}

export default App

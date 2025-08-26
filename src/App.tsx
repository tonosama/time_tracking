import { useState } from 'react'
import { Dashboard } from '@/components/dashboard'
import { Layout } from '@/components/common'
import './styles/components.css'

function App() {
  console.log('[App] Component rendering')
  const [refreshKey, setRefreshKey] = useState(0)

  const handleTimerStopped = () => {
    console.log('[App] handleTimerStopped called')
    console.log('[App] Current refreshKey:', refreshKey)
    setRefreshKey(prev => {
      const newKey = prev + 1
      console.log('[App] Setting refreshKey from', prev, 'to', newKey)
      return newKey
    })
    console.log('[App] handleTimerStopped completed')
  }

  console.log('[App] Rendering with refreshKey:', refreshKey)

  return (
    <Layout title="TimeTracker" onTimerStopped={handleTimerStopped}>
      <Dashboard key={refreshKey} />
    </Layout>
  )
}

export default App

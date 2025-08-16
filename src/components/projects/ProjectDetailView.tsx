import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { Project, Task } from '@/types'
import { Button } from '../common/Button'
import { CreateTaskModal } from '../tasks/CreateTaskModal'
import { TaskList } from '../tasks/TaskList'

interface ProjectDetailViewProps {
  project: Project
  onBack: () => void
  onProjectUpdate?: (project: Project) => void
}

export function ProjectDetailView({ project, onBack }: ProjectDetailViewProps) {
  const [tasks, setTasks] = useState<Task[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [showCreateTaskModal, setShowCreateTaskModal] = useState(false)

  const loadTasks = async () => {
    try {
      setLoading(true)
      setError(null)
      
      const result = await invoke<Task[]>('get_tasks_by_project', {
        projectId: project.id
      })
      setTasks(result)
    } catch (err) {
      setError('タスクの読み込みに失敗しました')
      console.error('タスク読み込みエラー:', err)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadTasks()
  }, [project.id])

  const handleTaskCreated = (newTask: Task) => {
    setTasks(prevTasks => [newTask, ...prevTasks])
    setShowCreateTaskModal(false)
  }

  const handleTaskUpdate = (updatedTask: Task) => {
    setTasks(prevTasks => 
      prevTasks.map(task => 
        task.id === updatedTask.id ? updatedTask : task
      )
    )
  }

  const handleTaskArchived = (taskId: number) => {
    setTasks(prevTasks => 
      prevTasks.filter(task => task.id !== taskId)
    )
  }

  const isProjectArchived = project.status === 'archived'

  return (
    <div className="project-detail-view">
      <div className="project-detail-header">
        <div className="breadcrumb">
          <Button
            variant="secondary"
            size="sm"
            onClick={onBack}
          >
            ← プロジェクト一覧に戻る
          </Button>
        </div>
        
        <div className="project-info">
          <h1 className="project-title">{project.name}</h1>
          <span className={`project-status ${project.status}`}>
            {project.status === 'active' ? 'アクティブ' : 'アーカイブ済み'}
          </span>
        </div>

        <div className="project-actions">
          {!isProjectArchived && (
            <Button
              variant="primary"
              onClick={() => setShowCreateTaskModal(true)}
            >
              新しいタスク
            </Button>
          )}
        </div>
      </div>

      {isProjectArchived && (
        <div className="archived-notice">
          <p>このプロジェクトはアーカイブ済みです。新しいタスクを作成することはできません。</p>
        </div>
      )}

      <div className="project-detail-content">
        <div className="tasks-section">
          <h2>タスク一覧</h2>
          
          {loading ? (
            <div className="loading-state">
              <p>タスクを読み込み中...</p>
            </div>
          ) : error ? (
            <div className="error-state">
              <p className="error">{error}</p>
              <Button variant="secondary" onClick={loadTasks}>
                再読み込み
              </Button>
            </div>
          ) : (
            <TaskList
              tasks={tasks}
              onTaskUpdate={handleTaskUpdate}
              onTaskArchived={handleTaskArchived}
              readonly={isProjectArchived}
            />
          )}
        </div>
      </div>

      <CreateTaskModal
        isOpen={showCreateTaskModal}
        onClose={() => setShowCreateTaskModal(false)}
        onTaskCreated={handleTaskCreated}
        projectId={project.id}
      />
    </div>
  )
}

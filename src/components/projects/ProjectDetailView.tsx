import { useState } from 'react'
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
  console.log('[ProjectDetailView] Rendering with project:', project)
  console.log('[ProjectDetailView] Project details:', { id: project.id, name: project.name, status: project.status })
  const [showCreateTaskModal, setShowCreateTaskModal] = useState(false)

  const handleTaskCreated = (newTask: Task) => {
    console.log('[ProjectDetailView] Task created:', newTask)
    setShowCreateTaskModal(false)
  }

  const handleTaskUpdate = (_updatedTask: Task) => {
    // TaskListコンポーネントが独自に管理するため、何もしない
  }

  const handleTaskArchived = (_task: Task) => {
    // TaskListコンポーネントが独自に管理するため、何もしない
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
              data-testid="create-task-button"
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
          
          <TaskList
            selectedProjectId={project.id}
            projects={[project]}
            onTaskEdit={handleTaskUpdate}
            onTaskArchive={handleTaskArchived}
            onTaskRestore={handleTaskUpdate}
            onTaskStartTimer={() => {}}
          />
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

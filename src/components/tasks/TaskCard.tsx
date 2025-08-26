import { Button } from '../common/Button'
import { formatDate } from '@/utils/dateUtils'
import type { Task } from '@/types'

interface TaskCardProps {
  task: Task
  onEdit: (task: Task) => void
  onArchive: (task: Task) => void
  onRestore: (task: Task) => void
  onStartTimer: (task: Task) => void
}

export function TaskCard({ task, onEdit, onArchive, onRestore, onStartTimer }: TaskCardProps) {
  console.log('[TaskCard] Rendering task:', task)
  console.log('[TaskCard] Task details:', {
    id: task.id,
    name: task.name,
    project_id: task.project_id,
    status: task.status,
    nameIsUntitled: task.name === 'Untitled task',
    nameLength: task.name?.length || 0
  })
  
  const isActive = task.status === 'active'
  const isArchived = task.status === 'archived'

  const handleEdit = (e: React.MouseEvent) => {
    e.preventDefault()
    e.stopPropagation()
    onEdit(task)
  }

  const handleArchive = (e: React.MouseEvent) => {
    e.preventDefault()
    e.stopPropagation()
    onArchive(task)
  }

  const handleRestore = (e: React.MouseEvent) => {
    e.preventDefault()
    e.stopPropagation()
    onRestore(task)
  }

  const handleStartTimer = (e: React.MouseEvent) => {
    e.preventDefault()
    e.stopPropagation()
    onStartTimer(task)
  }

  return (
    <div 
      className={`task-card ${isArchived ? 'task-card--archived' : ''}`}
      data-testid={`task-card-${task.id}`}
    >
      <div className="task-card__header">
        <h3 className="task-card__title" title={task.name}>
          {task.name}
        </h3>
        <span className={`task-card__status task-card__status--${task.status}`}>
          {isActive ? 'アクティブ' : 'アーカイブ済み'}
        </span>
      </div>

      <div className="task-card__content">
        <p className="task-card__date">
          作成日: {formatDate(task.effective_at)}
        </p>
      </div>

      <div className="task-card__actions">
        {isActive ? (
          <>
            <Button
              variant="secondary"
              size="sm"
              onClick={handleEdit}
              data-testid={`task-edit-${task.id}`}
            >
              編集
            </Button>
            <Button
              variant="secondary"
              size="sm"
              onClick={handleArchive}
              data-testid={`task-archive-${task.id}`}
            >
              アーカイブ
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={handleStartTimer}
              data-testid={`task-start-timer-${task.id}`}
            >
              タイマー開始
            </Button>
          </>
        ) : (
          <Button
            variant="secondary"
            size="sm"
            onClick={handleRestore}
            data-testid={`task-restore-${task.id}`}
          >
            復元
          </Button>
        )}
      </div>
    </div>
  )
}

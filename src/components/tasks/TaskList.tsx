import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { Task } from '@/types'
import { Button } from '../common/Button'
import { EditTaskModal } from './EditTaskModal'
import { TimerButton, TimeHistoryModal } from '@/components/time_tracking'

interface TaskListProps {
  tasks: Task[]
  onTaskUpdate?: (task: Task) => void
  onTaskArchived?: (taskId: number) => void
  readonly?: boolean
}

export function TaskList({ 
  tasks, 
  onTaskUpdate, 
  onTaskArchived,
  readonly = false 
}: TaskListProps) {
  const [editingTask, setEditingTask] = useState<Task | null>(null)
  const [historyTask, setHistoryTask] = useState<Task | null>(null)

  const handleArchiveTask = async (taskId: number) => {
    try {
      await invoke('archive_task', {
        request: { id: taskId }
      })
      onTaskArchived?.(taskId)
    } catch (err) {
      console.error('タスクアーカイブエラー:', err)
    }
  }

  const handleRestoreTask = async (task: Task) => {
    try {
      const restoredTask = await invoke<Task>('restore_task', {
        request: { id: task.id }
      })
      onTaskUpdate?.(restoredTask)
    } catch (err) {
      console.error('タスク復元エラー:', err)
    }
  }

  const handleTaskUpdated = (updatedTask: Task) => {
    setEditingTask(null)
    onTaskUpdate?.(updatedTask)
  }

  if (tasks.length === 0) {
    return (
      <div className="empty-state">
        {readonly ? (
          <p>タスクがありません</p>
        ) : (
          <p>まだタスクがありません。新しいタスクを作成してみましょう。</p>
        )}
      </div>
    )
  }

  return (
    <div className="task-list">
      <div className="task-grid">
        {tasks.map(task => (
          <TaskItem
            key={task.id}
            task={task}
            onEdit={() => setEditingTask(task)}
            onArchive={() => handleArchiveTask(task.id)}
            onRestore={() => handleRestoreTask(task)}
            onViewHistory={() => setHistoryTask(task)}
            readonly={readonly}
          />
        ))}
      </div>

      {editingTask && (
        <EditTaskModal
          isOpen={true}
          onClose={() => setEditingTask(null)}
          onTaskUpdated={handleTaskUpdated}
          task={editingTask}
        />
      )}

      {historyTask && (
        <TimeHistoryModal
          isOpen={true}
          onClose={() => setHistoryTask(null)}
          task={historyTask}
        />
      )}
    </div>
  )
}

interface TaskItemProps {
  task: Task
  onEdit: () => void
  onArchive: () => void
  onRestore: () => void
  onViewHistory: () => void
  readonly: boolean
}

function TaskItem({ task, onEdit, onArchive, onRestore, onViewHistory, readonly }: TaskItemProps) {
  return (
    <div className="task-item" data-testid="task-item">
      <div className="task-header">
        <h3 className="task-name">{task.name}</h3>
        <span 
          className={`task-status ${task.status}`}
          data-testid="task-status"
        >
          {task.status === 'active' ? 'アクティブ' : 'アーカイブ済み'}
        </span>
      </div>
      
      <div className="task-meta">
        <time dateTime={task.effective_at}>
          更新: {new Date(task.effective_at).toLocaleDateString('ja-JP')}
        </time>
      </div>

      {/* タイマーセクション - アクティブなタスクのみ */}
      {task.status === 'active' && (
        <div className="task-timer-section">
          <TimerButton 
            task={task}
            disabled={readonly}
          />
        </div>
      )}

      <div className="task-actions">
        {/* タイム履歴ボタン */}
        <Button
          variant="secondary"
          size="sm"
          onClick={onViewHistory}
          aria-label={`${task.name}のタイム履歴`}
        >
          タイム履歴
        </Button>

        {!readonly && (
          <>
            {task.status === 'active' ? (
              <>
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={onEdit}
                  aria-label={`${task.name}を編集`}
                >
                  編集
                </Button>
                <Button
                  variant="warning"
                  size="sm"
                  onClick={onArchive}
                  aria-label={`${task.name}をアーカイブ`}
                >
                  アーカイブ
                </Button>
              </>
            ) : (
              <Button
                variant="success"
                size="sm"
                onClick={onRestore}
                aria-label={`${task.name}を復元`}
              >
                復元
              </Button>
            )}
          </>
        )}
      </div>
    </div>
  )
}

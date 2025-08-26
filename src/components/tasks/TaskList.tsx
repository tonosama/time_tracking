import { useEffect, useState } from 'react'
import { useTasks } from '@/hooks/useTasks'
import { TaskCard } from './TaskCard'
import { Button } from '../common/Button'
import { Loading } from '../common/Loading'
import { formatDate } from '@/utils/dateUtils'
import type { Task, Project } from '@/types'

interface TaskListProps {
  selectedProjectId: number | null
  projects: Project[]
  showArchived?: boolean
  onTaskEdit?: (task: Task) => void
  onTaskArchive?: (task: Task) => void
  onTaskRestore?: (task: Task) => void
  onTaskStartTimer?: (task: Task) => void
}

export function TaskList({ 
  selectedProjectId, 
  projects, 
  showArchived = false,
  onTaskEdit,
  onTaskArchive,
  onTaskRestore,
  onTaskStartTimer
}: TaskListProps) {
  console.log('[TaskList] Component rendering with props:', { 
    selectedProjectId, 
    projectsCount: projects?.length || 0,
    showArchived 
  })
  const { 
    tasks, 
    loading, 
    error, 
    selectedDate,
    loadTasks, 
    goToPreviousDay,
    goToNextDay,
    clearError 
  } = useTasks()
  const [filteredTasks, setFilteredTasks] = useState<Task[]>([])

  // プロジェクトが変更された時にタスクを再読み込み
  useEffect(() => {
    console.log('[TaskList] useEffect triggered:', { selectedProjectId, selectedDate })
    if (selectedProjectId) {
      console.log('[TaskList] Loading tasks for project:', selectedProjectId)
      loadTasks(selectedProjectId, selectedDate)
    } else {
      console.log('[TaskList] No project selected, clearing tasks')
      // プロジェクトが選択されていない場合はタスクをクリア
      setFilteredTasks([])
    }
  }, [selectedProjectId, selectedDate, loadTasks])

  // タスクのフィルタリング
  useEffect(() => {
    console.log('[TaskList] Tasks filtering effect:', { tasks: tasks?.length, showArchived })
    if (!tasks) {
      console.log('[TaskList] No tasks, setting empty filtered tasks')
      setFilteredTasks([])
      return
    }

    const filtered = showArchived 
      ? tasks // アーカイブ済みも含めてすべて表示
      : tasks.filter(task => task.status === 'active') // アクティブのみ表示

    console.log('[TaskList] Filtered tasks:', { total: tasks.length, filtered: filtered.length })
    setFilteredTasks(filtered)
  }, [tasks, showArchived])

  const handleRetry = () => {
    clearError()
    if (selectedProjectId) {
      loadTasks(selectedProjectId, selectedDate)
    }
  }

  const handlePreviousDay = () => {
    goToPreviousDay()
  }

  const handleNextDay = () => {
    goToNextDay()
  }

  const isToday = () => {
    const today = new Date()
    return selectedDate.toDateString() === today.toDateString()
  }

  const isFutureDate = () => {
    const today = new Date()
    today.setHours(23, 59, 59, 999)
    return selectedDate > today
  }

  const handleTaskEdit = (task: Task) => {
    if (onTaskEdit) {
      onTaskEdit(task)
    }
  }

  const handleTaskArchive = (task: Task) => {
    if (onTaskArchive) {
      onTaskArchive(task)
    }
  }

  const handleTaskRestore = (task: Task) => {
    if (onTaskRestore) {
      onTaskRestore(task)
    }
  }

  const handleTaskStartTimer = (task: Task) => {
    if (onTaskStartTimer) {
      onTaskStartTimer(task)
    }
  }

  // プロジェクトが選択されていない場合
  if (!selectedProjectId) {
    console.log('[TaskList] No project selected, showing empty state')
    return (
      <div className="task-list task-list--empty" data-testid="task-list-no-project">
        <div className="task-list__empty-state">
          <h3>プロジェクトを選択してください</h3>
          <p>タスクを表示するには、プロジェクトを選択してください。</p>
        </div>
      </div>
    )
  }

  // ローディング状態
  if (loading) {
    console.log('[TaskList] Loading state, showing loading component')
    return (
      <div className="task-list" data-testid="task-list-loading">
        <Loading message="タスクを読み込み中..." />
      </div>
    )
  }

  // エラー状態
  if (error) {
    console.log('[TaskList] Error state:', error)
    return (
      <div className="task-list task-list--error" data-testid="task-list-error">
        <div className="task-list__error-state">
          <h3>エラーが発生しました</h3>
          <p>{error}</p>
          <Button onClick={handleRetry} variant="primary">
            再試行
          </Button>
        </div>
      </div>
    )
  }

  // タスクが存在しない場合
  if (filteredTasks.length === 0) {
    console.log('[TaskList] No tasks to display, showing empty state')
    return (
      <div className="task-list task-list--empty" data-testid="task-list-empty">
        <div className="task-list__empty-state">
          <h3>タスクがありません</h3>
          <p>新しいタスクを作成してください。</p>
        </div>
      </div>
    )
  }

  // タスク一覧表示
  console.log('[TaskList] Rendering task list with', filteredTasks.length, 'tasks')
  return (
    <div className="task-list" data-testid="task-list" style={{ border: '2px solid red', padding: '20px' }}>
      <h2>タスク一覧 (デバッグ表示)</h2>
      <p>selectedProjectId: {selectedProjectId}</p>
      <p>loading: {loading.toString()}</p>
      <p>error: {error || 'なし'}</p>
      <p>tasks count: {tasks?.length || 0}</p>
      <p>filteredTasks count: {filteredTasks.length}</p>
      
      <div className="task-list__header">
        <div className="task-list__header-left">
          <h2>タスク一覧</h2>
          <div className="task-list__date-navigation">
            <Button
              variant="secondary"
              size="sm"
              onClick={handlePreviousDay}
              disabled={false}
              data-testid="previous-day-btn"
            >
              &lt;
            </Button>
            <span className="task-list__date">
              {isToday() ? 'Today' : formatDate(selectedDate.toISOString())} - {filteredTasks.length}件のタスク
            </span>
            <Button
              variant="secondary"
              size="sm"
              onClick={handleNextDay}
              disabled={isFutureDate()}
              data-testid="next-day-btn"
            >
              &gt;
            </Button>
          </div>
        </div>
        <span className="task-list__count">
          {filteredTasks.length}件のタスク
        </span>
      </div>
      
      <div className="task-list__content">
        {filteredTasks.map(task => (
          <TaskCard
            key={task.id}
            task={task}
            onEdit={handleTaskEdit}
            onArchive={handleTaskArchive}
            onRestore={handleTaskRestore}
            onStartTimer={handleTaskStartTimer}
          />
        ))}
      </div>
    </div>
  )
}

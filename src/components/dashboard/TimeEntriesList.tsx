import { useState, useRef, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Project, Task, TimeEntry } from '@/types'
import { formatDuration, formatTime, groupEntriesByDate } from '@/utils'
import { ConfirmDialog } from '../common/ConfirmDialog'

interface TimeEntriesListProps {
  entries: TimeEntry[]
  projects: Project[]
  tasks: Task[]
  onRefresh?: () => void
}

interface TimeEntryItemProps {
  entry: TimeEntry
  task: Task | undefined
  project: Project | undefined
  onRefresh?: () => void
}

function TimeEntryItem({ entry, task, project, onRefresh }: TimeEntryItemProps) {
  const [showActions, setShowActions] = useState(false)
  const [isProcessing, setIsProcessing] = useState(false)
  const [showEditDialog, setShowEditDialog] = useState(false)
  const [showDeleteDialog, setShowDeleteDialog] = useState(false)
  const [showDuplicateDialog, setShowDuplicateDialog] = useState(false)
  
  const actionsRef = useRef<HTMLDivElement>(null)
  
  const duration = entry.end_time 
    ? new Date(entry.end_time).getTime() - new Date(entry.start_time).getTime()
    : Date.now() - new Date(entry.start_time).getTime()
  
  const startTime = formatTime(new Date(entry.start_time))
  const endTime = entry.end_time ? formatTime(new Date(entry.end_time)) : 'Running'
  
  const getTaskDisplayName = () => {
    if (task?.name) {
      if (entry.description && entry.description !== task.name) {
        return `${task.name}: ${entry.description}`
      }
      return task.name
    }
    return entry.description || 'Untitled task'
  }

  // 外部クリックでメニューを閉じる
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        showActions &&
        actionsRef.current &&
        !actionsRef.current.contains(event.target as Node)
      ) {
        console.log('Closing actions menu - clicked outside')
        setShowActions(false)
      }
    }

    if (showActions) {
      setTimeout(() => {
        document.addEventListener('mousedown', handleClickOutside)
      }, 100)
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
    }
  }, [showActions])

  const handleActionsToggle = (e: React.MouseEvent) => {
    e.preventDefault()
    e.stopPropagation()
    console.log('=== Actions toggle clicked ===')
    console.log('Current showActions state:', showActions)
    console.log('Will set showActions to:', !showActions)
    setShowActions(!showActions)
  }

  const handleEdit = () => {
    console.log('🟢 === EDIT BUTTON CLICKED ===')
    console.log('Entry ID:', entry.id)
    console.log('Entry data:', entry)
    console.log('Task data:', task)
    setShowActions(false)
    
    // カスタムダイアログを表示
    console.log('🟢 Showing custom edit dialog')
    setShowEditDialog(true)
  }

  const handleDelete = () => {
    console.log('🔴 === DELETE BUTTON CLICKED ===')
    console.log('Entry ID:', entry.id)
    console.log('Entry data:', entry)
    console.log('Task data:', task)
    setShowActions(false)
    
    // カスタムダイアログを表示
    console.log('🔴 Showing custom delete dialog')
    setShowDeleteDialog(true)
  }

  const handleDuplicate = async () => {
    console.log('🟡 === DUPLICATE BUTTON CLICKED ===')
    console.log('Entry ID:', entry.id)
    console.log('Entry data:', entry)
    console.log('Task data:', task)
    setShowActions(false)
    
    if (!task?.id) {
      console.log('🟡 No task found for duplication')
      alert('タスクが見つからないため、複製できません。')
      return
    }

    // カスタムダイアログを表示
    console.log('🟡 Showing custom duplicate dialog')
    setShowDuplicateDialog(true)
  }
  
  return (
    <>
      <div className="time-entry-item">
      <div 
        className="entry-color-bar" 
        style={{ backgroundColor: project?.color || '#6c757d' }}
      />
      
      <div className="entry-content">
        <div className="entry-main">
          <h4 className="entry-task">{getTaskDisplayName()}</h4>
          <div className="entry-details">
            <span className="entry-project">📁 {project?.name || 'No Project'}</span>
            {entry.tags && entry.tags.length > 0 && (
              <span className="entry-tags">
                🏷️ {entry.tags.join(', ')}
              </span>
            )}
          </div>
        </div>
        
        <div className="entry-time-info">
          <div className="entry-duration">{formatDuration(duration)}</div>
          <div className="entry-period">{startTime} - {endTime}</div>
        </div>
        
        <div className="entry-actions" ref={actionsRef}>
          <button 
            className="actions-btn"
            onClick={handleActionsToggle}
            disabled={isProcessing}
            title="アクションメニュー"
            type="button"
          >
            {isProcessing ? '⏳' : '⋯'}
          </button>
          {showActions && (
            <div className="actions-menu">
              <button 
                type="button"
                onClick={(e) => {
                  console.log('🟢 === EDIT BUTTON RAW CLICK EVENT ===')
                  console.log('Event target:', e.target)
                  console.log('Event currentTarget:', e.currentTarget)
                  console.log('Event type:', e.type)
                  e.preventDefault()
                  e.stopPropagation()
                  console.log('🟢 About to call handleEdit()')
                  handleEdit()
                  console.log('🟢 handleEdit() called')
                }}
                onMouseDown={(_e) => {
                  console.log('🟢 Edit button mousedown event')
                }}
                onMouseUp={(_e) => {
                  console.log('🟢 Edit button mouseup event')
                }}
                style={{ cursor: 'pointer', userSelect: 'none' }}
              >
                ✏️ Edit
              </button>
              <button 
                type="button"
                onClick={(e) => {
                  console.log('🔴 === DELETE BUTTON RAW CLICK EVENT ===')
                  console.log('Event target:', e.target)
                  console.log('Event currentTarget:', e.currentTarget)
                  console.log('Event type:', e.type)
                  e.preventDefault()
                  e.stopPropagation()
                  console.log('🔴 About to call handleDelete()')
                  handleDelete()
                  console.log('🔴 handleDelete() called')
                }}
                onMouseDown={(_e) => {
                  console.log('🔴 Delete button mousedown event')
                }}
                onMouseUp={(_e) => {
                  console.log('🔴 Delete button mouseup event')
                }}
                style={{ cursor: 'pointer', userSelect: 'none' }}
              >
                🗑️ Delete
              </button>
              <button 
                type="button"
                onClick={(e) => {
                  console.log('🟡 === DUPLICATE BUTTON RAW CLICK EVENT ===')
                  console.log('Event target:', e.target)
                  console.log('Event currentTarget:', e.currentTarget)
                  console.log('Event type:', e.type)
                  e.preventDefault()
                  e.stopPropagation()
                  console.log('🟡 About to call handleDuplicate()')
                  handleDuplicate()
                  console.log('🟡 handleDuplicate() called')
                }}
                onMouseDown={(_e) => {
                  console.log('🟡 Duplicate button mousedown event')
                }}
                onMouseUp={(_e) => {
                  console.log('🟡 Duplicate button mouseup event')
                }}
                style={{ cursor: 'pointer', userSelect: 'none' }}
              >
                📋 Duplicate
              </button>
            </div>
          )}
        </div>
      </div>
    </div>

    {/* カスタム編集ダイアログ */}
    <ConfirmDialog
      isOpen={showEditDialog}
      title="編集機能について"
      message="タイムエントリの編集機能は今後実装予定です。"
      onConfirm={() => setShowEditDialog(false)}
      onCancel={() => setShowEditDialog(false)}
      confirmText="OK"
      cancelText="閉じる"
      variant="info"
    />

    {/* カスタム削除確認ダイアログ */}
    <ConfirmDialog
      isOpen={showDeleteDialog}
      title="タイムエントリの削除"
      message={
        <div>
          <p>この時間エントリを削除しますか？</p>
          <div style={{ marginTop: '12px', padding: '12px', backgroundColor: '#f8f9fa', borderRadius: '4px', fontSize: '14px' }}>
            <div><strong>タスク:</strong> {getTaskDisplayName()}</div>
            <div><strong>時間:</strong> {startTime} - {endTime}</div>
            <div><strong>継続時間:</strong> {formatDuration(duration)}</div>
          </div>
        </div>
      }
      onConfirm={() => {
        console.log('🔴 User confirmed deletion via custom dialog')
        setShowDeleteDialog(false)
        alert('削除機能は今後実装予定です。')
      }}
      onCancel={() => {
        console.log('🔴 User cancelled deletion via custom dialog')
        setShowDeleteDialog(false)
      }}
      confirmText="削除"
      cancelText="キャンセル"
      variant="danger"
    />

    {/* カスタム複製確認ダイアログ */}
    <ConfirmDialog
      isOpen={showDuplicateDialog}
      title="タイムエントリの複製"
      message={
        <div>
          <p>この時間エントリを複製しますか？</p>
          <div style={{ marginTop: '12px', padding: '12px', backgroundColor: '#f8f9fa', borderRadius: '4px', fontSize: '14px' }}>
            <div><strong>タスク:</strong> {getTaskDisplayName()}</div>
            <div><strong>継続時間:</strong> {formatDuration(duration)}</div>
          </div>
        </div>
      }
      onConfirm={async () => {
        console.log('🟡 User confirmed duplication via custom dialog')
        setShowDuplicateDialog(false)
        
        if (!task?.id) {
          alert('タスクが見つからないため、複製できません。')
          return
        }

        const startDate = new Date(entry.start_time)
        const endDate = entry.end_time ? new Date(entry.end_time) : new Date()
        const durationMs = endDate.getTime() - startDate.getTime()
        const newStartTime = new Date()
        const newEndTime = new Date(newStartTime.getTime() + durationMs)

        setIsProcessing(true)
        try {
          await invoke('add_manual_entry', {
            request: {
              task_id: task.id,
              start_time: newStartTime.toISOString(),
              end_time: newEndTime.toISOString(),
              note: entry.description || null
            }
          })
          
          console.log('🟡 Duplication successful via custom dialog')
          alert('時間エントリを複製しました。')
          onRefresh?.()
        } catch (error) {
          console.error('🟡 Failed to duplicate time entry via custom dialog:', error)
          alert(`複製に失敗しました: ${error}`)
        } finally {
          setIsProcessing(false)
        }
      }}
      onCancel={() => {
        console.log('🟡 User cancelled duplication via custom dialog')
        setShowDuplicateDialog(false)
      }}
      confirmText="複製"
      cancelText="キャンセル"
      variant="warning"
    />
    </>
  )
}

export function TimeEntriesList({ entries, projects, tasks, onRefresh }: TimeEntriesListProps) {
  console.log('TimeEntriesList rendering with:', { 
    entriesCount: entries.length, 
    projectsCount: projects.length, 
    tasksCount: tasks.length 
  })

  try {
    const groupedEntries = groupEntriesByDate(entries)
    
    return (
      <div className="time-entries-section">
        {Object.entries(groupedEntries).map(([date, dayEntries]) => {
          const totalDuration = dayEntries.reduce((total, entry) => {
            const duration = entry.end_time 
              ? new Date(entry.end_time).getTime() - new Date(entry.start_time).getTime()
              : Date.now() - new Date(entry.start_time).getTime()
            return total + duration
          }, 0)
          
          const isToday = date === new Date().toDateString()
          
          return (
            <div key={date} className="entries-day-group">
              <h3 className="day-header">
                {isToday ? 'Today' : date} - {formatDuration(totalDuration)}
              </h3>
              
              <div className="day-entries">
                {dayEntries.map((entry, index) => {
                  const task = tasks.find(t => t.id === entry.task_id)
                  const project = task ? projects.find(p => p.id === task.project_id) : undefined
                  
                  return (
                    <TimeEntryItem 
                      key={entry.id || `${entry.task_id}-${index}`} 
                      entry={entry}
                      task={task}
                      project={project}
                      onRefresh={onRefresh}
                    />
                  )
                })}
              </div>
            </div>
          )
        })}
        
        {entries.length === 0 && (
          <div className="no-entries">
            <p>No time entries for today.</p>
            <p>Start tracking time to see your entries here.</p>
          </div>
        )}
      </div>
    )
  } catch (error) {
    console.error('TimeEntriesList error:', error)
    return (
      <div className="time-entries-section">
        <div className="error-state">
          <p>時間エントリの表示中にエラーが発生しました。</p>
          <p>エラー: {error instanceof Error ? error.message : String(error)}</p>
        </div>
      </div>
    )
  }
}

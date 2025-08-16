import { useState } from 'react'
import { Project, TimeEntry } from '@/types'
import { formatDuration, formatTime, groupEntriesByDate } from '@/utils'

interface TimeEntriesListProps {
  entries: TimeEntry[]
  projects: Project[]
}

interface TimeEntryItemProps {
  entry: TimeEntry
  project: Project | undefined
}

function TimeEntryItem({ entry, project }: TimeEntryItemProps) {
  const [showActions, setShowActions] = useState(false)
  
  const duration = entry.end_time 
    ? new Date(entry.end_time).getTime() - new Date(entry.start_time).getTime()
    : Date.now() - new Date(entry.start_time).getTime()
  
  const startTime = formatTime(new Date(entry.start_time))
  const endTime = entry.end_time ? formatTime(new Date(entry.end_time)) : 'Running'
  
  return (
    <div className="time-entry-item">
      <div 
        className="entry-color-bar" 
        style={{ backgroundColor: project?.color || '#6c757d' }}
      />
      
      <div className="entry-content">
        <div className="entry-main">
          <h4 className="entry-task">{entry.description || 'Untitled task'}</h4>
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
        
        <div className="entry-actions">
          <button 
            className="actions-btn"
            onClick={() => setShowActions(!showActions)}
          >
            ⋯
          </button>
          {showActions && (
            <div className="actions-menu">
              <button>Edit</button>
              <button>Delete</button>
              <button>Duplicate</button>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export function TimeEntriesList({ entries, projects }: TimeEntriesListProps) {
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
                const project = projects.find(p => p.id === entry.task_id) // 修正
                return (
                  <TimeEntryItem 
                    key={entry.id || `${entry.task_id}-${index}`} 
                    entry={entry} 
                    project={project}
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
}

import { Project } from '@/types'

interface TimerSectionProps {
  taskDescription: string
  onTaskDescriptionChange: (value: string) => void
  selectedProject: Project | null
  onProjectChange: (project: Project | null) => void
  projects: Project[]
  onStartTimer: () => void
  isRunning: boolean
}

export function TimerSection({
  taskDescription,
  onTaskDescriptionChange,
  selectedProject,
  onProjectChange,
  projects,
  onStartTimer,
  isRunning
}: TimerSectionProps) {
  return (
    <div className="timer-section">
      <div className="timer-input-container">
        <input
          type="text"
          className="task-input"
          placeholder="What are you working on?"
          value={taskDescription}
          onChange={(e) => onTaskDescriptionChange(e.target.value)}
          disabled={isRunning}
        />
      </div>
      
      <div className="timer-controls">
        <div className="timer-selectors">
          <select
            className="project-selector"
            value={selectedProject?.id || ''}
            onChange={(e) => {
              const projectId = parseInt(e.target.value)
              const project = projects.find(p => p.id === projectId) || null
              onProjectChange(project)
            }}
            disabled={isRunning}
          >
            <option value="">📁 Select Project</option>
            {projects.map(project => (
              <option key={project.id} value={project.id}>
                📁 {project.name}
              </option>
            ))}
          </select>
          
          <button className="tag-selector" disabled={isRunning}>
            🏷️ Tags
          </button>
        </div>
        
        <button
          className={`start-timer-btn ${isRunning ? 'disabled' : ''}`}
          onClick={onStartTimer}
          disabled={isRunning || !selectedProject}
        >
          <span className="play-icon">▶</span>
        </button>
      </div>
      
      <div className="manual-entry">
        <button className="manual-entry-link">+ Add time manually</button>
      </div>
    </div>
  )
}

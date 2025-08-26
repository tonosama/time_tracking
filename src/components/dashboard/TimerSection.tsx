import { useState } from 'react'
import { Project, Task } from '@/types'
import { ProjectEditButton } from '../projects/ProjectEditButton'
import { ManualEntryModal } from '../time_tracking/ManualEntryModal'

interface TimerSectionProps {
  taskDescription: string
  onTaskDescriptionChange: (value: string) => void
  selectedProject: Project | null
  onProjectChange: (project: Project | null) => void
  projects: Project[]
  tasks: Task[]
  onStartTimer: () => void
  isRunning: boolean
  onProjectEdit?: (project: Project) => void
  onManualEntrySuccess?: () => void
}

export function TimerSection({
  taskDescription,
  onTaskDescriptionChange,
  selectedProject,
  onProjectChange,
  projects,
  tasks,
  onStartTimer,
  isRunning,
  onProjectEdit,
  onManualEntrySuccess
}: TimerSectionProps) {
  const [showManualEntryModal, setShowManualEntryModal] = useState(false)

  const handleManualEntryClick = () => {
    console.log('Manual entry button clicked')
    setShowManualEntryModal(true)
  }

  const handleManualEntryClose = () => {
    console.log('Manual entry modal closed')
    setShowManualEntryModal(false)
  }

  const handleManualEntrySuccess = () => {
    console.log('Manual entry added successfully')
    setShowManualEntryModal(false)
    onManualEntrySuccess?.()
  }
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
          <div className="project-selector-container">
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
            {selectedProject && onProjectEdit && (
              <ProjectEditButton
                onClick={() => {
                  console.log('TimerSection ProjectEditButton onClick', { selectedProject });
                  onProjectEdit(selectedProject);
                }}
                aria-label={`プロジェクト「${selectedProject.name}」を編集`}
                className="project-selector-edit-btn"
              />
            )}

          </div>
          
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
        <button 
          className="manual-entry-link"
          onClick={handleManualEntryClick}
          type="button"
        >
          + Add time manually
        </button>
      </div>

      {/* 手動時間追加モーダル */}
      <ManualEntryModal
        isOpen={showManualEntryModal}
        onClose={handleManualEntryClose}
        onSuccess={handleManualEntrySuccess}
        selectedProject={selectedProject || undefined}
        projects={projects}
        tasks={tasks}
      />
    </div>
  )
}

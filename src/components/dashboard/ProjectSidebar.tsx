import { Project, TimeEntry } from '@/types'
import { formatDuration, Logger } from '@/utils'
import { invoke } from '@tauri-apps/api/core'
import { useState } from 'react'
import { ProjectEditButton } from '../projects/ProjectEditButton'

interface ProjectSidebarProps {
  projects: Project[]
  timeEntries: TimeEntry[]
  selectedProject: Project | null
  onProjectSelect: (project: Project) => void
  onRefresh: () => void
  onProjectEdit?: (project: Project) => void
}

interface ProjectItemProps {
  project: Project
  totalTime: number
  taskCount: number
  isSelected: boolean
  onClick: () => void
  onEdit?: (project: Project) => void
}

function ProjectItem({ project, totalTime, taskCount, isSelected, onClick, onEdit }: ProjectItemProps) {
  console.log('[ProjectItem] Rendering project:', { id: project.id, name: project.name, isSelected })
  
  const handleClick = () => {
    console.log('[ProjectItem] Clicked project:', { id: project.id, name: project.name })
    Logger.userAction('ProjectSidebar', 'project_selected', { 
      projectId: project.id,
      projectName: project.name,
      totalTime: totalTime,
      taskCount: taskCount
    });
    onClick();
  };

  return (
    <div 
      className={`project-item ${isSelected ? 'selected' : ''}`}
      onClick={handleClick}
    >
      <div 
        className="project-color-indicator"
        style={{ backgroundColor: project.color || '#6c757d' }}
      />
      <span className="project-icon">📁</span>
      <div className="project-info">
        <span className="project-name">{project.name}</span>
        <span className="project-stats">{taskCount} tasks</span>
      </div>
      <span className="project-time">{formatDuration(totalTime)}</span>
      {onEdit && (
        <ProjectEditButton
          onClick={() => {
            console.log('ProjectSidebar ProjectEditButton onClick', { project });
            onEdit(project)
          }}
          aria-label={`プロジェクト「${project.name}」を編集`}
          className="project-item-edit-btn"
        />
      )}

    </div>
  )
}

export function ProjectSidebar({ 
  projects, 
  timeEntries, 
  selectedProject, 
  onProjectSelect,
  onRefresh,
  onProjectEdit
}: ProjectSidebarProps) {
  console.log('[ProjectSidebar] Component rendering')
  // デバッグ用ログ
  console.log('[ProjectSidebar] Props:', {
    projectsCount: projects?.length || 0,
    projects: projects,
    timeEntriesCount: timeEntries?.length || 0,
    selectedProject: selectedProject
  });
  const [showCreateForm, setShowCreateForm] = useState(false)
  const [newProjectName, setNewProjectName] = useState('')
  const [isCreating, setIsCreating] = useState(false)
  // プロジェクトごとの時間を計算
  const projectTimes = projects.map(project => {
    // プロジェクトのタスク数を計算（現在は簡易的に1として扱う）
    const taskCount = 1 // TODO: 実際のタスク数を取得する必要がある
    
    // プロジェクトの時間を計算（現在は簡易的に0として扱う）
    const totalTime = 0 // TODO: プロジェクト配下のタスクの時間を合計する必要がある
    
    return { project, totalTime, taskCount }
  })

  const handleCreateProject = async () => {
    Logger.userAction('ProjectSidebar', 'create_project_initiated', { 
      projectName: newProjectName.trim() 
    });

    if (!newProjectName.trim()) {
      Logger.warn('ProjectSidebar', 'Project creation failed: empty name');
      alert('プロジェクト名を入力してください')
      return
    }

    // Tauriの初期化チェック
    if (typeof invoke === 'undefined') {
      Logger.error('ProjectSidebar', 'Tauri invoke function not available');
      alert('Tauriアプリケーションとして実行してください')
      return
    }

    setIsCreating(true)
    try {
      Logger.apiCall('ProjectSidebar', 'POST', 'create_project', { 
        projectName: newProjectName.trim() 
      });

      const result = await invoke('create_project', {
        request: { name: newProjectName.trim() }
      }) as { id: number; name: string; status: string; effective_at: string }

      Logger.apiSuccess('ProjectSidebar', 'POST', 'create_project', { 
        projectId: result.id,
        projectName: result.name 
      });
      
      setNewProjectName('')
      setShowCreateForm(false)
      onRefresh() // データを再読み込み

      Logger.userAction('ProjectSidebar', 'create_project_completed', { 
        projectId: result.id,
        projectName: result.name 
      });
    } catch (error) {
      Logger.apiError('ProjectSidebar', 'POST', 'create_project', error, { 
        projectName: newProjectName.trim() 
      });
      alert(`プロジェクトの作成に失敗しました: ${error}`)
    } finally {
      setIsCreating(false)
    }
  }

  const handleCancelCreate = () => {
    Logger.userAction('ProjectSidebar', 'create_project_cancelled', { 
      projectName: newProjectName.trim() 
    });
    setNewProjectName('')
    setShowCreateForm(false)
  }
  
  return (
    <div className="project-sidebar" data-testid="project-sidebar">
      <div className="sidebar-header">
        <h3>Projects</h3>
        <button 
          className="add-project-btn"
          onClick={() => {
            Logger.userAction('ProjectSidebar', 'show_create_form_header');
            setShowCreateForm(true);
          }}
          disabled={showCreateForm}
        >
          +
        </button>
      </div>
      
      <div className="projects-list">
        {(() => {
          console.log('ProjectSidebar - Rendering projects list:', {
            projectTimesLength: projectTimes.length,
            projectTimes: projectTimes,
            projectsLength: projects.length,
            projects: projects
          });
          
          return projectTimes.length > 0 ? (
            projectTimes.map(({ project, totalTime, taskCount }) => (
              <ProjectItem
                key={project.id}
                project={project}
                totalTime={totalTime}
                taskCount={taskCount}
                isSelected={selectedProject?.id === project.id}
                onClick={() => onProjectSelect(project)}
                onEdit={onProjectEdit}
              />
            ))
          ) : (
            <div className="no-projects">
              <p>プロジェクトがありません</p>
              <p>新しいプロジェクトを作成してください</p>
            </div>
          );
        })()}
      </div>
      
      {showCreateForm && (
        <div className="create-project-form">
          <input
            type="text"
            className="project-name-input"
            placeholder="プロジェクト名を入力"
            value={newProjectName}
            onChange={(e) => setNewProjectName(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                handleCreateProject()
              } else if (e.key === 'Escape') {
                handleCancelCreate()
              }
            }}
            autoFocus
          />
          <div className="form-actions">
            <button 
              className="create-btn"
              onClick={handleCreateProject}
              disabled={isCreating || !newProjectName.trim()}
            >
              {isCreating ? '作成中...' : '作成'}
            </button>
            <button 
              className="cancel-btn"
              onClick={handleCancelCreate}
              disabled={isCreating}
            >
              キャンセル
            </button>
          </div>
        </div>
      )}
      
      <div className="sidebar-footer">
        <button 
          className="add-project-link"
          onClick={() => {
            Logger.userAction('ProjectSidebar', 'show_create_form');
            setShowCreateForm(true);
          }}
          disabled={showCreateForm}
        >
          + Add new project
        </button>
      </div>
    </div>
  )
}

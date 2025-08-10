import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { Project } from '@/types'

interface ProjectListProps {
  showArchived?: boolean
}

export function ProjectList({ showArchived = false }: ProjectListProps) {
  const [projects, setProjects] = useState<Project[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [isShowingArchived, setIsShowingArchived] = useState(showArchived)

  const loadProjects = async () => {
    try {
      setLoading(true)
      setError(null)
      
      const command = isShowingArchived ? 'get_all_projects' : 'get_all_active_projects'
      const result = await invoke<Project[]>(command)
      setProjects(result)
    } catch (err) {
      setError('プロジェクトの読み込みに失敗しました')
      console.error('プロジェクト読み込みエラー:', err)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadProjects()
  }, [isShowingArchived])

  const handleArchiveProject = async (projectId: number) => {
    try {
      await invoke('archive_project', { id: projectId, force: false })
      await loadProjects() // リフレッシュ
    } catch (err) {
      console.error('プロジェクトアーカイブエラー:', err)
    }
  }

  const handleRestoreProject = async (projectId: number) => {
    try {
      await invoke('restore_project', { id: projectId })
      await loadProjects() // リフレッシュ
    } catch (err) {
      console.error('プロジェクト復元エラー:', err)
    }
  }

  const toggleArchiveView = () => {
    setIsShowingArchived(!isShowingArchived)
  }

  if (loading) {
    return (
      <div className="project-list">
        <p>プロジェクトを読み込み中...</p>
      </div>
    )
  }

  if (error) {
    return (
      <div className="project-list">
        <p className="error">{error}</p>
      </div>
    )
  }

  const activeProjects = projects.filter(p => p.status === 'active')
  const archivedProjects = projects.filter(p => p.status === 'archived')

  return (
    <div className="project-list">
      <div className="project-list-header">
        <h2>プロジェクト一覧</h2>
        <div className="project-list-actions">
          <button 
            type="button"
            className="btn btn-primary"
            aria-label="新しいプロジェクト"
          >
            新しいプロジェクト
          </button>
          <button
            type="button"
            className="btn btn-secondary"
            onClick={toggleArchiveView}
            aria-label={isShowingArchived ? 'アクティブのみ表示' : 'アーカイブ済みを表示'}
          >
            {isShowingArchived ? 'アクティブのみ表示' : 'アーカイブ済みを表示'}
          </button>
        </div>
      </div>

      <div className="projects-grid">
        {isShowingArchived ? (
          // 全プロジェクト表示（アクティブ + アーカイブ）
          projects.map(project => (
            <ProjectCard
              key={project.id}
              project={project}
              onArchive={handleArchiveProject}
              onRestore={handleRestoreProject}
            />
          ))
        ) : (
          // アクティブプロジェクトのみ表示
          activeProjects.map(project => (
            <ProjectCard
              key={project.id}
              project={project}
              onArchive={handleArchiveProject}
              onRestore={handleRestoreProject}
            />
          ))
        )}
      </div>

      {projects.length === 0 && (
        <div className="empty-state">
          <p>プロジェクトがありません</p>
        </div>
      )}
    </div>
  )
}

interface ProjectCardProps {
  project: Project
  onArchive: (id: number) => void
  onRestore: (id: number) => void
}

function ProjectCard({ project, onArchive, onRestore }: ProjectCardProps) {
  return (
    <div className="project-card" data-testid="project-card">
      <div className="project-card-header">
        <h3 className="project-name">{project.name}</h3>
        <span className={`project-status ${project.status}`}>
          {project.status === 'active' ? 'アクティブ' : 'アーカイブ済み'}
        </span>
      </div>
      
      <div className="project-card-meta">
        <time dateTime={project.effective_at}>
          更新: {new Date(project.effective_at).toLocaleDateString('ja-JP')}
        </time>
      </div>

      <div className="project-card-actions">
        {project.status === 'active' ? (
          <button
            type="button"
            className="btn btn-warning btn-sm"
            onClick={() => onArchive(project.id)}
            aria-label={`プロジェクト${project.id}をアーカイブ`}
          >
            アーカイブ
          </button>
        ) : (
          <button
            type="button"
            className="btn btn-success btn-sm"
            onClick={() => onRestore(project.id)}
            aria-label={`プロジェクト${project.id}を復元`}
          >
            復元
          </button>
        )}
      </div>
    </div>
  )
}

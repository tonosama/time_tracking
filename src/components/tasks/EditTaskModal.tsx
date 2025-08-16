import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Modal } from '../common/Modal'
import { Button } from '../common/Button'
import { Input } from '../common/Input'
import type { Task, Project } from '@/types'

interface EditTaskModalProps {
  isOpen: boolean
  onClose: () => void
  onTaskUpdated: (task: Task) => void
  task: Task
}

interface ProjectOption {
  value: number
  label: string
}

export function EditTaskModal({ isOpen, onClose, onTaskUpdated, task }: EditTaskModalProps) {
  const [name, setName] = useState(task.name)
  const [selectedProjectId, setSelectedProjectId] = useState(task.project_id)
  const [projects, setProjects] = useState<ProjectOption[]>([])
  const [loading, setLoading] = useState(false)
  const [loadingProjects, setLoadingProjects] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // プロジェクト一覧を読み込み
  useEffect(() => {
    const loadProjects = async () => {
      try {
        setLoadingProjects(true)
        const result = await invoke<Project[]>('get_all_active_projects')
        const projectOptions = result.map(p => ({
          value: p.id,
          label: p.name
        }))
        setProjects(projectOptions)
      } catch (err) {
        console.error('プロジェクト読み込みエラー:', err)
      } finally {
        setLoadingProjects(false)
      }
    }

    if (isOpen) {
      loadProjects()
    }
  }, [isOpen])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!name.trim()) {
      setError('タスク名を入力してください')
      return
    }

    try {
      setLoading(true)
      setError(null)
      
      const updateRequest: any = {
        id: task.id,
        name: name.trim()
      }
      
      // プロジェクトが変更された場合のみproject_idを含める
      if (selectedProjectId !== task.project_id) {
        updateRequest.project_id = selectedProjectId
      }

      const updatedTask = await invoke<Task>('update_task', {
        request: updateRequest
      })
      
      onTaskUpdated(updatedTask)
      handleClose()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'タスクの更新に失敗しました')
    } finally {
      setLoading(false)
    }
  }

  const handleClose = () => {
    setName(task.name)
    setSelectedProjectId(task.project_id)
    setError(null)
    onClose()
  }

  return (
    <Modal isOpen={isOpen} onClose={handleClose} title="タスクを編集">
      <form onSubmit={handleSubmit} className="edit-task-form">
        <Input
          label="タスク名"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="タスク名を入力"
          error={error || undefined}
          required
          autoFocus
        />

        <div className="input-group">
          <label htmlFor="project-select" className="input-label">
            プロジェクト
          </label>
          <select
            id="project-select"
            className="input"
            value={selectedProjectId}
            onChange={(e) => setSelectedProjectId(Number(e.target.value))}
            disabled={loadingProjects}
          >
            {projects.map(project => (
              <option key={project.value} value={project.value}>
                {project.label}
              </option>
            ))}
          </select>
          {loadingProjects && (
            <span className="input-help-text">プロジェクトを読み込み中...</span>
          )}
        </div>
        
        <div className="modal-actions">
          <Button
            type="button"
            variant="secondary"
            onClick={handleClose}
            disabled={loading}
          >
            キャンセル
          </Button>
          <Button
            type="submit"
            variant="primary"
            loading={loading}
            disabled={!name.trim() || loading || loadingProjects}
          >
            保存
          </Button>
        </div>
      </form>
    </Modal>
  )
}

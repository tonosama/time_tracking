import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Modal } from '../common/Modal'
import { Button } from '../common/Button'
import { Input } from '../common/Input'
import type { Task } from '@/types'

interface CreateTaskModalProps {
  isOpen: boolean
  onClose: () => void
  onTaskCreated: (task: Task) => void
  projectId: number
}

export function CreateTaskModal({ isOpen, onClose, onTaskCreated, projectId }: CreateTaskModalProps) {
  const [name, setName] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!name.trim()) {
      setError('タスク名を入力してください')
      return
    }

    try {
      setLoading(true)
      setError(null)
      
      const task = await invoke<Task>('create_task', {
        request: { 
          project_id: projectId,
          name: name.trim() 
        }
      })
      
      onTaskCreated(task)
      handleClose()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'タスクの作成に失敗しました')
    } finally {
      setLoading(false)
    }
  }

  const handleClose = () => {
    setName('')
    setError(null)
    onClose()
  }

  return (
    <Modal isOpen={isOpen} onClose={handleClose} title="新しいタスク">
      <form onSubmit={handleSubmit} className="create-task-form">
        <Input
          label="タスク名"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="タスク名を入力"
          error={error || undefined}
          required
          autoFocus
        />
        
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
            disabled={!name.trim() || loading}
          >
            作成
          </Button>
        </div>
      </form>
    </Modal>
  )
}

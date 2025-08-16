import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Modal } from '../common/Modal'
import { Button } from '../common/Button'
import { Input } from '../common/Input'
import type { Project } from '@/types'

interface CreateProjectModalProps {
  isOpen: boolean
  onClose: () => void
  onProjectCreated: (project: Project) => void
}

export function CreateProjectModal({ isOpen, onClose, onProjectCreated }: CreateProjectModalProps) {
  const [name, setName] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!name.trim()) {
      setError('プロジェクト名を入力してください')
      return
    }

    try {
      setLoading(true)
      setError(null)
      
      // Tauriが利用可能かチェック
      if (typeof window !== 'undefined' && window.__TAURI__) {
        const project = await invoke<Project>('create_project', {
          request: { name: name.trim() }
        })
        
        onProjectCreated(project)
        handleClose()
      } else {
        throw new Error('Tauriが初期化されていません')
      }
    } catch (err) {
      console.error('プロジェクト作成エラー:', err)
      setError(err instanceof Error ? err.message : 'プロジェクトの作成に失敗しました')
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
    <Modal isOpen={isOpen} onClose={handleClose} title="新しいプロジェクト">
      <form onSubmit={handleSubmit} className="create-project-form">
        <Input
          label="プロジェクト名"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="プロジェクト名を入力"
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

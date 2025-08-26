import { useState, useEffect, useMemo, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { Modal } from '../common/Modal'
import { Button } from '../common/Button'
import type { Project, Task } from '@/types'
import type { AddManualEntryRequest } from '@/services/timeTrackingApi'

interface ManualEntryModalProps {
  isOpen: boolean
  onClose: () => void
  onSuccess: () => void
  selectedProject?: Project
  projects: Project[]
  tasks: Task[]
}

interface FormData {
  project_id: number | null
  task_id: number | null
  start_time: string
  end_time: string
  note: string
}

interface ValidationErrors {
  project?: string
  task?: string
  startTime?: string
  endTime?: string
  timeRange?: string
}

export function ManualEntryModal({
  isOpen,
  onClose,
  onSuccess,
  selectedProject,
  projects,
  tasks
}: ManualEntryModalProps) {
  const [formData, setFormData] = useState<FormData>({
    project_id: selectedProject?.id || null,
    task_id: null,
    start_time: '',
    end_time: '',
    note: ''
  })
  
  const [errors, setErrors] = useState<ValidationErrors>({})
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string>('')

  // selectedProjectが変更された時にフォームを更新
  useEffect(() => {
    if (selectedProject) {
      setFormData(prev => ({
        ...prev,
        project_id: selectedProject.id,
        task_id: null // プロジェクト変更時はタスクをリセット
      }))
    }
  }, [selectedProject])

  // 選択されたプロジェクトのタスクのみをフィルタリング
  const availableTasks = useMemo(() => {
    if (!formData.project_id) return []
    return tasks.filter(task => task.project_id === formData.project_id && task.status === 'active')
  }, [tasks, formData.project_id])

  // フォームデータ変更時のリアルタイムバリデーション
  useEffect(() => {
    // 時間フィールドの変更を監視してバリデーションを実行
    if (formData.start_time || formData.end_time) {
      const validationErrors = validateForm()
      
      // 時間関連のエラーのみを更新
      setErrors(prev => ({
        ...prev,
        timeRange: validationErrors.timeRange,
        startTime: validationErrors.startTime,
        endTime: validationErrors.endTime
      }))
    }
  }, [formData.start_time, formData.end_time])

  // 継続時間の計算
  const duration = useMemo(() => {
    if (!formData.start_time || !formData.end_time) return null
    
    const start = new Date(formData.start_time)
    const end = new Date(formData.end_time)
    
    if (isNaN(start.getTime()) || isNaN(end.getTime())) return null
    if (end <= start) return null
    
    const diffMs = end.getTime() - start.getTime()
    const hours = Math.floor(diffMs / (1000 * 60 * 60))
    const minutes = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60))
    
    if (hours > 0 && minutes > 0) {
      return `${hours}時間${minutes}分`
    } else if (hours > 0) {
      return `${hours}時間`
    } else if (minutes > 0) {
      return `${minutes}分`
    } else {
      return '1分未満'
    }
  }, [formData.start_time, formData.end_time])

  // バリデーション
  const validateForm = (): ValidationErrors => {
    const newErrors: ValidationErrors = {}
    
    if (!formData.project_id) {
      newErrors.project = 'プロジェクトを選択してください'
    }
    
    if (!formData.task_id) {
      newErrors.task = 'タスクを選択してください'
    }
    
    // 時間フィールドの詳細バリデーション
    if (formData.start_time && formData.end_time) {
      const start = new Date(formData.start_time)
      const end = new Date(formData.end_time)
      const now = new Date()
      
      // 時間範囲チェックを最初に行う
      if (start >= end) {
        newErrors.timeRange = '終了時刻は開始時刻より後の時刻を指定してください'
      }
      
      // 未来時刻チェック
      if (start > now) {
        newErrors.startTime = '未来の時刻が指定されています'
      }
      
      if (end > now) {
        newErrors.endTime = '未来の時刻が指定されています'
      }
    } else {
      // 開始時刻のみが入力されている場合の未来時刻チェック
      if (formData.start_time) {
        const start = new Date(formData.start_time)
        const now = new Date()
        
        if (start > now) {
          newErrors.startTime = '未来の時刻が指定されています'
        }
      } else {
        newErrors.startTime = '開始時刻を入力してください'
      }
      
      // 終了時刻のみが入力されている場合の未来時刻チェック
      if (formData.end_time) {
        const end = new Date(formData.end_time)
        const now = new Date()
        
        if (end > now) {
          newErrors.endTime = '未来の時刻が指定されています'
        }
      } else {
        newErrors.endTime = '終了時刻を入力してください'
      }
    }
    
    return newErrors
  }

  // フォーム更新ハンドラー
  const handleInputChange = (field: keyof FormData, value: string | number | null) => {
    setFormData(prev => ({
      ...prev,
      [field]: value
    }))
    
    // プロジェクト変更時はタスクをリセット
    if (field === 'project_id') {
      setFormData(prev => ({
        ...prev,
        project_id: value as number | null,
        task_id: null
      }))
    }
    
    // エラーをクリア（時間フィールド以外）
    if (field !== 'start_time' && field !== 'end_time') {
      setErrors(prev => ({
        ...prev,
        [field]: undefined
      }))
    }
    setSubmitError('')
  }

  // フォーム送信
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    const validationErrors = validateForm()
    setErrors(validationErrors)
    
    if (Object.keys(validationErrors).length > 0) {
      return
    }
    
    if (!formData.task_id) {
      return
    }
    
    setIsSubmitting(true)
    setSubmitError('')
    
    try {
      const request: AddManualEntryRequest = {
        task_id: formData.task_id,
        start_time: new Date(formData.start_time).toISOString(),
        end_time: new Date(formData.end_time).toISOString(),
        note: formData.note || undefined
      }
      
      console.log('Submitting manual entry:', request)
      
      await invoke('add_manual_entry', { request })
      
      console.log('Manual entry added successfully')
      onSuccess()
      onClose()
      
      // フォームリセット
      setFormData({
        project_id: selectedProject?.id || null,
        task_id: null,
        start_time: '',
        end_time: '',
        note: ''
      })
      
    } catch (error) {
      console.error('Failed to add manual entry:', error)
      setSubmitError(`時間エントリの追加に失敗しました: ${error}`)
    } finally {
      setIsSubmitting(false)
    }
  }

  // フォームの有効性チェック
  const isFormValid = formData.project_id && 
                     formData.task_id && 
                     formData.start_time && 
                     formData.end_time && 
                     Object.keys(validateForm()).length === 0

  if (!isOpen) return null

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="手動で時間を追加">
      <form onSubmit={handleSubmit} className="manual-entry-form">
        {/* プロジェクト選択 */}
        <div className="form-group">
          <label htmlFor="project-select" className="form-label">プロジェクト</label>
          <select
            id="project-select"
            className={`form-select ${errors.project ? 'error' : ''}`}
            value={formData.project_id || ''}
            onChange={(e) => handleInputChange('project_id', e.target.value ? parseInt(e.target.value) : null)}
            disabled={isSubmitting}
          >
            <option value="">プロジェクトを選択...</option>
            {projects
              .filter(project => project.status === 'active')
              .map(project => (
                <option key={project.id} value={project.id}>
                  📁 {project.name}
                </option>
              ))
            }
          </select>
          {errors.project && <div className="error-message">{errors.project}</div>}
        </div>

        {/* タスク選択 */}
        <div className="form-group">
          <label htmlFor="task-select" className="form-label">タスク</label>
          <select
            id="task-select"
            className={`form-select ${errors.task ? 'error' : ''}`}
            value={formData.task_id || ''}
            onChange={(e) => handleInputChange('task_id', e.target.value ? parseInt(e.target.value) : null)}
            disabled={!formData.project_id || isSubmitting}
          >
            <option value="">タスクを選択...</option>
            {availableTasks.map(task => (
              <option key={task.id} value={task.id}>
                {task.name}
              </option>
            ))}
          </select>
          {errors.task && <div className="error-message">{errors.task}</div>}
        </div>

        {/* 開始時刻 */}
        <div className="form-group">
          <label htmlFor="start-time" className="form-label">開始時刻</label>
          <input
            type="datetime-local"
            id="start-time"
            className={`form-input ${errors.startTime ? 'error' : ''}`}
            value={formData.start_time}
            onChange={(e) => handleInputChange('start_time', e.target.value)}
            disabled={isSubmitting}
          />
          {errors.startTime && <div className="error-message">{errors.startTime}</div>}
        </div>

        {/* 終了時刻 */}
        <div className="form-group">
          <label htmlFor="end-time" className="form-label">終了時刻</label>
          <input
            type="datetime-local"
            id="end-time"
            className={`form-input ${errors.endTime ? 'error' : ''}`}
            value={formData.end_time}
            onChange={(e) => handleInputChange('end_time', e.target.value)}
            disabled={isSubmitting}
          />
          {errors.endTime && <div className="error-message">{errors.endTime}</div>}
        </div>

        {/* 時間範囲エラー */}
        {errors.timeRange && (
          <div className="error-message">{errors.timeRange}</div>
        )}

        {/* 継続時間表示 */}
        {duration && (
          <div className="duration-display">
            継続時間: {duration}
          </div>
        )}

        {/* メモ */}
        <div className="form-group">
          <label htmlFor="note" className="form-label">メモ (オプション)</label>
          <textarea
            id="note"
            className="form-textarea"
            value={formData.note}
            onChange={(e) => handleInputChange('note', e.target.value)}
            placeholder="作業内容や備考を入力..."
            rows={3}
            disabled={isSubmitting}
          />
        </div>

        {/* 送信エラー */}
        {submitError && (
          <div className="error-message">{submitError}</div>
        )}

        {/* ボタン */}
        <div className="modal-actions">
          <Button
            type="button"
            variant="secondary"
            onClick={onClose}
            disabled={isSubmitting}
          >
            キャンセル
          </Button>
          <Button
            type="submit"
            variant="primary"
            disabled={!isFormValid || isSubmitting}
          >
            {isSubmitting ? '追加中...' : '追加'}
          </Button>
        </div>
      </form>
    </Modal>
  )
}

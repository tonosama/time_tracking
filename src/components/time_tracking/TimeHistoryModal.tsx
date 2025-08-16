import { useState, useEffect } from 'react'
import { Modal } from '@/components/common'
import { TimeTrackingApi, TimeFormatUtils } from '@/services'
import type { TimeEntry, Task } from '@/types'

interface TimeHistoryModalProps {
  isOpen: boolean
  onClose: () => void
  task: Task
}

export function TimeHistoryModal({ isOpen, onClose, task }: TimeHistoryModalProps) {
  const [entries, setEntries] = useState<TimeEntry[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // 時間エントリを読み込み
  const loadTimeEntries = async () => {
    setLoading(true)
    setError(null)
    
    try {
      const taskEntries = await TimeTrackingApi.getTaskEntries(task.id)
      setEntries(taskEntries)
    } catch (err) {
      console.error('Failed to load time entries:', err)
      setError('時間履歴の読み込みに失敗しました')
    } finally {
      setLoading(false)
    }
  }

  // モーダルが開かれた時にデータを読み込み
  useEffect(() => {
    if (isOpen) {
      loadTimeEntries()
    }
  }, [isOpen, task.id])

  // 合計時間を計算
  const totalSeconds = entries
    .filter(entry => entry.duration_in_seconds)
    .reduce((sum, entry) => sum + (entry.duration_in_seconds || 0), 0)

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={`タイム履歴 - ${task.name}`}
    >
      <div className="time-history-modal">
        {/* サマリー情報 */}
        <div className="time-history-summary">
          <div className="time-history-stats">
            <div className="stat-item">
              <span className="stat-label">総作業時間:</span>
              <span className="stat-value">{TimeFormatUtils.formatDuration(totalSeconds)}</span>
            </div>
            <div className="stat-item">
              <span className="stat-label">セッション数:</span>
              <span className="stat-value">{entries.length}</span>
            </div>
          </div>
        </div>

        {/* ローディング状態 */}
        {loading && (
          <div className="time-history-loading">
            <div className="loading-spinner"></div>
            <p>時間履歴を読み込み中...</p>
          </div>
        )}

        {/* エラー状態 */}
        {error && (
          <div className="time-history-error">
            <div className="alert alert-danger" role="alert">
              {error}
            </div>
            <button
              type="button"
              className="btn btn-outline-primary"
              onClick={loadTimeEntries}
            >
              再試行
            </button>
          </div>
        )}

        {/* 履歴一覧 */}
        {!loading && !error && (
          <div className="time-history-list">
            {entries.length === 0 ? (
              <div className="time-history-empty">
                <p>まだ作業時間の記録がありません。</p>
                <p>タイマーを開始して作業を開始しましょう。</p>
              </div>
            ) : (
              <div className="time-history-entries">
                <div className="time-history-header">
                  <div className="entry-header-date">日時</div>
                  <div className="entry-header-duration">作業時間</div>
                  <div className="entry-header-status">状態</div>
                </div>
                
                {entries.map((entry) => (
                  <div 
                    key={entry.start_event_id} 
                    className={`time-entry ${entry.is_running ? 'entry-running' : 'entry-completed'}`}
                    data-testid="time-entry"
                  >
                    <div className="entry-date">
                      <div className="entry-start-time">
                        開始: {TimeFormatUtils.formatDateTime(entry.start_time)}
                      </div>
                      {entry.end_time && (
                        <div className="entry-end-time">
                          終了: {TimeFormatUtils.formatDateTime(entry.end_time)}
                        </div>
                      )}
                    </div>
                    
                    <div className="entry-duration">
                      <span className="duration-text">
                        {entry.elapsed_duration}
                      </span>
                      {entry.duration_in_seconds && (
                        <small className="duration-seconds">
                          ({entry.duration_in_seconds}秒)
                        </small>
                      )}
                    </div>
                    
                    <div className="entry-status">
                      {entry.is_running ? (
                        <span className="status-running">
                          <span className="status-indicator running"></span>
                          実行中
                        </span>
                      ) : (
                        <span className="status-completed">
                          <span className="status-indicator completed"></span>
                          完了
                        </span>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* アクションボタン */}
        <div className="time-history-actions">
          <button
            type="button"
            className="btn btn-secondary"
            onClick={onClose}
          >
            閉じる
          </button>
          
          {!loading && !error && entries.length > 0 && (
            <button
              type="button"
              className="btn btn-outline-primary"
              onClick={loadTimeEntries}
            >
              更新
            </button>
          )}
        </div>
      </div>
    </Modal>
  )
}

// 時間エントリの簡潔な表示用コンポーネント
export function TimeEntryCard({ entry }: { entry: TimeEntry }) {
  return (
    <div className={`time-entry-card ${entry.is_running ? 'running' : 'completed'}`}>
      <div className="entry-card-header">
        <span className="entry-card-time">
          {TimeFormatUtils.formatTime(entry.start_time)}
        </span>
        {entry.end_time && (
          <span className="entry-card-duration">
            {entry.elapsed_duration}
          </span>
        )}
      </div>
      
      <div className="entry-card-status">
        {entry.is_running ? (
          <span className="status-badge running">実行中</span>
        ) : (
          <span className="status-badge completed">完了</span>
        )}
      </div>
    </div>
  )
}

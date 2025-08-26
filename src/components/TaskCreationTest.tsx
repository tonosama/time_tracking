import { useState } from 'react'
import { CreateTaskModal } from './tasks/CreateTaskModal'
import { useTasks } from '../hooks/useTasks'
import type { Task } from '@/types'

export function TaskCreationTest() {
  const [isModalOpen, setIsModalOpen] = useState(false)
  const [selectedProjectId, setSelectedProjectId] = useState(1)
  const { tasks, loading, error, loadTasks } = useTasks()

  const handleTaskCreated = (task: Task) => {
    console.log('タスクが作成されました:', task)
    // タスクリストを再読み込み
    loadTasks(selectedProjectId)
  }

  const handleLoadTasks = () => {
    loadTasks(selectedProjectId)
  }

  return (
    <div className="p-8 max-w-4xl mx-auto">
      <h1 className="text-3xl font-bold mb-8">タスク作成機能テスト</h1>
      
      {/* プロジェクト選択 */}
      <div className="mb-6">
        <label className="block text-sm font-medium mb-2">
          プロジェクトID:
        </label>
        <select
          value={selectedProjectId}
          onChange={(e) => setSelectedProjectId(Number(e.target.value))}
          className="border rounded px-3 py-2"
        >
          <option value={1}>プロジェクト1</option>
          <option value={2}>プロジェクト2</option>
        </select>
      </div>

      {/* タスク作成ボタン */}
      <div className="mb-6">
        <button
          onClick={() => setIsModalOpen(true)}
          className="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600"
        >
          新しいタスクを作成
        </button>
      </div>

      {/* タスク読み込みボタン */}
      <div className="mb-6">
        <button
          onClick={handleLoadTasks}
          disabled={loading}
          className="bg-green-500 text-white px-4 py-2 rounded hover:bg-green-600 disabled:opacity-50"
        >
          {loading ? '読み込み中...' : 'タスクを読み込み'}
        </button>
      </div>

      {/* エラー表示 */}
      {error && (
        <div className="mb-6 p-4 bg-red-100 border border-red-400 text-red-700 rounded">
          エラー: {error}
        </div>
      )}

      {/* タスクリスト */}
      <div className="mb-6">
        <h2 className="text-xl font-semibold mb-4">タスク一覧</h2>
        {loading ? (
          <p>読み込み中...</p>
        ) : tasks.length === 0 ? (
          <p className="text-gray-500">タスクがありません</p>
        ) : (
          <ul className="space-y-2">
            {tasks.map((task) => (
              <li
                key={task.id}
                className="p-3 border rounded bg-white shadow-sm"
              >
                <div className="font-medium">{task.name}</div>
                <div className="text-sm text-gray-500">
                  ID: {task.id} | プロジェクトID: {task.project_id} | ステータス: {task.status}
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>

      {/* タスク作成モーダル */}
      <CreateTaskModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        onTaskCreated={handleTaskCreated}
        projectId={selectedProjectId}
      />

      {/* テスト手順 */}
      <div className="mt-8 p-4 bg-blue-50 border border-blue-200 rounded">
        <h3 className="font-semibold mb-2">テスト手順:</h3>
        <ol className="list-decimal list-inside space-y-1 text-sm">
          <li>プロジェクトIDを選択してください</li>
          <li>「タスクを読み込み」ボタンをクリックして既存タスクを表示</li>
          <li>「新しいタスクを作成」ボタンをクリック</li>
          <li>タスク名を入力して「作成」ボタンをクリック</li>
          <li>タスクが作成され、リストに追加されることを確認</li>
          <li>ESCキーやキャンセルボタンでモーダルが閉じることを確認</li>
        </ol>
      </div>
    </div>
  )
}

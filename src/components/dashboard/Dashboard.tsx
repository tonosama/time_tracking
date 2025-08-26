import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { TimerSection } from './TimerSection'
import { TimeEntriesList } from './TimeEntriesList'
import { ProjectSidebar } from './ProjectSidebar'
import { ProjectEditModal } from '../projects/ProjectEditModal'
import { ProjectDetailView } from '../projects/ProjectDetailView'
import { Project, TimeEntry, TimerStatus, Task } from '@/types'
import { Logger } from '@/utils'

export function Dashboard() {
  console.log('[Dashboard] Component rendering')
  console.log('[Dashboard] Component props:', {})
  const [projects, setProjects] = useState<Project[]>([])
  const [tasks, setTasks] = useState<Task[]>([]) // タスクデータを追加
  const [timeEntries, setTimeEntries] = useState<TimeEntry[]>([])
  const [timerStatus, setTimerStatus] = useState<TimerStatus | null>(null)
  const [selectedProject, setSelectedProject] = useState<Project | null>(null)
  const [taskDescription, setTaskDescription] = useState('')
  const [showEditModal, setShowEditModal] = useState(false)
  const [editingProject, setEditingProject] = useState<Project | null>(null)

  useEffect(() => {
    loadData()
  }, [])

  const loadData = async () => {
    Logger.debug('Dashboard', 'Loading dashboard data');
    const startTime = performance.now();
    
    try {
      Logger.debug('Dashboard', 'Starting API calls');
      
      // 個別にAPIを呼び出してエラーハンドリング
      let projectsData: Project[] = [];
      let tasksData: Task[] = []; // タスクデータを追加
      let entriesData: TimeEntry[] = [];
      let statusData: any = null;

      // プロジェクト一覧を取得
      try {
        Logger.debug('Dashboard', 'Calling get_all_active_projects');
        projectsData = await invoke<Project[]>('get_all_active_projects') || [];
        Logger.debug('Dashboard', 'get_all_active_projects success', { count: projectsData.length });
      } catch (e) {
        Logger.error('Dashboard', 'get_all_active_projects failed', { error: String(e) });
        // プロジェクト取得に失敗しても他の処理は継続
        projectsData = [];
      }

      // タスク一覧を取得
      try {
        Logger.debug('Dashboard', 'Calling get_all_active_tasks');
        tasksData = await invoke<Task[]>('get_all_active_tasks') || [];
        Logger.debug('Dashboard', 'get_all_active_tasks success', { count: tasksData.length });
      } catch (e) {
        Logger.error('Dashboard', 'get_all_active_tasks failed', { error: String(e) });
        tasksData = [];
      }

      // テストコマンドを呼び出し
      try {
        Logger.debug('Dashboard', 'Calling test_get_time_entries');
        const testResult = await invoke<string>('test_get_time_entries');
        Logger.debug('Dashboard', 'test_get_time_entries success', { result: testResult });
      } catch (e) {
        Logger.error('Dashboard', 'test_get_time_entries failed', { error: String(e) });
      }

      // 時間エントリを取得
      try {
        Logger.debug('Dashboard', 'Calling get_time_entries');
        entriesData = await invoke<TimeEntry[]>('get_time_entries', { startDate: new Date().toISOString().split('T')[0] }) || [];
        Logger.debug('Dashboard', 'get_time_entries success', { count: entriesData.length });
      } catch (e) {
        Logger.error('Dashboard', 'get_time_entries failed', { error: String(e) });
        // 時間エントリ取得に失敗しても他の処理は継続
        entriesData = [];
      }

      // タイマーステータスを取得
      try {
        Logger.debug('Dashboard', 'Calling get_global_timer_status');
        statusData = await invoke<any>('get_global_timer_status');
        Logger.debug('Dashboard', 'get_global_timer_status success', { hasStatus: !!statusData });
      } catch (e) {
        Logger.error('Dashboard', 'get_global_timer_status failed', { error: String(e) });
        // タイマーステータス取得に失敗しても他の処理は継続
        statusData = null;
      }
      
      console.log('[Dashboard] Setting state with:', { 
        projectsCount: projectsData.length,
        tasksCount: tasksData.length, // 追加
        entriesCount: entriesData.length,
        hasTimerStatus: !!statusData
      })
      
      setProjects(projectsData)
      setTasks(tasksData) // 追加: tasksの状態管理が必要
      setTimeEntries(entriesData)
      setTimerStatus(statusData)
      
      // デバッグ用ログ
      Logger.debug('Dashboard', 'Projects loaded', { 
        projectCount: projectsData?.length || 0,
        projects: projectsData 
      });
      
      // コンソールにも出力
      console.log('Dashboard - Projects loaded:', {
        projectCount: projectsData?.length || 0,
        projects: projectsData,
        projectsState: projectsData || []
      });
      
      const duration = performance.now() - startTime;
      Logger.performance('Dashboard', 'data_load', duration, {
        projectCount: projectsData?.length || 0,
        entryCount: entriesData?.length || 0,
        hasActiveTimer: !!statusData?.task_id
      });
    } catch (error) {
      // 予期しないエラーが発生した場合の処理
      console.error('Dashboard Unexpected Error:', {
        error: error,
        message: error instanceof Error ? error.message : String(error),
        name: error instanceof Error ? error.name : 'Unknown',
        stack: error instanceof Error ? error.stack : undefined,
        toString: String(error),
        constructor: error instanceof Error ? error.constructor.name : 'Unknown'
      });
      
      Logger.error('Dashboard', 'Unexpected error during data loading', { 
        error: error instanceof Error ? error.message : String(error),
        errorType: error instanceof Error ? error.constructor.name : 'Unknown',
        stack: error instanceof Error ? error.stack : undefined,
        fullError: String(error)
      });
      
      // エラー時も空の状態を設定
      setProjects([])
      setTimeEntries([])
      setTimerStatus(null)
    }
  }

  const handleStartTimer = async () => {
    // 入力メソッド関連のエラーを回避するため、少し遅延を入れる
    await new Promise(resolve => setTimeout(resolve, 100));
    
    Logger.userAction('Dashboard', 'timer_start_initiated', { 
      selectedProject: selectedProject?.id,
      taskDescription: taskDescription 
    });
    
    if (!selectedProject) {
      Logger.warn('Dashboard', 'Timer start failed: no project selected');
      alert('プロジェクトを選択してください')
      return
    }

    // デバッグ用：データベース状態を確認
    try {
      const dbState = await invoke<string>('debug_database_state');
      console.log('[Dashboard] Database state:', dbState);
    } catch (debugError) {
      console.warn('[Dashboard] Failed to get database state:', debugError);
    }

    try {
      // まず、プロジェクト配下のタスクを取得または作成
      let taskId: number
      
      // タスク入力がある場合は新しいタスクを作成、ない場合は既存タスクを探す
      if (taskDescription && taskDescription.trim()) {
        // タスク入力がある場合は新しいタスクを作成
        Logger.apiCall('Dashboard', 'POST', 'create_task', { 
          projectId: selectedProject.id,
          taskName: taskDescription 
        });
        
        try {
          console.log('[Dashboard] タスク作成開始:', {
            projectId: selectedProject.id,
            taskName: taskDescription,
            selectedProject: selectedProject
          });
          
          // 入力メソッド関連のエラーを回避するため、少し遅延を入れる
          await new Promise(resolve => setTimeout(resolve, 50));
          
          console.log('[Dashboard] invoke呼び出し前:', {
            command: 'create_task',
            args: {
              project_id: selectedProject.id,
              name: taskDescription
            }
          });
          
          let newTask;
          try {
            newTask = await invoke<any>('create_task', {
              request: {
                project_id: selectedProject.id,
                name: taskDescription
              }
            })
            
            console.log('[Dashboard] invoke呼び出し成功:', newTask);
          } catch (invokeError) {
            console.error('[Dashboard] invoke呼び出しエラー:', {
              error: invokeError,
              message: invokeError instanceof Error ? invokeError.message : String(invokeError),
              name: invokeError instanceof Error ? invokeError.name : 'Unknown',
              stack: invokeError instanceof Error ? invokeError.stack : undefined
            });
            throw invokeError;
          }
          
          console.log('[Dashboard] タスク作成成功:', newTask);
          taskId = newTask.id
          
          Logger.apiSuccess('Dashboard', 'POST', 'create_task', { 
            taskId: taskId,
            taskName: taskDescription,
            projectId: selectedProject.id 
          });
          
          // タスク作成後の確認ログ
          Logger.debug('Dashboard', 'Task created successfully', { 
            taskId: taskId,
            taskName: taskDescription,
            projectId: selectedProject.id,
            newTask: newTask
          });
        } catch (createError) {
          // 入力メソッド関連のエラーの場合は特別な処理
          const errorMessage = createError instanceof Error ? createError.message : String(createError);
          const isInputMethodError = errorMessage.includes('mach port') || errorMessage.includes('IMKCF');
          
          Logger.error('Dashboard', 'Task creation failed', { 
            error: errorMessage,
            isInputMethodError,
            projectId: selectedProject.id,
            taskName: taskDescription
          });
          
          console.error('[Dashboard] タスク作成エラー詳細:', {
            error: createError,
            message: errorMessage,
            name: createError instanceof Error ? createError.name : 'Unknown',
            stack: createError instanceof Error ? createError.stack : undefined,
            isInputMethodError,
            projectId: selectedProject.id,
            taskName: taskDescription,
            selectedProject: selectedProject,
            errorType: createError instanceof Error ? createError.constructor.name : 'Unknown',
            errorKeys: createError instanceof Error ? Object.keys(createError) : [],
            errorToString: createError instanceof Error ? createError.toString() : String(createError),
            errorValueOf: createError instanceof Error && createError.valueOf ? createError.valueOf() : 'N/A'
          });
          
          // 入力メソッド関連のエラーの場合は再試行を提案
          if (isInputMethodError) {
            console.warn('[Dashboard] 入力メソッド関連のエラーが発生しました。再試行してください。');
            alert('システムの一時的な問題が発生しました。もう一度お試しください。')
          } else {
            alert(`タスクの作成に失敗しました: ${errorMessage}`)
          }
          return
        }
      } else {
        // タスク入力がない場合は既存のタスクを探す
        try {
          const existingTasks = await invoke<any[]>('get_active_tasks_by_project', { 
            projectId: selectedProject.id 
          })
          
          if (existingTasks && existingTasks.length > 0) {
            // 最初のアクティブなタスクを使用
            taskId = existingTasks[0].id
            Logger.debug('Dashboard', 'Using existing task', { 
              taskId: taskId,
              taskName: existingTasks[0].name,
              projectId: selectedProject.id 
            });
          } else {
            // アクティブなタスクがない場合はデフォルトタスクを作成
            const defaultTask = await invoke<any>('create_task', {
              request: {
                project_id: selectedProject.id,
                name: `${selectedProject.name} - 作業`
              }
            })
            taskId = defaultTask.id
            Logger.debug('Dashboard', 'Created default task', { 
              taskId: taskId,
              taskName: defaultTask.name,
              projectId: selectedProject.id 
            });
          }
        } catch (taskError) {
          Logger.error('Dashboard', 'Failed to get or create default task', { 
            error: taskError instanceof Error ? taskError.message : String(taskError),
            projectId: selectedProject.id
          });
          
          console.error('Task get/create error:', {
            error: taskError,
            message: taskError instanceof Error ? taskError.message : String(taskError),
            projectId: selectedProject.id
          });
          
          alert(`タスクの取得・作成に失敗しました: ${taskError instanceof Error ? taskError.message : String(taskError)}`)
          return
        }
      }

      // タイマー開始前の確認ログ
      Logger.debug('Dashboard', 'About to start timer', { 
        taskId: taskId,
        projectId: selectedProject.id,
        taskDescription: taskDescription
      });
      
      // タイマーを開始
      Logger.apiCall('Dashboard', 'POST', 'start_timer', { taskId: taskId });
      
      await invoke('start_timer', {
        request: { task_id: taskId }
      })
      
      Logger.apiSuccess('Dashboard', 'POST', 'start_timer', { taskId: taskId });
      Logger.userAction('Dashboard', 'timer_started', { 
        taskId: taskId,
        projectId: selectedProject.id,
        projectName: selectedProject.name 
      });
      
      // データを再読み込み（エラーが発生しても処理を継続）
      try {
        await loadData()
      } catch (loadError) {
        Logger.warn('Dashboard', 'Failed to reload data after timer start', { 
          error: loadError instanceof Error ? loadError.message : String(loadError) 
        });
        // エラーが発生しても処理を継続
      }
      
      setTaskDescription('') // 入力欄をクリア
    } catch (error) {
      Logger.apiError('Dashboard', 'POST', 'start_timer', error, { 
        projectId: selectedProject.id,
        taskDescription: taskDescription 
      });
      alert(`タイマーの開始に失敗しました: ${error}`)
    }
  }

  const handleProjectEdit = (project: Project) => {
    console.log('Dashboard handleProjectEdit called', { project });
    setEditingProject(project)
    setShowEditModal(true)
  }

  const handleProjectSave = async (updatedProject: Project) => {
    try {
      // プロジェクトを更新
      await invoke('update_project', { request: updatedProject })
      
      // プロジェクトリストを更新
      setProjects(prevProjects => 
        prevProjects.map(p => p.id === updatedProject.id ? updatedProject : p)
      )
      
      // 選択中のプロジェクトも更新
      if (selectedProject?.id === updatedProject.id) {
        setSelectedProject(updatedProject)
      }
      
      setShowEditModal(false)
      setEditingProject(null)
    } catch (error) {
      console.error('プロジェクト更新エラー:', error)
      alert(`プロジェクトの更新に失敗しました: ${error}`)
    }
  }

  const handleProjectEditCancel = () => {
    setShowEditModal(false)
    setEditingProject(null)
  }

  return (
    <div className="dashboard" data-testid="dashboard">
      {/* selectedProject条件分岐を削除し、常にメイン画面を表示 */}
      <div className="dashboard-main">
        <TimerSection
          taskDescription={taskDescription}
          onTaskDescriptionChange={setTaskDescription}
          selectedProject={selectedProject}
          onProjectChange={setSelectedProject}
          projects={projects}
          tasks={tasks}
          onStartTimer={handleStartTimer}
          isRunning={!!timerStatus?.is_running}
          onProjectEdit={handleProjectEdit}
          onManualEntrySuccess={loadData}
        />
        <TimeEntriesList 
          entries={timeEntries}
          projects={projects}
          tasks={tasks}
          onRefresh={loadData} // データ再読み込み用のコールバックを追加
        />
      </div>
      
      <ProjectSidebar
        projects={projects}
        timeEntries={timeEntries}
        selectedProject={selectedProject}
        onProjectSelect={(project) => {
          console.log('[Dashboard] Project selected:', project)
          setSelectedProject(project)
        }}
        onRefresh={loadData}
        onProjectEdit={handleProjectEdit}
      />
      
      {showEditModal && editingProject && (
        <ProjectEditModal
          project={editingProject}
          isOpen={showEditModal}
          onSave={handleProjectSave}
          onCancel={handleProjectEditCancel}
          onClose={handleProjectEditCancel}
          projects={projects}
        />
      )}
    </div>
  )
}

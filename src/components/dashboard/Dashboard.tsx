import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { TimerSection } from './TimerSection'
import { TimeEntriesList } from './TimeEntriesList'
import { ProjectSidebar } from './ProjectSidebar'
import { Project, TimeEntry, TimerStatus } from '@/types'
import { Logger } from '@/utils'

export function Dashboard() {
  const [projects, setProjects] = useState<Project[]>([])
  const [timeEntries, setTimeEntries] = useState<TimeEntry[]>([])
  const [timerStatus, setTimerStatus] = useState<TimerStatus | null>(null)
  const [selectedProject, setSelectedProject] = useState<Project | null>(null)
  const [taskDescription, setTaskDescription] = useState('')

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

      // 時間エントリを取得
      try {
        Logger.debug('Dashboard', 'Calling get_time_entries');
        entriesData = await invoke<TimeEntry[]>('get_time_entries', { start_date: new Date().toISOString().split('T')[0] }) || [];
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
      
      setProjects(projectsData || [])
      setTimeEntries(entriesData || [])
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
    Logger.userAction('Dashboard', 'timer_start_initiated', { 
      selectedProject: selectedProject?.id,
      taskDescription: taskDescription 
    });
    
    if (!selectedProject) {
      Logger.warn('Dashboard', 'Timer start failed: no project selected');
      alert('プロジェクトを選択してください')
      return
    }

    try {
      // まず、プロジェクト配下のタスクを取得または作成
      let taskId: number
      
      try {
        // プロジェクトのアクティブなタスクを取得
        Logger.apiCall('Dashboard', 'GET', 'get_active_tasks_by_project', { 
          projectId: selectedProject.id 
        });
        
        const tasks = await invoke<any[]>('get_active_tasks_by_project', { 
          project_id: selectedProject.id 
        })
        
        if (tasks.length > 0) {
          taskId = tasks[0].id
          Logger.debug('Dashboard', 'Using existing task', { taskId: taskId });
        } else {
          // タスクが存在しない場合は作成
          Logger.apiCall('Dashboard', 'POST', 'create_task', { 
            projectId: selectedProject.id,
            taskName: taskDescription || 'Default Task' 
          });
          
          const newTask = await invoke<any>('create_task', {
            project_id: selectedProject.id,
            name: taskDescription || 'Default Task'
          })
          taskId = newTask.id
          
          Logger.apiSuccess('Dashboard', 'POST', 'create_task', { taskId: taskId });
        }
      } catch (taskError) {
        Logger.error('Dashboard', 'Failed to get or create task', { 
          error: taskError instanceof Error ? taskError.message : String(taskError),
          projectId: selectedProject.id 
        });
        // フォールバック：プロジェクトIDをタスクIDとして使用
        taskId = selectedProject.id
      }

      // タイマーを開始
      Logger.apiCall('Dashboard', 'POST', 'start_timer', { taskId: taskId });
      
      await invoke('start_timer', {
        task_id: taskId
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

  return (
    <div className="dashboard">
      <div className="dashboard-main">
        <TimerSection
          taskDescription={taskDescription}
          onTaskDescriptionChange={setTaskDescription}
          selectedProject={selectedProject}
          onProjectChange={setSelectedProject}
          projects={projects}
          onStartTimer={handleStartTimer}
          isRunning={timerStatus?.is_running || false}
        />
        
        <TimeEntriesList 
          entries={timeEntries}
          projects={projects}
        />
      </div>
      
      <ProjectSidebar 
        projects={projects}
        timeEntries={timeEntries}
        selectedProject={selectedProject}
        onProjectSelect={setSelectedProject}
        onRefresh={loadData}
      />
    </div>
  )
}

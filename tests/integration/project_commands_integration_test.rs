use time_tracker_go::application::dto::CreateProjectRequest;
use time_tracker_go::application::services::ApplicationService;
use time_tracker_go::domain::repositories::tests::InMemoryProjectRepository;
use time_tracker_go::domain::repositories::task_tests::InMemoryTaskRepository;
use time_tracker_go::domain::services::ProjectManagementServiceImpl;
use time_tracker_go::presentation::commands::project_commands::{
    create_project, get_all_active_projects, get_all_projects
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn プロジェクト作成から一覧取得までの統合テスト() {
    // テスト用のリポジトリとサービスを設定
    let project_repo = Arc::new(InMemoryProjectRepository::new());
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let service = ProjectManagementServiceImpl::new(project_repo.clone(), task_repo);
    let app_service = ApplicationService::new_for_test(project_repo, service);
    let app_service = Arc::new(Mutex::new(app_service));

    // 1. プロジェクトを作成
    let create_request = CreateProjectRequest {
        name: "統合テストプロジェクト".to_string(),
    };
    let create_result = create_project(app_service.clone(), create_request).await;
    assert!(create_result.is_ok());

    let created_project = create_result.unwrap();
    assert_eq!(created_project.name, "統合テストプロジェクト");
    assert_eq!(created_project.status, "active");

    // 2. アクティブプロジェクト一覧を取得
    let active_projects_result = get_all_active_projects(app_service.clone()).await;
    assert!(active_projects_result.is_ok());

    let active_projects = active_projects_result.unwrap();
    assert_eq!(active_projects.len(), 1);
    assert_eq!(active_projects[0].name, "統合テストプロジェクト");
    assert_eq!(active_projects[0].id, created_project.id);

    // 3. 全プロジェクト一覧を取得
    let all_projects_result = get_all_projects(app_service).await;
    assert!(all_projects_result.is_ok());

    let all_projects = all_projects_result.unwrap();
    assert_eq!(all_projects.len(), 1);
    assert_eq!(all_projects[0].name, "統合テストプロジェクト");
    assert_eq!(all_projects[0].id, created_project.id);
}

#[tokio::test]
async fn 複数プロジェクトの作成と一覧取得の統合テスト() {
    // テスト用のリポジトリとサービスを設定
    let project_repo = Arc::new(InMemoryProjectRepository::new());
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let service = ProjectManagementServiceImpl::new(project_repo.clone(), task_repo);
    let app_service = ApplicationService::new_for_test(project_repo, service);
    let app_service = Arc::new(Mutex::new(app_service));

    // 複数のプロジェクトを作成
    let project_names = vec![
        "プロジェクトA".to_string(),
        "プロジェクトB".to_string(),
        "プロジェクトC".to_string(),
    ];

    let mut created_project_ids = Vec::new();

    for project_name in &project_names {
        let create_request = CreateProjectRequest {
            name: project_name.clone(),
        };
        let create_result = create_project(app_service.clone(), create_request).await;
        assert!(create_result.is_ok());

        let created_project = create_result.unwrap();
        created_project_ids.push(created_project.id);
        assert_eq!(created_project.name, *project_name);
        assert_eq!(created_project.status, "active");
    }

    // アクティブプロジェクト一覧を取得
    let active_projects_result = get_all_active_projects(app_service.clone()).await;
    assert!(active_projects_result.is_ok());

    let active_projects = active_projects_result.unwrap();
    assert_eq!(active_projects.len(), 3);

    // 作成されたプロジェクトが全て含まれていることを確認
    for project_name in &project_names {
        let found_project = active_projects.iter().find(|p| p.name == *project_name);
        assert!(found_project.is_some());
        assert_eq!(found_project.unwrap().status, "active");
    }

    // 全プロジェクト一覧を取得
    let all_projects_result = get_all_projects(app_service).await;
    assert!(all_projects_result.is_ok());

    let all_projects = all_projects_result.unwrap();
    assert_eq!(all_projects.len(), 3);

    // 作成されたプロジェクトが全て含まれていることを確認
    for project_name in &project_names {
        let found_project = all_projects.iter().find(|p| p.name == *project_name);
        assert!(found_project.is_some());
    }
}

#[tokio::test]
async fn 重複名プロジェクト作成の統合テスト() {
    // テスト用のリポジトリとサービスを設定
    let project_repo = Arc::new(InMemoryProjectRepository::new());
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let service = ProjectManagementServiceImpl::new(project_repo.clone(), task_repo);
    let app_service = ApplicationService::new_for_test(project_repo, service);
    let app_service = Arc::new(Mutex::new(app_service));

    // 最初のプロジェクトを作成
    let create_request1 = CreateProjectRequest {
        name: "重複テストプロジェクト".to_string(),
    };
    let create_result1 = create_project(app_service.clone(), create_request1).await;
    assert!(create_result1.is_ok());

    // 同じ名前のプロジェクトを作成しようとする
    let create_request2 = CreateProjectRequest {
        name: "重複テストプロジェクト".to_string(),
    };
    let create_result2 = create_project(app_service.clone(), create_request2).await;
    assert!(create_result2.is_err());

    // アクティブプロジェクト一覧を取得
    let active_projects_result = get_all_active_projects(app_service).await;
    assert!(active_projects_result.is_ok());

    let active_projects = active_projects_result.unwrap();
    // 重複作成が失敗したため、1つのプロジェクトのみ存在する
    assert_eq!(active_projects.len(), 1);
    assert_eq!(active_projects[0].name, "重複テストプロジェクト");
}

#[tokio::test]
async fn 空のプロジェクト名での作成エラーの統合テスト() {
    // テスト用のリポジトリとサービスを設定
    let project_repo = Arc::new(InMemoryProjectRepository::new());
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let service = ProjectManagementServiceImpl::new(project_repo.clone(), task_repo);
    let app_service = ApplicationService::new_for_test(project_repo, service);
    let app_service = Arc::new(Mutex::new(app_service));

    // 空のプロジェクト名で作成
    let create_request = CreateProjectRequest {
        name: "".to_string(),
    };
    let create_result = create_project(app_service.clone(), create_request).await;
    assert!(create_result.is_err());

    // アクティブプロジェクト一覧を取得
    let active_projects_result = get_all_active_projects(app_service).await;
    assert!(active_projects_result.is_ok());

    let active_projects = active_projects_result.unwrap();
    // 作成が失敗したため、プロジェクトは存在しない
    assert_eq!(active_projects.len(), 0);
}

#[tokio::test]
async fn プロジェクト作成後のデータ整合性テスト() {
    // テスト用のリポジトリとサービスを設定
    let project_repo = Arc::new(InMemoryProjectRepository::new());
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let service = ProjectManagementServiceImpl::new(project_repo.clone(), task_repo);
    let app_service = ApplicationService::new_for_test(project_repo, service);
    let app_service = Arc::new(Mutex::new(app_service));

    // プロジェクトを作成
    let create_request = CreateProjectRequest {
        name: "データ整合性テストプロジェクト".to_string(),
    };
    let create_result = create_project(app_service.clone(), create_request).await;
    assert!(create_result.is_ok());

    let created_project = create_result.unwrap();
    let project_id = created_project.id;

    // アクティブプロジェクト一覧を取得
    let active_projects_result = get_all_active_projects(app_service.clone()).await;
    assert!(active_projects_result.is_ok());

    let active_projects = active_projects_result.unwrap();
    let found_project = active_projects.iter().find(|p| p.id == project_id);
    assert!(found_project.is_some());

    let found_project = found_project.unwrap();
    assert_eq!(found_project.name, "データ整合性テストプロジェクト");
    assert_eq!(found_project.status, "active");
    assert_eq!(found_project.id, project_id);
    assert_eq!(found_project.effective_at, created_project.effective_at);

    // 全プロジェクト一覧を取得
    let all_projects_result = get_all_projects(app_service).await;
    assert!(all_projects_result.is_ok());

    let all_projects = all_projects_result.unwrap();
    let found_project = all_projects.iter().find(|p| p.id == project_id);
    assert!(found_project.is_some());

    let found_project = found_project.unwrap();
    assert_eq!(found_project.name, "データ整合性テストプロジェクト");
    assert_eq!(found_project.status, "active");
    assert_eq!(found_project.id, project_id);
    assert_eq!(found_project.effective_at, created_project.effective_at);
}

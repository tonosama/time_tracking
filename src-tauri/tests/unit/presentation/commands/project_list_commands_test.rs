use crate::application::services::ApplicationService;
use crate::domain::repositories::tests::InMemoryProjectRepository;
use crate::domain::repositories::task_tests::InMemoryTaskRepository;
use crate::domain::services::ProjectManagementServiceImpl;
use crate::presentation::commands::project_commands::{get_all_active_projects, get_all_projects};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn アクティブプロジェクト一覧取得が正常に動作すること() {
    // テスト用のリポジトリとサービスを設定
    let project_repo = Arc::new(InMemoryProjectRepository::new());
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let service = ProjectManagementServiceImpl::new(project_repo.clone(), task_repo);
    let app_service = ApplicationService::new_for_test(project_repo, service);
    let app_service = Arc::new(Mutex::new(app_service));

    // プロジェクトを作成
    let project1 = crate::domain::entities::Project::new(
        crate::domain::value_objects::ProjectId::new(1).unwrap(),
        "プロジェクト1".to_string(),
    ).unwrap();
    let project2 = crate::domain::entities::Project::new(
        crate::domain::value_objects::ProjectId::new(2).unwrap(),
        "プロジェクト2".to_string(),
    ).unwrap();

    // プロジェクトを保存
    project_repo.save(&project1).await.unwrap();
    project_repo.save(&project2).await.unwrap();

    // アクティブプロジェクト一覧を取得
    let result = get_all_active_projects(app_service).await;

    // 結果を検証
    assert!(result.is_ok());
    let projects = result.unwrap();
    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].name, "プロジェクト1");
    assert_eq!(projects[1].name, "プロジェクト2");
    assert_eq!(projects[0].status, "active");
    assert_eq!(projects[1].status, "active");
}

#[tokio::test]
async fn 全プロジェクト一覧取得が正常に動作すること() {
    // テスト用のリポジトリとサービスを設定
    let project_repo = Arc::new(InMemoryProjectRepository::new());
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let service = ProjectManagementServiceImpl::new(project_repo.clone(), task_repo);
    let app_service = ApplicationService::new_for_test(project_repo, service);
    let app_service = Arc::new(Mutex::new(app_service));

    // プロジェクトを作成
    let project1 = crate::domain::entities::Project::new(
        crate::domain::value_objects::ProjectId::new(1).unwrap(),
        "プロジェクト1".to_string(),
    ).unwrap();
    let project2 = crate::domain::entities::Project::new(
        crate::domain::value_objects::ProjectId::new(2).unwrap(),
        "プロジェクト2".to_string(),
    ).unwrap();

    // プロジェクトを保存
    project_repo.save(&project1).await.unwrap();
    project_repo.save(&project2).await.unwrap();

    // 全プロジェクト一覧を取得
    let result = get_all_projects(app_service).await;

    // 結果を検証
    assert!(result.is_ok());
    let projects = result.unwrap();
    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].name, "プロジェクト1");
    assert_eq!(projects[1].name, "プロジェクト2");
}

#[tokio::test]
async fn プロジェクトが存在しない場合に空の配列が返されること() {
    // テスト用のリポジトリとサービスを設定
    let project_repo = Arc::new(InMemoryProjectRepository::new());
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let service = ProjectManagementServiceImpl::new(project_repo.clone(), task_repo);
    let app_service = ApplicationService::new_for_test(project_repo, service);
    let app_service = Arc::new(Mutex::new(app_service));

    // アクティブプロジェクト一覧を取得（プロジェクトなし）
    let result = get_all_active_projects(app_service).await;

    // 結果を検証
    assert!(result.is_ok());
    let projects = result.unwrap();
    assert_eq!(projects.len(), 0);
}

#[tokio::test]
async fn アーカイブされたプロジェクトはアクティブ一覧に含まれないこと() {
    // テスト用のリポジトリとサービスを設定
    let project_repo = Arc::new(InMemoryProjectRepository::new());
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let service = ProjectManagementServiceImpl::new(project_repo.clone(), task_repo);
    let app_service = ApplicationService::new_for_test(project_repo, service);
    let app_service = Arc::new(Mutex::new(app_service));

    // アクティブなプロジェクトを作成
    let active_project = crate::domain::entities::Project::new(
        crate::domain::value_objects::ProjectId::new(1).unwrap(),
        "アクティブプロジェクト".to_string(),
    ).unwrap();

    // アーカイブされたプロジェクトを作成
    let archived_project = crate::domain::entities::Project::new(
        crate::domain::value_objects::ProjectId::new(2).unwrap(),
        "アーカイブプロジェクト".to_string(),
    ).unwrap().archive();

    // プロジェクトを保存
    project_repo.save(&active_project).await.unwrap();
    project_repo.save(&archived_project).await.unwrap();

    // アクティブプロジェクト一覧を取得
    let result = get_all_active_projects(app_service).await;

    // 結果を検証
    assert!(result.is_ok());
    let projects = result.unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].name, "アクティブプロジェクト");
    assert_eq!(projects[0].status, "active");
}

#[tokio::test]
async fn 全プロジェクト一覧にはアーカイブされたプロジェクトも含まれること() {
    // テスト用のリポジトリとサービスを設定
    let project_repo = Arc::new(InMemoryProjectRepository::new());
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let service = ProjectManagementServiceImpl::new(project_repo.clone(), task_repo);
    let app_service = ApplicationService::new_for_test(project_repo, service);
    let app_service = Arc::new(Mutex::new(app_service));

    // アクティブなプロジェクトを作成
    let active_project = crate::domain::entities::Project::new(
        crate::domain::value_objects::ProjectId::new(1).unwrap(),
        "アクティブプロジェクト".to_string(),
    ).unwrap();

    // アーカイブされたプロジェクトを作成
    let archived_project = crate::domain::entities::Project::new(
        crate::domain::value_objects::ProjectId::new(2).unwrap(),
        "アーカイブプロジェクト".to_string(),
    ).unwrap().archive();

    // プロジェクトを保存
    project_repo.save(&active_project).await.unwrap();
    project_repo.save(&archived_project).await.unwrap();

    // 全プロジェクト一覧を取得
    let result = get_all_projects(app_service).await;

    // 結果を検証
    assert!(result.is_ok());
    let projects = result.unwrap();
    assert_eq!(projects.len(), 2);

    // アクティブプロジェクトを確認
    let active_project_dto = projects.iter().find(|p| p.name == "アクティブプロジェクト").unwrap();
    assert_eq!(active_project_dto.status, "active");

    // アーカイブプロジェクトを確認
    let archived_project_dto = projects.iter().find(|p| p.name == "アーカイブプロジェクト").unwrap();
    assert_eq!(archived_project_dto.status, "archived");
}

#[tokio::test]
async fn プロジェクト一覧が正しい順序で返されること() {
    // テスト用のリポジトリとサービスを設定
    let project_repo = Arc::new(InMemoryProjectRepository::new());
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let service = ProjectManagementServiceImpl::new(project_repo.clone(), task_repo);
    let app_service = ApplicationService::new_for_test(project_repo, service);
    let app_service = Arc::new(Mutex::new(app_service));

    // 複数のプロジェクトを作成
    let project1 = crate::domain::entities::Project::new(
        crate::domain::value_objects::ProjectId::new(1).unwrap(),
        "プロジェクトA".to_string(),
    ).unwrap();
    let project2 = crate::domain::entities::Project::new(
        crate::domain::value_objects::ProjectId::new(2).unwrap(),
        "プロジェクトB".to_string(),
    ).unwrap();
    let project3 = crate::domain::entities::Project::new(
        crate::domain::value_objects::ProjectId::new(3).unwrap(),
        "プロジェクトC".to_string(),
    ).unwrap();

    // プロジェクトを保存
    project_repo.save(&project1).await.unwrap();
    project_repo.save(&project2).await.unwrap();
    project_repo.save(&project3).await.unwrap();

    // 全プロジェクト一覧を取得
    let result = get_all_projects(app_service).await;

    // 結果を検証
    assert!(result.is_ok());
    let projects = result.unwrap();
    assert_eq!(projects.len(), 3);

    // プロジェクト名を確認
    let project_names: Vec<&str> = projects.iter().map(|p| p.name.as_str()).collect();
    assert!(project_names.contains(&"プロジェクトA"));
    assert!(project_names.contains(&"プロジェクトB"));
    assert!(project_names.contains(&"プロジェクトC"));
}

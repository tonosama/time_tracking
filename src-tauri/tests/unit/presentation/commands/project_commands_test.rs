use crate::application::dto::{CreateProjectRequest, ProjectDto};
use crate::application::services::ApplicationService;
use crate::domain::entities::Project;
use crate::domain::repositories::tests::InMemoryProjectRepository;
use crate::domain::repositories::task_tests::InMemoryTaskRepository;
use crate::domain::services::ProjectManagementServiceImpl;
use crate::domain::value_objects::ProjectId;
use crate::presentation::commands::project_commands::create_project;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn プロジェクト作成コマンドが正常に動作すること() {
    // テスト用のリポジトリとサービスを設定
    let project_repo = Arc::new(InMemoryProjectRepository::new());
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let service = ProjectManagementServiceImpl::new(project_repo.clone(), task_repo);
    let app_service = ApplicationService::new_for_test(project_repo, service);
    let app_service = Arc::new(Mutex::new(app_service));

    // プロジェクト作成リクエスト
    let request = CreateProjectRequest {
        name: "テストプロジェクト".to_string(),
    };

    // コマンドを実行
    let result = create_project(app_service, request).await;

    // 結果を検証
    assert!(result.is_ok());
    let project_dto = result.unwrap();
    assert_eq!(project_dto.name, "テストプロジェクト");
    assert_eq!(project_dto.status, "active");
    assert!(project_dto.id > 0);
}

#[tokio::test]
async fn 重複名のプロジェクト作成が失敗すること() {
    // テスト用のリポジトリとサービスを設定
    let project_repo = Arc::new(InMemoryProjectRepository::new());
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let service = ProjectManagementServiceImpl::new(project_repo.clone(), task_repo);
    let app_service = ApplicationService::new_for_test(project_repo, service);
    let app_service = Arc::new(Mutex::new(app_service));

    // 最初のプロジェクトを作成
    let request1 = CreateProjectRequest {
        name: "テストプロジェクト".to_string(),
    };
    let result1 = create_project(app_service.clone(), request1).await;
    assert!(result1.is_ok());

    // 同じ名前のプロジェクトを作成しようとする
    let request2 = CreateProjectRequest {
        name: "テストプロジェクト".to_string(),
    };
    let result2 = create_project(app_service, request2).await;

    // エラーが発生することを確認
    assert!(result2.is_err());
    let error_message = result2.unwrap_err();
    assert!(error_message.contains("already exists"));
}

#[tokio::test]
async fn 空のプロジェクト名で作成が失敗すること() {
    // テスト用のリポジトリとサービスを設定
    let project_repo = Arc::new(InMemoryProjectRepository::new());
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let service = ProjectManagementServiceImpl::new(project_repo.clone(), task_repo);
    let app_service = ApplicationService::new_for_test(project_repo, service);
    let app_service = Arc::new(Mutex::new(app_service));

    // 空のプロジェクト名で作成
    let request = CreateProjectRequest {
        name: "".to_string(),
    };
    let result = create_project(app_service, request).await;

    // エラーが発生することを確認
    assert!(result.is_err());
}

#[tokio::test]
async fn プロジェクト作成後にDTOが正しく変換されること() {
    // テスト用のリポジトリとサービスを設定
    let project_repo = Arc::new(InMemoryProjectRepository::new());
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let service = ProjectManagementServiceImpl::new(project_repo.clone(), task_repo);
    let app_service = ApplicationService::new_for_test(project_repo, service);
    let app_service = Arc::new(Mutex::new(app_service));

    // プロジェクト作成リクエスト
    let request = CreateProjectRequest {
        name: "テストプロジェクト".to_string(),
    };

    // コマンドを実行
    let result = create_project(app_service, request).await;
    assert!(result.is_ok());

    let project_dto = result.unwrap();

    // DTOの内容を検証
    assert_eq!(project_dto.name, "テストプロジェクト");
    assert_eq!(project_dto.status, "active");
    assert!(project_dto.id > 0);
    assert!(!project_dto.effective_at.is_empty());

    // DTOからドメインエンティティに変換できることを確認
    let domain_project = project_dto.to_domain();
    assert!(domain_project.is_ok());
    let domain_project = domain_project.unwrap();
    assert_eq!(domain_project.name(), "テストプロジェクト");
    assert!(domain_project.is_active());
}

#[tokio::test]
async fn 複数のプロジェクトが連続して作成できること() {
    // テスト用のリポジトリとサービスを設定
    let project_repo = Arc::new(InMemoryProjectRepository::new());
    let task_repo = Arc::new(InMemoryTaskRepository::new());
    let service = ProjectManagementServiceImpl::new(project_repo.clone(), task_repo);
    let app_service = ApplicationService::new_for_test(project_repo, service);
    let app_service = Arc::new(Mutex::new(app_service));

    // 複数のプロジェクトを作成
    let projects = vec![
        "プロジェクト1".to_string(),
        "プロジェクト2".to_string(),
        "プロジェクト3".to_string(),
    ];

    let mut created_projects = Vec::new();

    for project_name in projects {
        let request = CreateProjectRequest {
            name: project_name.clone(),
        };
        let result = create_project(app_service.clone(), request).await;
        assert!(result.is_ok());
        created_projects.push(result.unwrap());
    }

    // 作成されたプロジェクトの数を確認
    assert_eq!(created_projects.len(), 3);

    // 各プロジェクトの名前を確認
    assert_eq!(created_projects[0].name, "プロジェクト1");
    assert_eq!(created_projects[1].name, "プロジェクト2");
    assert_eq!(created_projects[2].name, "プロジェクト3");

    // IDが異なることを確認
    assert_ne!(created_projects[0].id, created_projects[1].id);
    assert_ne!(created_projects[1].id, created_projects[2].id);
    assert_ne!(created_projects[0].id, created_projects[2].id);
}

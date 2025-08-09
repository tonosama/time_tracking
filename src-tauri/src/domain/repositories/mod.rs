// リポジトリトレイト - データ永続化の抽象化インターフェース
// 具体的な実装はインフラ層で行う

pub mod project_repository;
pub mod task_repository;

pub use project_repository::{ProjectRepository};
pub use task_repository::{TaskRepository};

#[cfg(test)]
pub use project_repository::tests;
#[cfg(test)]
pub use task_repository::tests as task_tests;


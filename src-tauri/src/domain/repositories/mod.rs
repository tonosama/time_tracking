// リポジトリトレイト - データ永続化の抽象化インターフェース
// 具体的な実装はインフラ層で行う

pub mod project_repository;
pub mod task_repository;
pub mod time_entry_repository;

pub use project_repository::{ProjectRepository};
pub use task_repository::{TaskRepository};
pub use time_entry_repository::{TimeEntryRepository};

#[cfg(test)]
pub use project_repository::tests;
#[cfg(test)]
pub use task_repository::tests as task_tests;
#[cfg(test)]
pub use time_entry_repository::tests as time_entry_tests;


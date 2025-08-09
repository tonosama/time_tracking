// ドメイン層 - ビジネスロジックとドメインルールを定義
// 外部依存は一切持たない純粋なビジネス層

pub mod entities;
pub mod value_objects;
pub mod repositories;
pub mod services;

// ドメイン全体で使用される型やトレイトの再エクスポート
pub use entities::*;
pub use value_objects::*;
pub use repositories::*;
pub use services::*;


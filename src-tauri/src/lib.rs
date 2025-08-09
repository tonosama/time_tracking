// Time Tracker Go - Rust Backend Library
// DDDアーキテクチャのエントリーポイント

pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;

// 外部クレートの共通インポート
pub use anyhow::{Result, Context};
pub use serde::{Deserialize, Serialize};

// アプリケーション全体で使用する型の再エクスポート
pub use domain::entities::*;
pub use domain::value_objects::*;


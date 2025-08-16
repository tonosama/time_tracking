# ログ仕様書

## 概要

Time Tracker Goアプリケーションにおけるログ出力の仕様を定義します。
本仕様は、開発、デバッグ、運用監視、トラブルシューティングを効率的に行うためのログ標準を確立することを目的とします。

## ログレベル定義

### 標準ログレベル

| レベル | 用途 | 出力タイミング | 例 |
|--------|------|---------------|-----|
| **ERROR** | エラー・例外 | システムエラー、処理失敗、例外発生時 | データベース接続失敗、API呼び出しエラー |
| **WARN** | 警告・潜在的問題 | 非致命的だが注意が必要な状況 | 非推奨API使用、リソース不足警告 |
| **INFO** | 重要なアプリケーションイベント | ユーザーアクション、状態変化 | ユーザーログイン、プロジェクト作成、タイマー開始 |
| **DEBUG** | 開発用詳細情報 | 開発・デバッグ時の詳細な処理情報 | 関数の入出力、中間処理結果 |
| **TRACE** | 最詳細レベル | 非常に詳細な実行トレース | 全ての関数呼び出し、変数の変化 |

### 環境別推奨レベル

- **開発環境**: `DEBUG` または `TRACE`
- **テスト環境**: `INFO`
- **本番環境**: `WARN`

## ログ構造仕様

### 統一ログエントリ形式

```typescript
interface LogEntry {
  timestamp: string      // ISO 8601形式 (YYYY-MM-DDTHH:mm:ss.sssZ)
  level: LogLevel        // ログレベル
  component: string      // コンポーネント/モジュール名
  message: string        // 人間が読みやすいメッセージ
  context?: object       // 構造化されたコンテキスト情報
  userId?: string        // ユーザー識別子（該当する場合）
  sessionId?: string     // セッション識別子
  traceId?: string       // 分散トレーシング用ID
  error?: ErrorInfo      // エラー情報（ERROR/WARNレベル時）
}

interface ErrorInfo {
  name: string           // エラー名/タイプ
  message: string        // エラーメッセージ
  stack?: string         // スタックトレース（開発環境のみ）
  code?: string          // エラーコード
}
```

### ログメッセージフォーマット

#### 基本形式
```
[TIMESTAMP] [LEVEL] [COMPONENT] MESSAGE [CONTEXT]
```

#### 実例
```
[2025-01-15T13:30:45.123Z] [INFO] [ProjectService] Project created successfully {"projectId": "proj_123", "name": "New Project", "userId": "user_456"}
[2025-01-15T13:30:46.124Z] [ERROR] [DatabaseService] Failed to save project {"error": "Connection timeout", "projectId": "proj_123"}
```

## バックエンド（Rust）ログ実装

### 1. ライブラリ設定

```toml
# Cargo.toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-appender = "0.2"
serde_json = "1.0"
```

### 2. 初期化コード

```rust
// main.rs
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // ログ設定初期化
    init_logging();
    
    tracing::info!("Starting Time Tracker application");
    // ...
}

fn init_logging() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "time_tracker_go=debug,info".into());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true))
        .init();
}
```

### 3. 使用例

```rust
// プロジェクト作成時
tracing::info!(
    project_id = %project.id(),
    project_name = %project.name(),
    user_id = ?user_id,
    "Project created successfully"
);

// エラー時
tracing::error!(
    error = %e,
    project_id = %project_id,
    "Failed to create project"
);

// デバッグ時
tracing::debug!(
    input = ?request,
    "Processing create project request"
);
```

### 4. 構造化ログマクロ

```rust
// utils/logging.rs
macro_rules! log_user_action {
    ($level:expr, $action:expr, $user_id:expr, $($field:tt)*) => {
        tracing::event!(
            $level,
            user_id = $user_id,
            action = $action,
            $($field)*
        );
    };
}

// 使用例
log_user_action!(
    tracing::Level::INFO,
    "project_created",
    user_id,
    project_id = %project.id(),
    project_name = %project.name()
);
```

## フロントエンド（TypeScript）ログ実装

### 1. ログユーティリティ

```typescript
// src/utils/logger.ts
export enum LogLevel {
  TRACE = 0,
  DEBUG = 1,
  INFO = 2,
  WARN = 3,
  ERROR = 4,
}

export interface LogContext {
  [key: string]: any;
}

export interface LoggerConfig {
  level: LogLevel;
  enableConsole: boolean;
  enableRemote: boolean;
  remoteEndpoint?: string;
}

export class Logger {
  private static config: LoggerConfig = {
    level: LogLevel.INFO,
    enableConsole: true,
    enableRemote: false,
  };

  private static sessionId = crypto.randomUUID();

  static configure(config: Partial<LoggerConfig>) {
    this.config = { ...this.config, ...config };
  }

  static trace(component: string, message: string, context?: LogContext) {
    this.log(LogLevel.TRACE, component, message, context);
  }

  static debug(component: string, message: string, context?: LogContext) {
    this.log(LogLevel.DEBUG, component, message, context);
  }

  static info(component: string, message: string, context?: LogContext) {
    this.log(LogLevel.INFO, component, message, context);
  }

  static warn(component: string, message: string, context?: LogContext) {
    this.log(LogLevel.WARN, component, message, context);
  }

  static error(component: string, message: string, context?: LogContext) {
    this.log(LogLevel.ERROR, component, message, context);
  }

  private static log(level: LogLevel, component: string, message: string, context?: LogContext) {
    if (level < this.config.level) return;

    const logEntry = {
      timestamp: new Date().toISOString(),
      level: LogLevel[level],
      component,
      message,
      context,
      sessionId: this.sessionId,
    };

    if (this.config.enableConsole) {
      this.logToConsole(level, logEntry);
    }

    if (this.config.enableRemote) {
      this.logToRemote(logEntry);
    }
  }

  private static logToConsole(level: LogLevel, entry: any) {
    const msg = `[${entry.timestamp}] [${entry.level}] [${entry.component}] ${entry.message}`;
    
    switch (level) {
      case LogLevel.ERROR:
        console.error(msg, entry.context);
        break;
      case LogLevel.WARN:
        console.warn(msg, entry.context);
        break;
      case LogLevel.INFO:
        console.info(msg, entry.context);
        break;
      case LogLevel.DEBUG:
      case LogLevel.TRACE:
        console.debug(msg, entry.context);
        break;
    }
  }

  private static async logToRemote(entry: any) {
    try {
      // リモートログサーバーへの送信（実装時に追加）
      // await fetch(this.config.remoteEndpoint, {
      //   method: 'POST',
      //   headers: { 'Content-Type': 'application/json' },
      //   body: JSON.stringify(entry)
      // });
    } catch (error) {
      console.error('Failed to send log to remote server:', error);
    }
  }
}
```

### 2. 使用例

```typescript
// コンポーネントでの使用
import { Logger } from '@/utils/logger';

// プロジェクト作成
const handleCreateProject = async (name: string) => {
  Logger.info('ProjectCreation', 'Starting project creation', { projectName: name });
  
  try {
    const result = await invoke('create_project', { name });
    Logger.info('ProjectCreation', 'Project created successfully', { 
      projectId: result.id, 
      projectName: result.name 
    });
  } catch (error) {
    Logger.error('ProjectCreation', 'Failed to create project', { 
      projectName: name,
      error: error.message 
    });
  }
};

// タイマー操作
const startTimer = async (taskId: number) => {
  Logger.debug('Timer', 'Starting timer', { taskId });
  
  try {
    await invoke('start_timer', { task_id: taskId });
    Logger.info('Timer', 'Timer started successfully', { taskId });
  } catch (error) {
    Logger.error('Timer', 'Failed to start timer', { taskId, error: error.message });
  }
};
```

### 3. 環境設定

```typescript
// src/config/logger.ts
import { Logger, LogLevel } from '@/utils/logger';

// 環境に応じたログレベル設定
const getLogLevel = (): LogLevel => {
  if (process.env.NODE_ENV === 'development') {
    return LogLevel.DEBUG;
  } else if (process.env.NODE_ENV === 'test') {
    return LogLevel.WARN;
  } else {
    return LogLevel.INFO;
  }
};

Logger.configure({
  level: getLogLevel(),
  enableConsole: true,
  enableRemote: process.env.NODE_ENV === 'production',
  remoteEndpoint: process.env.VITE_LOG_ENDPOINT,
});
```

## コンポーネント別ログ方針

### 1. ユーザーアクション

**対象**: ボタンクリック、フォーム送信、ナビゲーション

```typescript
// プロジェクト作成
Logger.info('UserAction', 'Project creation initiated', { userId, projectName });

// タイマー操作
Logger.info('UserAction', 'Timer started', { userId, taskId, projectId });
```

### 2. システムイベント

**対象**: API呼び出し、状態変化、データ同期

```rust
// Rust側
tracing::info!(
    event = "database_migration",
    version = %new_version,
    "Database migrated successfully"
);
```

### 3. エラーハンドリング

**対象**: 例外、API エラー、バリデーションエラー

```typescript
// フロントエンド
Logger.error('APICall', 'Failed to fetch projects', {
  endpoint: '/api/projects',
  statusCode: response.status,
  error: response.statusText
});
```

```rust
// バックエンド
tracing::error!(
    error = %e,
    endpoint = "/api/projects",
    "Database query failed"
);
```

### 4. パフォーマンス監視

**対象**: 処理時間、リソース使用量

```typescript
// 処理時間計測
const startTime = performance.now();
// ... 処理 ...
const duration = performance.now() - startTime;

Logger.debug('Performance', 'Project load completed', {
  duration_ms: duration,
  project_count: projects.length
});
```

## ログファイル管理

### 1. ローテーション設定

```rust
// ログローテーション設定
use tracing_appender::rolling::{RollingFileAppender, Rotation};

let file_appender = RollingFileAppender::new(
    Rotation::daily(),
    "logs",
    "time-tracker.log"
);
```

### 2. ファイル命名規則

```
logs/
├── time-tracker.2025-01-15.log
├── time-tracker.2025-01-16.log
└── time-tracker.log (current)
```

### 3. 保持期間

- **開発環境**: 7日間
- **本番環境**: 30日間

## セキュリティ考慮事項

### 1. 機密情報の除外

**❌ ログに含めてはいけない情報**:
- パスワード
- API キー
- セッショントークン
- 個人識別情報（PII）

### 2. データマスキング

```typescript
// 機密情報のマスキング
const maskSensitiveData = (data: any) => {
  return {
    ...data,
    password: '***',
    token: data.token ? `${data.token.slice(0, 6)}...` : undefined,
  };
};

Logger.info('Authentication', 'User logged in', maskSensitiveData(userInfo));
```

## 監視・アラート設定

### 1. エラー監視

- **ERROR レベル**: 即座にアラート
- **WARN レベル**: 5分以内に10件以上で注意
- **異常なERROR率**: 1%以上でアラート

### 2. パフォーマンス監視

- **API レスポンス時間**: 5秒以上で警告
- **データベースクエリ時間**: 3秒以上で警告

### 3. ログボリューム監視

- **急激なログ増加**: 通常の3倍以上で調査
- **ログ停止**: 5分間ログなしで緊急アラート

## 実装チェックリスト

### バックエンド（Rust）

- [ ] tracing-subscriber 初期化
- [ ] 環境変数によるログレベル制御
- [ ] 構造化ログ出力
- [ ] ファイルローテーション
- [ ] エラーコンテキスト情報

### フロントエンド（TypeScript）

- [ ] Logger ユーティリティ実装
- [ ] 環境別ログレベル設定
- [ ] ユーザーアクションログ
- [ ] エラーバウンダリとの連携
- [ ] パフォーマンス計測ログ

### 運用

- [ ] ログ監視設定
- [ ] アラート設定
- [ ] ログ保持ポリシー
- [ ] ダッシュボード構築

## 関連リソース

- [Rustトレーシングドキュメント](https://docs.rs/tracing/)
- [構造化ログのベストプラクティス](https://12factor.net/logs)
- [ログレベルガイドライン](https://sematext.com/blog/logging-levels/)

---

**作成日**: 2025-01-15  
**更新日**: 2025-01-15  
**バージョン**: 1.0  
**作成者**: Time Tracker Go 開発チーム


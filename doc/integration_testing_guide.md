# 統合テストガイド

## 概要

このドキュメントでは、Time Tracker Goアプリケーションの統合テストについて説明します。統合テストは、フロントエンドとバックエンドの連携が正常に動作することを確認するために重要です。

## 統合テストの種類

### 1. フロントエンド・バックエンド統合テスト

**ファイル**: `tests/integration/frontend_backend_integration_test.ts`

**目的**: 
- 実際のTauriコマンドを呼び出してフロントエンドとバックエンドの連携をテスト
- プロジェクト管理、タスク管理、タイムトラッキング機能の統合動作を確認
- エラーハンドリングの動作を確認

**テスト対象機能**:
- プロジェクト作成・更新・アーカイブ
- タスク作成・更新・アーカイブ
- タイマー開始・停止・状態確認
- エラーケース（存在しないID、無効な入力など）

### 2. バックエンド統合テスト

**ファイル**: `tests/integration/task_management_integration_test.rs`

**目的**:
- バックエンドのユースケースとリポジトリの連携をテスト
- データベース操作の整合性を確認
- ビジネスロジックの動作を確認

## 統合テストの実行方法

### 前提条件

1. **Tauriアプリケーションの準備**
   ```bash
   # 依存関係のインストール
   npm install
   
   # Rust依存関係の確認
   cd src-tauri && cargo build
   ```

2. **データベースの準備**
   ```bash
   # マイグレーションの実行
   cd src-tauri && cargo run --bin migrate
   ```

### 実行手順

#### 1. フロントエンド・バックエンド統合テスト

```bash
# 統合テストを実行（Tauriアプリケーションを自動起動）
npm run test:integration

# または、手動で実行する場合
# 1. Tauriアプリケーションを起動
npm run tauri:dev

# 2. 別のターミナルで統合テストを実行
npm test -- tests/integration/frontend_backend_integration_test.ts
```

#### 2. バックエンド統合テスト

```bash
# Rust統合テストを実行
cd src-tauri && cargo test --test task_management_integration_test
```

#### 3. 全テストの実行

```bash
# ユニットテスト + 統合テスト + E2Eテスト
npm run test:all
```

## テストの構造

### フロントエンド・バックエンド統合テスト

```typescript
describe('フロントエンド・バックエンド統合テスト', () => {
  let testProjectId: number | null = null
  let testTaskId: number | null = null

  beforeEach(async () => {
    // テスト用データの準備
  })

  afterEach(async () => {
    // テスト用データのクリーンアップ
  })

  describe('プロジェクト管理機能', () => {
    it('プロジェクト作成が正常に動作する', async () => {
      // 実際のTauriコマンドを呼び出し
      const project = await invoke('create_project', {
        request: { name: 'テストプロジェクト' }
      })
      
      // 結果を検証
      expect(project.name).toBe('テストプロジェクト')
    })
  })
})
```

### バックエンド統合テスト

```rust
#[tokio::test]
async fn test_task_creation_with_database() {
    let mut ctx = TestContext::new().await;
    
    // テストデータの準備
    let project = ctx.create_test_project("テストプロジェクト").await;
    
    // ユースケースを実行
    let command = CreateTaskCommand {
        project_id: project.id(),
        name: "重要なタスク".to_string(),
    };
    
    let result = ctx.app_service.task_use_cases().create_task(command).await;
    assert!(result.is_ok());
}
```

## テストデータの管理

### テスト用データベース

- **フロントエンド・バックエンド統合テスト**: 実際のSQLiteデータベースを使用
- **バックエンド統合テスト**: インメモリSQLiteデータベースを使用

### データクリーンアップ

各テストの後に、作成したテストデータを自動的にクリーンアップします：

```typescript
afterEach(async () => {
  if (testTaskId) {
    await invoke('archive_task', { request: { id: testTaskId } })
  }
  if (testProjectId) {
    await invoke('archive_project', { request: { id: testProjectId, force: true } })
  }
})
```

## トラブルシューティング

### よくある問題

#### 1. Tauriアプリケーションが起動しない

```bash
# プロセスを確認
ps aux | grep tauri

# 強制終了
pkill -f "tauri|cargo.*tauri|vite|time-tracker-go"

# 再起動
npm run tauri:dev
```

#### 2. データベースエラー

```bash
# データベースファイルを削除して再作成
rm -f src-tauri/data/time_tracker.db
cd src-tauri && cargo run --bin migrate
```

#### 3. ポート競合

```bash
# 使用中のポートを確認
lsof -i :1420

# プロセスを終了
kill -9 <PID>
```

### デバッグ方法

#### 1. ログの確認

```bash
# Tauriアプリケーションのログ
RUST_LOG=debug npm run tauri:dev

# テスト実行時のログ
npm test -- tests/integration/frontend_backend_integration_test.ts --reporter=verbose
```

#### 2. 手動テスト

```bash
# Tauriアプリケーションを起動
npm run tauri:dev

# ブラウザで http://localhost:1420 にアクセス
# 開発者ツールでコンソールログを確認
```

## 継続的インテグレーション

### CI/CDパイプライン

統合テストはCI/CDパイプラインに組み込まれています：

```yaml
# .github/workflows/test.yml
- name: Run Integration Tests
  run: |
    npm run test:integration
```

### テスト結果の報告

統合テストの結果は以下の形式で報告されます：

- **成功**: すべてのTauriコマンドが正常に動作
- **失敗**: フロントエンド・バックエンド間の通信エラー
- **スキップ**: Tauriアプリケーションが起動できない場合

## ベストプラクティス

### 1. テストの独立性

各テストは独立して実行できるように設計されています：

```typescript
beforeEach(async () => {
  // テスト用データの準備
})

afterEach(async () => {
  // テスト用データのクリーンアップ
})
```

### 2. エラーハンドリング

エラーケースも積極的にテストします：

```typescript
it('存在しないプロジェクトIDでタスク作成時にエラーが発生する', async () => {
  await expect(
    invoke('create_task', {
      request: { project_id: 99999, name: 'テストタスク' }
    })
  ).rejects.toThrow()
})
```

### 3. 非同期処理の適切な待機

```typescript
// タイマー開始後の状態確認
await new Promise(resolve => setTimeout(resolve, 100))
const currentTimer = await invoke('get_current_timer')
expect(currentTimer.is_running).toBe(true)
```

## まとめ

統合テストは、フロントエンドとバックエンドの連携が正常に動作することを確認する重要なテストです。定期的に実行することで、リグレッションを早期に発見し、アプリケーションの品質を維持できます。

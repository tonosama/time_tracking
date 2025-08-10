# 開発者ガイド

## 開発環境詳細

### VS Code推奨拡張

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tauri-apps.tauri-vscode",
    "bradlc.vscode-tailwindcss",
    "esbenp.prettier-vscode",
    "ms-vscode.vscode-typescript-next",
    "vitest.explorer"
  ]
}
```

### 開発ワークフロー

#### 1. Rustバックエンド開発

```bash
# Rustテスト（単体・結合）
cd src-tauri && cargo test

# 特定テスト実行
cargo test test_name

# Lintチェック
cargo clippy

# フォーマット
cargo fmt
```

#### 2. React/TypeScriptフロントエンド開発

```bash
# Unitテスト
npm test

# 型チェック
npx tsc --noEmit

# Lintチェック
npx eslint src/

# フォーマット
npx prettier --write src/
```

### TDD開発フロー

#### バックエンド（Rust）

1. **Red**: テスト作成（失敗）
2. **Green**: 最小限の実装でテスト通過
3. **Refactor**: コード改善

```bash
# テストファイル作成例
src-tauri/src/domain/entities/project.rs

#[cfg(test)]
mod tests {
    #[test]
    fn プロジェクト作成ができること() {
        // Arrange, Act, Assert
    }
}
```

#### フロントエンド（React/TypeScript）

```bash
# テストファイル作成例
src/components/projects/__tests__/ProjectForm.test.tsx

describe('ProjectForm', () => {
  it('プロジェクト名入力ができること', () => {
    // Arrange, Act, Assert
  })
})
```

### デバッグ

#### Rustデバッグ

```bash
# デバッグビルド実行
cd src-tauri && cargo run

# ログ出力レベル設定
RUST_LOG=debug cargo run
```

#### React DevTools

ブラウザで http://localhost:1420 にアクセスし、React DevToolsを使用

### パフォーマンス

#### Rustプロファイリング

```bash
# リリースビルドでベンチマーク
cargo bench

# プロファイリング
cargo install flamegraph
cargo flamegraph --bin time-tracker-go
```

## Git運用

### ブランチ戦略

- `main`: 安定版
- `develop`: 開発版
- `feature/*`: 機能開発
- `hotfix/*`: 緊急修正

### コミットメッセージ

```
[type]: [subject]

[body]

[footer]
```

**Types:**
- `feat`: 新機能
- `fix`: バグ修正
- `docs`: ドキュメント
- `style`: フォーマット
- `refactor`: リファクタリング
- `test`: テスト追加
- `chore`: その他

**例:**
```
feat: プロジェクト作成フォームを追加

- バリデーション機能実装
- エラーハンドリング追加
- Unitテスト完備

Closes #123
```

## リリース

### バージョニング

Semantic Versioning (`MAJOR.MINOR.PATCH`)

### リリース手順

1. **テスト実行**
```bash
npm test
cd src-tauri && cargo test
npm run test:e2e
```

2. **ビルド確認**
```bash
npm run tauri:build
```

3. **バージョン更新**
```bash
# package.json
# src-tauri/Cargo.toml
# src-tauri/tauri.conf.json
```

4. **タグ作成**
```bash
git tag v0.1.0
git push origin v0.1.0
```

## トラブルシューティング（開発者向け）

### Cargoキャッシュクリア

```bash
cd src-tauri
cargo clean
rm -rf target/
```

### Node_modulesクリア

```bash
rm -rf node_modules/
npm install
```

### Tauri依存関係更新

```bash
# Cargo.toml の tauri バージョン確認
cargo update
```

### データベース初期化

```bash
# 開発用データベースリセット
rm -f ~/Library/Application\ Support/Time\ Tracker\ Go/time_tracker.db
```

# Time Tracker Go

軽量で高速なmacOS向けタイムトラッキングアプリケーション

## プロジェクト概要

このアプリケーションは、ドメイン駆動設計（DDD）とテスト駆動開発（TDD）に基づいて構築された、ローカル完結型のタイムトラッキングツールです。

### 技術スタック

- **フレームワーク**: Tauri
- **バックエンド**: Rust
- **フロントエンド**: React + TypeScript
- **データベース**: SQLite（暗号化対応）
- **テスト**: Rust（cargo test）、Vitest、Playwright

## アーキテクチャ

### DDDの4層アーキテクチャ

```
src-tauri/src/
├── domain/              # ドメイン層
│   ├── entities/        # エンティティ
│   ├── value_objects/   # 値オブジェクト
│   ├── repositories/    # リポジトリトレイト
│   └── services/        # ドメインサービス
├── application/         # アプリケーション層
│   ├── use_cases/       # ユースケース
│   ├── dto/             # データ転送オブジェクト
│   └── services/        # アプリケーションサービス
├── infrastructure/     # インフラ層
│   ├── database/        # データベース接続・マイグレーション
│   ├── repositories/   # リポジトリ実装
│   └── config/          # 設定管理
└── presentation/       # プレゼンテーション層
    └── commands/        # Tauriコマンド
```

### フロントエンド構造

```
src/
├── components/         # UIコンポーネント
│   ├── common/         # 共通コンポーネント
│   ├── projects/       # プロジェクト関連
│   ├── tasks/          # タスク関連
│   ├── time_tracking/  # 時間計測関連
│   └── reports/        # レポート関連
├── hooks/              # カスタムフック
├── services/           # API通信
├── types/              # 型定義
└── utils/              # ユーティリティ
```

## データモデル

イミュータブル（追記専用）データモデルを採用：

- **バージョン管理**: プロジェクト・タスクの変更履歴を管理
- **イベント管理**: 時間計測とタグ操作をイベントとして記録
- **ビュー**: 現在値は専用ビューで導出

## 開発環境セットアップ

### 前提条件

- Node.js 18+
- Rust 1.70+
- Tauri CLI

### インストール

```bash
# 依存関係のインストール
npm install

# Tauri CLIのインストール
npm install -g @tauri-apps/cli

# 開発サーバーの起動
npm run tauri dev
```

### テスト実行

```bash
# Rustユニットテスト
cd src-tauri && cargo test

# フロントエンドテスト
npm test

# E2Eテスト
npm run test:e2e
```

## プロジェクト設計文書

詳細な設計については以下の文書を参照してください：

- [プロダクト要求仕様書（PRD）](doc/prd_time_tracker_go.md)
- [設計文書](doc/design_doc_time_tracker_go.md)
- [バックログ](doc/backlog/backlog.yaml)


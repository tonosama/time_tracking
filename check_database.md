# データベースの登録状態確認方法

## データベースファイルの場所

macOSでは、データベースファイルは以下の場所に保存されます：
```
~/Library/Application Support/time-tracker-go/time_tracker.db
```

## データベース確認方法

### 1. アプリケーションを起動
まず、アプリケーションを起動してデータベースファイルを作成します：
```bash
cd /Users/tonosama/work/time_tracking
npm run tauri dev
```

### 2. SQLiteクライアントでデータベースを確認

#### 方法1: sqlite3コマンドラインツール
```bash
# データベースファイルに接続
sqlite3 ~/Library/Application\ Support/time-tracker-go/time_tracker.db

# 作成したSQLファイルを実行
.read check_database.sql

# または、個別のクエリを実行
SELECT * FROM projects;
SELECT * FROM project_versions;
SELECT * FROM project_current_view;
```

#### 方法2: DB Browser for SQLite（GUIツール）
1. [DB Browser for SQLite](https://sqlitebrowser.org/)をインストール
2. アプリケーションを起動
3. 「データベースを開く」で `time_tracker.db` を選択
4. 「SQL実行」タブでクエリを実行

#### 方法3: VSCode拡張機能
1. VSCodeに「SQLite」拡張機能をインストール
2. `time_tracker.db` ファイルを開く
3. SQLクエリを実行

## 確認すべき内容

### プロジェクト作成後の確認
1. **projectsテーブル**: プロジェクトIDが登録されているか
2. **project_versionsテーブル**: プロジェクトの詳細情報が登録されているか
3. **project_current_view**: 現在のプロジェクト一覧が正しく表示されるか

### サンプルデータの確認
アプリケーション初回起動時には、以下のサンプルデータが自動的に登録されます：
- プロジェクト: 「サンプルプロジェクト1」「サンプルプロジェクト2」
- タスク: 「設計タスク」「実装タスク」「テストタスク」
- 時間エントリ: サンプルの開始・停止イベント
- タグ: 「フロントエンド」「バックエンド」「データベース」「テスト」

## 主要なクエリ例

### プロジェクト一覧を確認
```sql
SELECT * FROM project_current_view;
```

### プロジェクトの履歴を確認
```sql
SELECT 
    project_id,
    version,
    name,
    status,
    effective_at
FROM project_versions
WHERE project_id = 1
ORDER BY version;
```

### タスク一覧を確認
```sql
SELECT 
    t.task_id,
    t.name as task_name,
    p.name as project_name,
    t.status
FROM task_current_view t
JOIN project_current_view p ON t.project_id = p.project_id;
```

### 時間エントリを確認
```sql
SELECT 
    t.name as task_name,
    te.start_time,
    te.end_time,
    te.duration_in_seconds
FROM time_entries_view te
JOIN task_current_view t ON te.task_id = t.task_id;
```

## トラブルシューティング

### データベースファイルが見つからない場合
1. アプリケーションが正常に起動しているか確認
2. ログファイルを確認（`~/Library/Logs/time-tracker-go/`）
3. データディレクトリが作成されているか確認

### テーブルが存在しない場合
1. マイグレーションが正常に実行されているか確認
2. `schema_migrations`テーブルでマイグレーション履歴を確認

### データが表示されない場合
1. ビューが正しく作成されているか確認
2. サンプルデータが読み込まれているか確認
3. プロジェクト作成処理が正常に完了しているか確認

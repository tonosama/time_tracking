-- データベース状態確認用SQLクエリ

-- 1. プロジェクト識別子テーブル
SELECT '=== projects テーブル ===' as info;
SELECT * FROM projects;

-- 2. プロジェクトバージョンテーブル
SELECT '=== project_versions テーブル ===' as info;
SELECT 
    project_id,
    version,
    name,
    status,
    effective_at
FROM project_versions
ORDER BY project_id, version;

-- 3. プロジェクト現在値ビュー
SELECT '=== project_current_view ===' as info;
SELECT 
    project_id,
    name,
    status,
    effective_at
FROM project_current_view
ORDER BY project_id;

-- 4. タスク識別子テーブル
SELECT '=== tasks テーブル ===' as info;
SELECT * FROM tasks;

-- 5. タスクバージョンテーブル
SELECT '=== task_versions テーブル ===' as info;
SELECT 
    task_id,
    version,
    project_id,
    name,
    status,
    effective_at
FROM task_versions
ORDER BY task_id, version;

-- 6. タスク現在値ビュー
SELECT '=== task_current_view ===' as info;
SELECT 
    task_id,
    project_id,
    name,
    status,
    effective_at
FROM task_current_view
ORDER BY task_id;

-- 7. 時間エントリイベント
SELECT '=== time_entry_events テーブル ===' as info;
SELECT 
    id,
    task_id,
    event_type,
    at,
    start_event_id,
    payload
FROM time_entry_events
ORDER BY task_id, at, id;

-- 8. 時間エントリビュー
SELECT '=== time_entries_view ===' as info;
SELECT 
    task_id,
    start_event_id,
    start_time,
    end_time,
    duration_in_seconds,
    next_start_time
FROM time_entries_view
ORDER BY task_id, start_time;

-- 9. タグマスタ
SELECT '=== tags テーブル ===' as info;
SELECT * FROM tags;

-- 10. タスクタグイベント
SELECT '=== task_tag_events テーブル ===' as info;
SELECT 
    id,
    task_id,
    tag_id,
    event_type,
    at
FROM task_tag_events
ORDER BY task_id, tag_id, at;

-- 11. 現在タグ集合ビュー
SELECT '=== task_tags_current ===' as info;
SELECT 
    task_id,
    tag_id
FROM task_tags_current
ORDER BY task_id, tag_id;

-- 12. テーブル一覧
SELECT '=== テーブル一覧 ===' as info;
SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;

-- 13. ビュー一覧
SELECT '=== ビュー一覧 ===' as info;
SELECT name FROM sqlite_master WHERE type='view' ORDER BY name;

-- 14. スキーマバージョン
SELECT '=== schema_migrations ===' as info;
SELECT * FROM schema_migrations;

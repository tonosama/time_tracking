-- サンプルデータ - 開発・テスト用

-- サンプルプロジェクト
INSERT INTO projects (id) VALUES (1), (2);

INSERT INTO project_versions (project_id, version, name, status, effective_at) VALUES
(1, 1, 'サンプルプロジェクト1', 'active', '2024-01-01T00:00:00Z'),
(2, 1, 'サンプルプロジェクト2', 'active', '2024-01-01T00:00:00Z');

-- サンプルタスク
INSERT INTO tasks (id) VALUES (1), (2), (3);

INSERT INTO task_versions (task_id, version, project_id, name, status, effective_at) VALUES
(1, 1, 1, '設計タスク', 'active', '2024-01-01T00:00:00Z'),
(2, 1, 1, '実装タスク', 'active', '2024-01-01T00:00:00Z'),
(3, 1, 2, 'テストタスク', 'active', '2024-01-01T00:00:00Z');

-- サンプルタグ
INSERT INTO tags (id, name) VALUES
(1, 'フロントエンド'),
(2, 'バックエンド'),
(3, 'データベース'),
(4, 'テスト');

-- サンプル時間エントリイベント
INSERT INTO time_entry_events (task_id, event_type, at) VALUES
(1, 'start', '2024-01-01T09:00:00Z'),
(1, 'stop', '2024-01-01T10:30:00Z'),
(2, 'start', '2024-01-01T11:00:00Z'),
(2, 'stop', '2024-01-01T12:00:00Z');

-- サンプルタグ付与
INSERT INTO task_tag_events (task_id, tag_id, event_type, at) VALUES
(1, 1, 'add', '2024-01-01T00:00:00Z'),
(2, 2, 'add', '2024-01-01T00:00:00Z'),
(3, 4, 'add', '2024-01-01T00:00:00Z');


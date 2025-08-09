-- 初期スキーマ - イミュータブルデータモデル

-- スキーマバージョン管理
CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);

-- プロジェクト識別子テーブル
CREATE TABLE projects (
  id INTEGER PRIMARY KEY
);

-- プロジェクトバージョンテーブル
CREATE TABLE project_versions (
  id INTEGER PRIMARY KEY,
  project_id INTEGER NOT NULL,
  version INTEGER NOT NULL,
  name TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('active','archived')),
  effective_at TEXT NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects(id),
  UNIQUE(project_id, version)
);

-- タスク識別子テーブル
CREATE TABLE tasks (
  id INTEGER PRIMARY KEY
);

-- タスクバージョンテーブル
CREATE TABLE task_versions (
  id INTEGER PRIMARY KEY,
  task_id INTEGER NOT NULL,
  version INTEGER NOT NULL,
  project_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('active','archived')),
  effective_at TEXT NOT NULL,
  FOREIGN KEY(task_id) REFERENCES tasks(id),
  FOREIGN KEY(project_id) REFERENCES projects(id),
  UNIQUE(task_id, version)
);

-- タイムエントリイベントテーブル
CREATE TABLE time_entry_events (
  id INTEGER PRIMARY KEY,
  task_id INTEGER NOT NULL,
  event_type TEXT NOT NULL CHECK(event_type IN ('start','stop','annotate')),
  at TEXT NOT NULL,
  start_event_id INTEGER,
  payload TEXT,
  FOREIGN KEY(task_id) REFERENCES tasks(id)
);

-- タグマスタテーブル
CREATE TABLE tags (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE
);

-- タスクタグイベントテーブル
CREATE TABLE task_tag_events (
  id INTEGER PRIMARY KEY,
  task_id INTEGER NOT NULL,
  tag_id INTEGER NOT NULL,
  event_type TEXT NOT NULL CHECK(event_type IN ('add','remove')),
  at TEXT NOT NULL,
  FOREIGN KEY(task_id) REFERENCES tasks(id),
  FOREIGN KEY(tag_id) REFERENCES tags(id)
);

-- インデックス
CREATE INDEX idx_project_versions_project_version ON project_versions(project_id, version DESC);
CREATE INDEX idx_task_versions_task_version ON task_versions(task_id, version DESC);
CREATE INDEX idx_time_entry_events_task_at ON time_entry_events(task_id, at);
CREATE INDEX idx_task_tag_events_task_tag_at ON task_tag_events(task_id, tag_id, at);


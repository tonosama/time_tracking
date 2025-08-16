-- ビュー定義 - 現在値導出用のビュー

-- プロジェクト現在値ビュー
CREATE VIEW IF NOT EXISTS project_current_view AS
WITH latest AS (
  SELECT pv.project_id, MAX(pv.effective_at) AS max_effective_at
  FROM project_versions pv
  GROUP BY pv.project_id
), latest_tie_break AS (
  SELECT pv.project_id, MAX(pv.version) AS max_version
  FROM project_versions pv
  JOIN latest l
    ON l.project_id = pv.project_id
   AND l.max_effective_at = pv.effective_at
  GROUP BY pv.project_id
)
SELECT pv.project_id, pv.name, pv.status, pv.effective_at
FROM project_versions pv
JOIN latest l
  ON l.project_id = pv.project_id AND l.max_effective_at = pv.effective_at
JOIN latest_tie_break lb
  ON lb.project_id = pv.project_id AND lb.max_version = pv.version;

-- タスク現在値ビュー
CREATE VIEW IF NOT EXISTS task_current_view AS
WITH latest AS (
  SELECT tv.task_id, MAX(tv.effective_at) AS max_effective_at
  FROM task_versions tv
  GROUP BY tv.task_id
), latest_tie_break AS (
  SELECT tv.task_id, MAX(tv.version) AS max_version
  FROM task_versions tv
  JOIN latest l
    ON l.task_id = tv.task_id
   AND l.max_effective_at = tv.effective_at
  GROUP BY tv.task_id
)
SELECT tv.task_id, tv.project_id, tv.name, tv.status, tv.effective_at
FROM task_versions tv
JOIN latest l
  ON l.task_id = tv.task_id AND l.max_effective_at = tv.effective_at
JOIN latest_tie_break lb
  ON lb.task_id = tv.task_id AND lb.max_version = tv.version;

-- 時間エントリビュー
CREATE VIEW IF NOT EXISTS time_entries_view AS
WITH starts AS (
  SELECT
    id AS start_event_id,
    task_id,
    at AS start_time,
    LEAD(at) OVER (PARTITION BY task_id ORDER BY at, id) AS next_start_time
  FROM time_entry_events
  WHERE event_type = 'start'
), paired AS (
  SELECT
    s.task_id,
    s.start_event_id,
    s.start_time,
    s.next_start_time,
    (
      SELECT st.at
      FROM time_entry_events st
      WHERE st.task_id = s.task_id
        AND st.event_type = 'stop'
        AND (st.at > s.start_time OR (st.at = s.start_time AND st.id > s.start_event_id))
        AND (s.next_start_time IS NULL OR st.at < s.next_start_time)
      ORDER BY st.at, st.id
      LIMIT 1
    ) AS stop_time
  FROM starts s
)
SELECT
  task_id,
  start_event_id,
  start_time,
  COALESCE(stop_time, next_start_time) AS end_time,
  CASE
    WHEN COALESCE(stop_time, next_start_time) IS NOT NULL
    THEN CAST((julianday(COALESCE(stop_time, next_start_time)) - julianday(start_time)) * 86400 AS INTEGER)
    ELSE NULL
  END AS duration_in_seconds,
  next_start_time
FROM paired;

-- 現在タグ集合ビュー
CREATE VIEW IF NOT EXISTS task_tags_current AS
WITH ranked AS (
  SELECT
    tte.task_id,
    tte.tag_id,
    tte.event_type,
    tte.at,
    tte.id,
    ROW_NUMBER() OVER (
      PARTITION BY tte.task_id, tte.tag_id
      ORDER BY tte.at DESC, tte.id DESC
    ) AS rn
  FROM task_tag_events tte
)
SELECT r.task_id, r.tag_id
FROM ranked r
WHERE r.rn = 1
  AND r.event_type = 'add';


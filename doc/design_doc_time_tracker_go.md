# Time Tracker Go - Design Doc（設計文書）

**作成者:** AI Assistant  
**作成日:** 2025-08-04
**ステータス:** 確定  
**バージョン:** 1.0

## 1. コンテキストとスコープ

### 問題の説明
市場にある多くのタイムトラッキングツールはSaaS型であり、セキュリティポリシーが厳しい企業環境などでは自由にインストールして利用することが難しい。このため、ユーザーはローカル環境で完結し、手軽に導入・利用できる高機能なタイムトラッキングツールを必要としている。

### 背景
プロダクト要求仕様書（PRD）に基づき、軽量で高速なmacOS向けのデスクトップアプリケーションを開発する。データはすべてローカルに保存され、外部との通信は行わない。

## 2. 目標と非目標

### 目標
- 軽量で、PCのリソースをほとんど消費しないアプリケーションを提供する。
- OSネイティブのような高速なパフォーマンスを実現する。
- ユーザーデータをローカルで安全に管理する。
- ドメイン駆動設計（DDD）とテスト駆動開発（TDD）に基づいた、保守性の高いコードベースを構築する。

### 非目標
- クラウドとのデータ同期機能。
- チームでの時間共有機能。
- WindowsおよびLinuxのサポート（初期リリース時点）。

## 3. 設計概要
Tauriフレームワークを採用し、フロントエンドとバックエンドを明確に分離したアーキテクチャとする。

- **フロントエンド:** ReactとTypeScriptを使用し、UIの描画とユーザーインタラクションに責務を限定する。OSネイティブのWebView上で動作する。
- **バックエンド:** Rustを使用し、すべてのビジネスロジック（ドメイン層、アプリケーション層）を実装する。データベースとのやり取りもバックエンドが担当する。
- **データベース:** SQLiteを使用し、アプリケーションのデータを単一のファイルとしてローカルに保存する。

```mermaid
graph TD
    A[フロントエンド / UI<br>(React + TypeScript)] -- Tauri Command --> B(バックエンド<br>Rust);
    B -- CRUD --> C(データベース<br>SQLite);
```

## 4. 詳細設計

### 技術的アプローチ
- **フレームワーク:** Tauri
- **フロントエンド言語/ライブラリ:** TypeScript, React
- **バックエンド言語:** Rust
- **データベース:** SQLite (rusqlite クレートを利用)
- **開発手法:** ドメイン駆動設計（DDD）とテスト駆動開発（TDD）を全面的に採用する。

### 決定事項（確定）
- **データモデル:** イミュータブル（追記専用）モデルを採用し、UPDATE/DELETEは不使用。
- **バージョン管理:** `projects`/`tasks` に対し `project_versions`/`task_versions` を導入。現在値は `effective_at` → `version` → `id` の優先順位で決定。
- **イベント:** 時間計測は `time_entry_events`、タグは `task_tag_events` に記録し、畳み込みで現在値を導出。
- **ビュー:** `task_current_view`、`time_entries_view`、`task_tags_current` を提供し、フロントは原則これらを介して取得。
- **整合性/補正:** START/STOPの状態遷移ルールを定義し、異常は補正イベント（INSERT-only）で解消。
- **採番/競合:** `version` はエンティティ内連番。挿入は `BEGIN IMMEDIATE` により競合を最小化、必要時はリトライ。
- **マイグレーション/運用:** `schema_migrations` を導入。`WAL`/`foreign_keys=ON` を採用し、バックアップ・暗号化（SQLCipher）手順を定義。
- **保存場所:** `~/Library/Application Support/<AppName>` に暗号化DBとして保存。

### データモデル/ストレージ
SQLite を使用したイミュータブル（追記専用）データモデルを採用します。更新（UPDATE）・削除（DELETE）は行わず、状態変化は「バージョン行」または「イベント行」の追加で表現します。日時は `TEXT` 型（ISO 8601: `YYYY-MM-DDTHH:MM:SSZ`）を採用します。

#### 方針
- 書き込みは常に INSERT のみ。
- エンティティの現在値（Current State）は「最新バージョン」または「イベントのリプレイ」で導出。
- 監査・ロールバック容易性と並列編集時の衝突低減を重視。

#### テーブル定義（バージョン管理型）

**1. `projects`（識別子テーブル）**
プロジェクトの不変識別子を保持します。

| カラム名 | データ型 | 制約 | 説明 |
| :--- | :--- | :--- | :--- |
| `id` | INTEGER | PRIMARY KEY | プロジェクトID（不変） |

**2. `project_versions`（バージョンテーブル）**
プロジェクトの属性変化を履歴として保持します。

| カラム名 | データ型 | 制約 | 説明 |
| :--- | :--- | :--- | :--- |
| `id` | INTEGER | PRIMARY KEY | 行ID |
| `project_id` | INTEGER | NOT NULL, FOREIGN KEY(projects.id) | 対象プロジェクト |
| `version` | INTEGER | NOT NULL | リビジョン番号（1,2,3,…） |
| `name` | TEXT | NOT NULL | プロジェクト名 |
| `status` | TEXT | NOT NULL, CHECK(status IN ('active','archived')) | 状態 |
| `effective_at` | TEXT | NOT NULL | このバージョンの発効日時 |
| | | UNIQUE(project_id, version) | 同一プロジェクト内で一意 |

**3. `tasks`（識別子テーブル）**
タスクの不変識別子を保持します。

| カラム名 | データ型 | 制約 | 説明 |
| :--- | :--- | :--- | :--- |
| `id` | INTEGER | PRIMARY KEY | タスクID（不変） |

**4. `task_versions`（バージョンテーブル）**
タスクの属性変化を履歴として保持します。

| カラム名 | データ型 | 制約 | 説明 |
| :--- | :--- | :--- | :--- |
| `id` | INTEGER | PRIMARY KEY | 行ID |
| `task_id` | INTEGER | NOT NULL, FOREIGN KEY(tasks.id) | 対象タスク |
| `version` | INTEGER | NOT NULL | リビジョン番号（1,2,3,…） |
| `project_id` | INTEGER | NOT NULL, FOREIGN KEY(projects.id) | 紐づくプロジェクト |
| `name` | TEXT | NOT NULL | タスク名 |
| `status` | TEXT | NOT NULL, CHECK(status IN ('active','archived')) | 状態 |
| `effective_at` | TEXT | NOT NULL | このバージョンの発効日時 |
| | | UNIQUE(task_id, version) | 同一タスク内で一意 |

#### テーブル定義（イベント管理型）

**5. `time_entry_events`（タイムトラッキングのイベント）**
タイマーの開始/停止などをイベントとして記録します。区間の確定はビューまたはクエリで導出します。

| カラム名 | データ型 | 制約 | 説明 |
| :--- | :--- | :--- | :--- |
| `id` | INTEGER | PRIMARY KEY | イベントID |
| `task_id` | INTEGER | NOT NULL, FOREIGN KEY(tasks.id) | 対象タスク |
| `event_type` | TEXT | NOT NULL, CHECK(event_type IN ('start','stop','annotate')) | イベント種別 |
| `at` | TEXT | NOT NULL | 発生日時 |
| `start_event_id` | INTEGER |  | STOPが対応するSTARTを参照（必要時） |
| `payload` | TEXT |  | 追加情報（JSON文字列: notes等） |

推奨ビュー（例）：`time_entries_view`
- START/STOPをペアリングして `[task_id, start_time, end_time, duration_in_seconds, notes]` を算出するビュー。
- 実装都合でマテリアライズドビュー（更新は再生成）を検討可。

**6. `tags`（マスタ）**
タグのマスタ定義です。

| カラム名 | データ型 | 制約 | 説明 |
| :--- | :--- | :--- | :--- |
| `id` | INTEGER | PRIMARY KEY | タグID |
| `name` | TEXT | NOT NULL, UNIQUE | タグ名 |

**7. `task_tag_events`（タグ付与/剥奪イベント）**
タスクへのタグの追加/削除をイベントとして記録します。現在付与中のタグはイベントを畳み込んで導出します。

| カラム名 | データ型 | 制約 | 説明 |
| :--- | :--- | :--- | :--- |
| `id` | INTEGER | PRIMARY KEY | イベントID |
| `task_id` | INTEGER | NOT NULL, FOREIGN KEY(tasks.id) | 対象タスク |
| `tag_id` | INTEGER | NOT NULL, FOREIGN KEY(tags.id) | 対象タグ |
| `event_type` | TEXT | NOT NULL, CHECK(event_type IN ('add','remove')) | 操作種別 |
| `at` | TEXT | NOT NULL | 発生日時 |

推奨ビュー（例）：`task_tags_current`
- 直近のイベントを評価して、現在有効なタグ集合を返すビュー。

#### インデックス
- `project_versions(project_id, version DESC)`
- `task_versions(task_id, version DESC)`
- `time_entry_events(task_id, at)`
- `task_tag_events(task_id, tag_id, at)`

#### 移行/運用ルール
- 更新・削除は禁止。訂正は新しいバージョン/イベントを追加して表現。
- 現在値の取得は最新バージョン（MAX(version)）や最新イベントで導出。
- 不整合（例: STOPなしのSTART）はクエリで検出し、補正イベントで解消。

#### データ保存場所
データは暗号化された単一のDBファイルとして、macOSのアプリケーションサポートディレクトリ (`~/Library/Application Support/<AppName>`) に保存する。

#### 現在値ビュー定義（提案1）
`task_versions` から各タスクの現在値を導出するビューを定義する。タイブレークは `effective_at` 降順 → `version` 降順 → `id` 降順。

```sql
CREATE VIEW task_current_view AS
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
```

受け入れ基準:
- 各 `task_id` につき常に1行のみを返す。
- 同一時刻の競合時も決定的に一意が選ばれる。

#### 時間計測ビュー定義（提案2）
`time_entry_events` の START/STOP をペアリングし、現在の区間リストを導出。次の STOP がなければ次の START を暗黙STOPとみなす。さらにどちらも無ければ実行中として `end_time` は NULL。

```sql
CREATE VIEW time_entries_view AS
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
  END AS duration_in_seconds
FROM paired;
```

インデックス指針: `time_entry_events(task_id, at, id)`。

#### 現在タグ集合ビュー定義（提案3）
`task_tag_events` を畳み込み、現在付与中のタグ集合を導出。

```sql
CREATE VIEW task_tags_current AS
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
```

詳細版（任意）:

```sql
CREATE VIEW task_tags_current_detailed AS
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
SELECT r.task_id, r.tag_id, tg.name AS tag_name, r.at AS applied_at
FROM ranked r
JOIN tags tg ON tg.id = r.tag_id
WHERE r.rn = 1
  AND r.event_type = 'add';
```

インデックス指針: `task_tag_events(task_id, tag_id, at DESC, id DESC)`。

#### START/STOP 整合性と補正運用（提案4）
- 状態遷移: idle → start → running → stop → idle。
- 許容イベント:
  - idle中: start のみ可
  - running中: stop/annotate 可（start 到来は暗黙STOPで直前区間をクローズ）
- 注釈: annotate は区間内（start〜stop/暗黙stop）に紐付け。区間外は backfill として扱い注記。
- 並び順: `(at ASC, id ASC)` を正準順序。

異常検出（例）:

```sql
-- 重複START候補
WITH ordered AS (
  SELECT id, task_id, event_type, at,
         LAG(event_type) OVER (PARTITION BY task_id ORDER BY at, id) AS prev_type
  FROM time_entry_events
)
SELECT * FROM ordered
WHERE event_type='start' AND prev_type='start';
```

```sql
-- 孤立STOP候補
WITH starts AS (
  SELECT id, task_id, at,
         ROW_NUMBER() OVER (PARTITION BY task_id ORDER BY at, id) AS rn_start
  FROM time_entry_events WHERE event_type='start'
),
stops AS (
  SELECT id, task_id, at,
         ROW_NUMBER() OVER (PARTITION BY task_id ORDER BY at, id) AS rn_stop
  FROM time_entry_events WHERE event_type='stop'
)
SELECT s.*
FROM stops s
LEFT JOIN starts t
  ON t.task_id=s.task_id AND t.rn_start = s.rn_stop
WHERE t.id IS NULL;
```

補正ポリシー（INSERT-only）:
- 暗黙STOP: 重複START時、直前に `stop` を補正挿入（`payload.reason='implicit_stop'`）。
- 孤立STOP: 直前の `start` を backfill 追加（`payload.reason='backfill_start'`）または STOP を無効化注記。
- 逆行/負区間: 補正イベントで除外フラグを付与、または集計から除外。
- 長時間実行: 閾値で自動STOP（`payload.reason='auto_cutoff'`）。

#### バージョン採番と同時挿入競合解決（提案5）
- 採番規則:
  - `version`: 各エンティティ内の連番（1,2,3,…）。
  - `effective_at`: 発効時刻。過去/未来のバックフィル可。
- 現在値決定: `effective_at` 最大 → `version` 最大 → `id` 最大。
- 一意性: `UNIQUE(project_id, version)` / `UNIQUE(task_id, version)` を維持。
- 競合解決（SQLite）擬似コード:

```text
BEGIN IMMEDIATE; -- 書き込みロック
next_version = SELECT COALESCE(MAX(version),0)+1 FROM task_versions WHERE task_id = :tid;
INSERT INTO task_versions(task_id, version, project_id, name, status, effective_at)
VALUES(:tid, next_version, :pid, :name, :status, :effective_at);
COMMIT;
-- UNIQUE違反時は指数バックオフでリトライ（最大N回）
```

#### マイグレーションと運用方針（提案6）
- スキーマバージョン管理:

```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);
```

- ビュー互換: 公開ビュー（`task_current_view`, `time_entries_view`, `task_tags_current`）は破壊的変更を避け、変更時は新ビュー名で段階移行。
- データ圧縮/健全化: バージョン/イベント肥大化時はスナップショット・コンパクション（アーカイブDB移送含む）を計画的に実施。`VACUUM`, `ANALYZE`, `REINDEX` を定期運用。
- 暗号化キー更新（SQLCipher想定）: `PRAGMA rekey` を用いた再暗号化。キーはOS Keychainに保存。
- PRAGMA推奨設定:

```sql
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA foreign_keys=ON;
```

- バックアップ/復旧: アプリ終了時または日次でスナップショットバックアップ。破損時は復旧後に差分イベント再適用。
- インデックス移行: 新設後に `ANALYZE` 実行、旧インデックスは計画確認後に削除。

## 5. 代替案と検討したトレードオフ

### 検討した代替案
1.  **フレームワーク: Electron**
    - **検討理由:** Web技術の知見を活かしやすく、クロスプラットフォーム開発で最も成熟している。
    - **不採用理由:** アプリケーションのバンドルサイズが大きくなり、メモリ消費量も多い。本プロジェクトの「軽量性」という目標と合致しないため不採用とした。

2.  **データベース: PostgreSQL**
    - **検討理由:** 高機能で堅牢なリレーショナルデータベース。
    - **不採用理由:** クライアント/サーバー型のアーキテクチャであり、ローカル完結型のデスクトップアプリには過剰スペック。ユーザーにDBサーバーのインストールと管理を強いることになり、手軽さを損なうため不採用とした。

### 選定の理由
**Tauri + SQLite** の組み合わせは、パフォーマンス、アプリケーションサイズ、セットアップの容易さの観点から、本プロジェクトの要件に最も合致している。Rustの学習コストは存在するが、DDD/TDDとの親和性も高く、長期的に保守性の高いアプリケーションを構築できると判断した。

## 6. 懸念事項と考慮点

- **Rustの学習コスト:** 開発チームがRust、特に所有権やライフタイムといった概念に習熟するための時間が必要となる。
- **UIのクロスプラットフォーム互換性:** TauriはOSのWebViewを利用するため、macOSと将来的なWindows版でCSSの挙動などに差異が出る可能性がある。
- **データベースのマイグレーション:** アプリケーションのバージョンアップに伴うスキーマ変更を管理する仕組みを初期段階で導入する必要がある。

## 7. 実装とテスト計画

### 実装計画
TDDに基づき、ドメイン層のロジックから実装を開始する。
1.  **ドメイン層の実装（Rust）:** データベースに依存しない純粋なビジネスロジックをテストと共に実装。
2.  **インフラ層の実装（Rust）:** SQLiteとの接続部分を実装。
3.  **アプリケーション層の実装（Rust）:** Tauriコマンドとしてフロントエンドに公開するAPIを実装。
4.  **UI層の実装（TypeScript/React）:** バックエンドAPIを呼び出すUIコンポーネントをテストと共に実装。

### テスト方針
- **単体テスト:** `cargo test` を使用し、Rust側のビジネスロジックを網羅的にテストする。フロントエンドはVitest/Jestを使用。
- **結合テスト:** TauriのAPIを通じて、フロントエンドとバックエンドを連携させたテストを実施する。
- **E2Eテスト:** Playwrightを利用し、実際のユーザー操作をシミュレートするテストを行う。
  - Chromium、Firefox、WebKitの3つの主要ブラウザエンジンでテストを実行し、Tauriアプリケーションの動作を保証する。
  - タスクの作成、タイマーの開始/停止、レポート表示など、主要なユーザーフローを自動化テストでカバーする。
  - Trace Viewerを活用して、テスト失敗時の原因調査を効率化する。
  - CI/CDパイプラインに組み込み、リリース前の品質保証を自動化する。


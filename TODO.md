# TODO / Feature Requests

## SQLiteインポート/エクスポート機能

### 概要

現在はJSON形式のみサポートしているが、SQLiteデータベースでのインポート/エクスポート機能を追加する。
より大規模な家系図データの管理、クエリ、データベースツールとの統合を可能にする。

### 要件

#### 基本機能

1. **SQLiteエクスポート**
   - 現在のJSON形式の家系図をSQLiteデータベースに変換
   - 正規化されたテーブル設計で効率的なストレージ
   - 既存のデータベースへの追加書き込みに対応

2. **SQLiteインポート**
   - SQLiteデータベースから家系図データを読込
   - JSON形式に変換して既存システムで利用
   - 部分的なインポート（特定の家族のみ）に対応

3. **双方向同期**
   - JSONとSQLite間の相互変換
   - データの整合性チェック

#### データベーススキーマ設計

```sql
-- 人物テーブル
CREATE TABLE persons (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    gender TEXT CHECK(gender IN ('Male', 'Female', 'Unknown')),
    birth TEXT,
    death TEXT,
    deceased INTEGER DEFAULT 0,
    memo TEXT,
    position_x REAL,
    position_y REAL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 親子関係テーブル
CREATE TABLE parent_child (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id TEXT NOT NULL,
    child_id TEXT NOT NULL,
    kind TEXT DEFAULT 'biological',
    FOREIGN KEY (parent_id) REFERENCES persons(id) ON DELETE CASCADE,
    FOREIGN KEY (child_id) REFERENCES persons(id) ON DELETE CASCADE,
    UNIQUE(parent_id, child_id, kind)
);

-- 配偶者関係テーブル
CREATE TABLE spouses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    person1_id TEXT NOT NULL,
    person2_id TEXT NOT NULL,
    memo TEXT,
    FOREIGN KEY (person1_id) REFERENCES persons(id) ON DELETE CASCADE,
    FOREIGN KEY (person2_id) REFERENCES persons(id) ON DELETE CASCADE,
    CHECK(person1_id < person2_id),
    UNIQUE(person1_id, person2_id)
);

-- 家族テーブル
CREATE TABLE families (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color_r INTEGER,
    color_g INTEGER,
    color_b INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 家族メンバーテーブル
CREATE TABLE family_members (
    family_id TEXT NOT NULL,
    person_id TEXT NOT NULL,
    FOREIGN KEY (family_id) REFERENCES families(id) ON DELETE CASCADE,
    FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE CASCADE,
    PRIMARY KEY (family_id, person_id)
);

-- インデックス
CREATE INDEX idx_parent_child_parent ON parent_child(parent_id);
CREATE INDEX idx_parent_child_child ON parent_child(child_id);
CREATE INDEX idx_spouses_person1 ON spouses(person1_id);
CREATE INDEX idx_spouses_person2 ON spouses(person2_id);
CREATE INDEX idx_family_members_person ON family_members(person_id);
```

### 実装箇所

#### 新規モジュール: `src/sqlite.rs`

```rust
use rusqlite::{Connection, Result};
use crate::tree::FamilyTree;

pub struct SqliteExporter {
    conn: Connection,
}

impl SqliteExporter {
    pub fn new(db_path: &str) -> Result<Self> { ... }
    pub fn export(&self, tree: &FamilyTree) -> Result<()> { ... }
}

pub struct SqliteImporter {
    conn: Connection,
}

impl SqliteImporter {
    pub fn new(db_path: &str) -> Result<Self> { ... }
    pub fn import(&self) -> Result<FamilyTree> { ... }
}
```

#### UI追加: `src/app.rs`

- 「個人」タブに「SQLiteにエクスポート」「SQLiteからインポート」ボタン追加
- ファイル選択ダイアログ（`.db`ファイル）
- エクスポート/インポートの進行状況表示

### 依存関係追加

```toml
[dependencies]
rusqlite = { version = "0.32", features = ["bundled"] }
```

### 実装順序

1. **Phase 1**: データベーススキーマ設計とテスト
2. **Phase 2**: エクスポート機能実装（JSON → SQLite）
3. **Phase 3**: インポート機能実装（SQLite → JSON）
4. **Phase 4**: UI統合とエラーハンドリング
5. **Phase 5**: 大規模データセットでのテストと最適化

### ユースケース

1. **大規模な家系図**: 数千人規模のデータを効率的に管理
2. **データ分析**: SQLでのクエリによる統計分析
   - 例: 世代ごとの人数、平均寿命、最も多い名前など
3. **バックアップ**: 複数フォーマットでのデータバックアップ
4. **他ツールとの連携**: 家系図データをDBツールで編集・分析

### 追加機能（オプション）

- **マイグレーション**: スキーマバージョン管理
- **トランザクション**: 複数操作の原子性保証
- **全文検索**: FTS5を使った人名・メモ検索
- **バッチ操作**: 複数の家系図の一括インポート/エクスポート

### テストケース

```rust
#[test]
fn test_export_to_sqlite() {
    let mut tree = FamilyTree::default();
    let id = tree.add_person(...);
    
    let exporter = SqliteExporter::new("test.db").unwrap();
    exporter.export(&tree).unwrap();
    
    // データベース内容を検証
}

#[test]
fn test_import_from_sqlite() {
    // SQLiteからインポートしてJSON形式と一致することを確認
}

#[test]
fn test_roundtrip_json_sqlite_json() {
    // JSON → SQLite → JSON の往復で同じデータになることを確認
}
```

---

## 古代の家系図対応

### 現在の問題点

1. **紀元前の年代表示**
   - 現在: `-753-01-01` のような数値表記のみ
   - 問題: 直感的でなく、読みにくい
   
2. **年齢計算の問題**
   - 故人でない人物の年齢を現在年（2026年）との差分で計算
   - 例: 紀元前100年生まれ → "age 2126" と表示される
   - 古代の人物には適用すべきでない

3. **日付の柔軟性**
   - 年代のみ（月日なし）の入力に対応していない
   - 例: "紀元前500年" を `-500` だけで入力できない

### 実装すべき機能

#### 1. BC/AD表記オプション (優先度: 高)

**要件:**
- 紀元前の年を "BC 753" のような形式で表示
- 紀元後の年を "AD 500" または年のみで表示
- 設定で BC/AD 表記と数値表記を切り替え可能に

**実装箇所:**
- `src/layout.rs`: `person_label()` 関数に表示形式の変換ロジック追加
- `src/app.rs`: 設定タブに BC/AD 表記オン/オフのチェックボックス追加
- `src/tree.rs`: 日付のパース・フォーマット用ヘルパー関数追加

**実装例:**
```rust
// src/tree.rs に追加
pub fn format_year_display(year_str: &str, use_bc_ad: bool) -> String {
    if let Ok(year) = year_str.split('-').next().unwrap_or("").parse::<i32>() {
        if use_bc_ad {
            if year < 0 {
                format!("BC {}", -year)
            } else if year < 1000 {
                format!("AD {}", year)
            } else {
                year.to_string()
            }
        } else {
            year_str.to_string()
        }
    } else {
        year_str.to_string()
    }
}
```

#### 2. 古代人物の年齢計算無効化 (優先度: 高)

**要件:**
- 紀元前の人物や古代の人物には年齢計算を行わない
- 閾値を設定（例: 1900年以前は年齢計算しない）
- または、没年月日がある場合のみ年齢を計算

**実装箇所:**
- `src/layout.rs`: `person_label()` の `calculate_age` ロジック修正

**実装例:**
```rust
// 故人の場合のみ年齢計算
if p.deceased && p.death.is_some() {
    if let Some(age) = calculate_age(b, p.death.as_deref()) {
        label.push_str(&format!(" (died at {})", age));
    }
}
// 生存者の年齢は現代人のみ計算（1900年以降生まれ）
else if birth_year >= 1900 {
    if let Some(age) = calculate_age(b, None) {
        label.push_str(&format!(" (age {})", age));
    }
}
```

#### 3. 年代のみ入力対応 (優先度: 中)

**要件:**
- `birth` フィールドに年のみ入力可能に（例: `-500`, `1500`）
- 月日が不明な場合は省略可能
- 表示時に年のみを表示

**実装箇所:**
- `src/tree.rs`: `Person` 構造体の日付検証ロジック
- `src/layout.rs`: `person_label()` で年のみの表示形式対応
- `src/app.rs`: UI のバリデーション調整

**フォーマット例:**
- 完全形式: `1990-05-15` (YYYY-MM-DD)
- 年月形式: `1990-05` (YYYY-MM)
- 年のみ: `1990` (YYYY)
- 紀元前: `-753` (BC 753)

#### 4. 多言語対応の拡張 (優先度: 低)

**要件:**
- 日本語: 「紀元前753年」「西暦500年」
- 英語: "BC 753", "AD 500"
- 設定で言語に応じた表記を自動選択

**実装箇所:**
- `src/app.rs`: `Texts` 構造体に BC/AD の翻訳追加

### 実装順序の提案

1. **Phase 1**: 古代人物の年齢計算無効化（バグ修正）
2. **Phase 2**: BC/AD 表記オプション追加
3. **Phase 3**: 年代のみ入力対応
4. **Phase 4**: 多言語対応の拡張

### テストケース追加

```rust
#[test]
fn test_ancient_person_label() {
    let mut tree = FamilyTree::default();
    let id = tree.add_person(
        "Julius Caesar".to_string(),
        Gender::Male,
        Some("-100-07-12".to_string()), // BC 100
        "".to_string(),
        true,
        Some("-44-03-15".to_string()), // BC 44
        (0.0, 0.0),
    );
    
    let label = LayoutEngine::person_label(&tree, id);
    // BC 100 と表示され、age 2126 とは表示されないこと
    assert!(label.contains("Julius Caesar"));
    assert!(!label.contains("age 2126"));
}
```

### 参考資料

- ISO 8601 拡張形式で紀元前を表現: `-YYYY-MM-DD`
- 歴史的な日付の扱い: https://en.wikipedia.org/wiki/ISO_8601#Years

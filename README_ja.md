# 家系図クリエイター

Rustとeguiで構築された、モダンでインタラクティブな家系図可視化アプリケーションです。

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-2024-orange.svg)

[English](README.md) | 日本語

## 📸 スクリーンショット

![メインキャンバス](screenshots/main-view.png)
*4世代のサンプル家系図を表示したインタラクティブキャンバス*

![個人エディタ](screenshots/person-editor.png)
*関係管理機能を備えた個人エディタ*

![家族グループ](screenshots/family-groups.png)
*視覚的な境界線で色分けされた家族グループ*

## 🌟 機能

### コア機能
- **インタラクティブキャンバス**: ドラッグ&ドロップで家系図のノードを配置
- **パン＆ズーム**: 大きな家系図もスムーズに操作（Ctrl+ホイールでズーム）
- **視覚的な関係性**: 親、子、配偶者を明確な線で接続
- **多世代対応**: 無制限の世代数をサポート

### 人物管理
- **詳細なプロフィール**: 名前、性別、生年月日、没年月日、カスタムメモ
- **故人ステータス**: 故人には特別なマーカー（†）を表示
- **年齢計算**: 生存者と故人の年齢を自動計算
- **関係追跡**: 親（実親/養親）、子供、配偶者の管理

### 家族グループ
- **色分けグループ**: カスタムカラーで家族をグループ化
- **視覚的な境界**: 家族グループを色付き背景で表示
- **柔軟なメンバーシップ**: 一人が複数の家族に所属可能

### ユーザーインターフェース
- **バイリンガル対応**: 日本語と英語のインターフェース
- **3パネルレイアウト**: 個人、家族、設定の各タブ
- **グリッド整列**: オプションのグリッド表示とスナップ機能
- **ダーク/ライトテーマ**: システムテーマに自動追従

### データ管理
- **JSON保存**: 人間が読めるJSON形式で保存・読込
- **サンプルデータ**: 事前入力された例示用家系図でクイックスタート
- **位置記憶**: 手動で配置したノードの位置を保存

## 🚀 はじめ方

### 必要環境

- [Rust](https://www.rust-lang.org/ja/tools/install) (2024 edition以降)
- Cargo (Rustに付属)

### インストール

1. リポジトリをクローン:
```bash
git clone https://github.com/yourusername/family-tree-creator.git
cd family-tree-creator
```

2. ビルドして実行:
```bash
cargo run --release
```

### クイックスタート

1. **サンプルデータを読込**: 「サンプル」ボタンをクリックして、4世代16人の例示用家系図を読込
2. **人物を追加**: 「個人」タブで「➕ 新しい個人を追加」をクリック
3. **関係を編集**: 人物を選択し、関係コントロールで親、子、配偶者を追加
4. **ノードを配置**: キャンバス上でノードをドラッグして位置調整
5. **保存**: ファイル名を入力して「保存」をクリック

## 📖 使い方ガイド

### 家系図の作成

#### 人物の追加

1. **個人**タブに移動
2. **➕ 新しい個人を追加**をクリック
3. 詳細を入力:
   - **名前**: フルネーム
   - **性別**: 男性、女性、不明
   - **生年月日**: `YYYY-MM-DD`形式（例：`1990-05-15`）
   - **故人**: 故人の場合はチェック
   - **没年月日**: `YYYY-MM-DD`形式（故人の場合のみ）
   - **メモ**: 追加メモ
4. **更新**をクリックして保存

#### 関係の追加

1. リストから人物を選択
2. **関係を追加**セクションまでスクロール
3. 関係のタイプを選択:
   - **親を追加**: 親を選択し、関係の種類を指定（実親/養親）
   - **子を追加**: 子を選択し、関係の種類を指定
   - **配偶者を追加**: 配偶者を選択し、結婚年月日/メモを追加

#### 家族グループの作成

1. **家族**タブに移動
2. **➕ 新しい家族を追加**をクリック
3. 家族名を入力し、色を選択
4. ドロップダウンから家族メンバーを選択
5. **更新**をクリックして保存

### キャンバスの操作

- **パン**: キャンバスの空白部分をクリック＆ドラッグ
- **ズーム**: Ctrlキーを押しながらマウスホイールをスクロール
- **ノード移動**: 任意の人物ノードをクリック＆ドラッグ
- **人物選択**: ノードをクリックして選択・編集

### 設定

**設定**タブで以下を設定可能:
- **言語**: 日本語と英語を切り替え
- **グリッド**: グリッド表示の切替とグリッドサイズの調整
- **レイアウト**: すべての位置を自動計算レイアウトにリセット

## 🏗️ プロジェクト構成

```
family-tree-creator/
├── src/
│   ├── main.rs       # アプリケーションエントリーポイント
│   ├── app.rs        # メインアプリケーションロジックとUI
│   ├── tree.rs       # データモデル（人物、家族、関係）
│   └── layout.rs     # レイアウトエンジンと描画ユーティリティ
├── Cargo.toml        # プロジェクト依存関係
├── TODO.md           # 将来の機能ロードマップ
├── README.md         # 英語版README
└── README_ja.md      # このファイル
```

## 🛠️ 技術スタック

- **言語**: Rust 2024 edition
- **GUIフレームワーク**: [egui](https://github.com/emilk/egui) 0.33.3 / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) 0.33.3
- **シリアライゼーション**: [serde](https://serde.rs/) with JSON
- **ID生成**: [uuid](https://github.com/uuid-rs/uuid)

### 主な依存関係

```toml
[dependencies]
eframe = "0.33.3"
egui = "0.33.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.11", features = ["v4", "serde"] }
```

## 📊 データ形式

家系図はJSON形式で以下の構造で保存されます:

```json
{
  "persons": {
    "uuid": {
      "id": "uuid",
      "name": "山田太郎",
      "gender": "Male",
      "birth": "1990-05-15",
      "deceased": false,
      "death": null,
      "memo": "メモ",
      "position": [100.0, 200.0]
    }
  },
  "edges": [
    {
      "parent": "parent-uuid",
      "child": "child-uuid",
      "kind": "biological"
    }
  ],
  "spouses": [
    {
      "person1": "person1-uuid",
      "person2": "person2-uuid",
      "memo": "1990"
    }
  ],
  "families": [
    {
      "id": "family-uuid",
      "name": "山田家",
      "members": ["uuid1", "uuid2"],
      "color": [200, 150, 100]
    }
  ]
}
```

## 🔮 ロードマップ

計画中の機能については[TODO.md](TODO.md)を参照してください:

- 古代家系図用のBC/AD表記
- 拡張された日付入力（年のみの形式）
- GEDCOM形式のインポート/エクスポート
- 人物への写真添付
- タイムラインビュー
- 印刷用レイアウト

## 🤝 貢献

貢献を歓迎します！プルリクエストをお気軽に送信してください。

1. リポジトリをフォーク
2. フィーチャーブランチを作成 (`git checkout -b feature/AmazingFeature`)
3. 変更をコミット (`git commit -m 'Add some AmazingFeature'`)
4. ブランチにプッシュ (`git push origin feature/AmazingFeature`)
5. プルリクエストを開く

## 📝 ライセンス

このプロジェクトはMITライセンスの下でライセンスされています - 詳細はLICENSEファイルを参照してください。

## 🙏 謝辞

- [egui](https://github.com/emilk/egui) - イミディエイトモードGUIフレームワーク
- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) - egui用アプリケーションフレームワーク
- 優れたツールとライブラリを提供するRustコミュニティ

## 📧 連絡先

プロジェクトリンク: [https://github.com/yourusername/family-tree-creator](https://github.com/yourusername/family-tree-creator)

---

**注意**: これはMVP（Minimum Viable Product）版です。今後のリリースでさらなる機能を追加予定です！

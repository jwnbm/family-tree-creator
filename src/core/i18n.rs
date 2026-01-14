/// 多言語対応モジュール
/// 
/// このモジュールはアプリケーションの多言語対応を提供します。
/// 現在、日本語と英語をサポートしています。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Japanese,
    English,
}

pub struct Texts;

impl Texts {
    pub fn get(key: &str, lang: Language) -> String {
        match lang {
            Language::Japanese => Self::ja(key),
            Language::English => Self::en(key),
        }
    }
    
    fn ja(key: &str) -> String {
        match key {
            "title" => "家系図 (MVP)",
            "persons" => "👤 個人",
            "families" => "👪 家族",
            "settings" => "⚙ 設定",
            "file_menu" => "ファイル",
            "new" => "新規",
            "open" => "開く",
            "save" => "保存",
            "save_as" => "名前を付けて保存",
            "new_tree_created" => "新しい家系図を作成しました",
            "add_new_person" => "➕ 新しい個人を追加",
            "person_editor" => "個人エディタ",
            "name" => "名前:",
            "gender" => "性別:",
            "male" => "男性",
            "female" => "女性",
            "unknown" => "不明",
            "birth" => "生年月日:",
            "deceased" => "故人",
            "death" => "没年月日:",
            "memo" => "メモ:",
            "update" => "更新",
            "cancel" => "キャンセル",
            "delete" => "削除",
            "relations" => "関係:",
            "father" => "父親:",
            "mother" => "母親:",
            "parent" => "親:",
            "spouses" => "配偶者:",
            "add_relations" => "関係を追加:",
            "add_parent" => "親を追加:",
            "add_child" => "子を追加:",
            "add_spouse" => "配偶者を追加:",
            "kind" => "種類:",
            "add" => "追加",
            "select" => "(選択)",
            "view_controls" => "操作: キャンバスをドラッグでパン、Ctrl+ホイールでズーム",
            "drag_nodes" => "ノードをドラッグして位置を調整",
            "manage_families" => "家族管理",
            "add_new_family" => "➕ 新しい家族を追加",
            "family_editor" => "家族エディタ",
            "color" => "色:",
            "members" => "メンバー",
            "no_members" => "(メンバーなし)",
            "no_family_selected" => "(家族が選択されていません)",
            "add_member" => "メンバーを追加:",
            "delete_family" => "家族を削除",
            "grid" => "グリッド:",
            "show_grid" => "グリッドを表示",
            "grid_size" => "グリッドサイズ:",
            "layout" => "レイアウト:",
            "reset_positions" => "すべての位置をリセット",
            "language" => "言語:",
            "japanese" => "日本語",
            "english" => "English",
            "new_person_added" => "新しい個人を追加しました",
            "person_updated" => "個人情報を更新しました",
            "name_required" => "名前は必須です",
            "person_deleted" => "個人を削除しました",
            "relation_removed" => "関係を削除しました",
            "parent_added" => "親を追加しました",
            "child_added" => "子を追加しました",
            "spouse_added" => "配偶者を追加しました",
            "spouse_memo_updated" => "配偶者メモを更新しました",
            "edit_memo" => "メモ編集",
            "edit_kind" => "種類編集",
            "relation_kind_updated" => "関係の種類を更新しました",
            "new_family_added" => "新しい家族を追加しました",
            "member_removed" => "メンバーを削除しました",
            "member_added" => "メンバーを追加しました",
            "family_updated" => "家族情報を更新しました",
            "family_deleted" => "家族を削除しました",
            "positions_reset" => "すべての位置をリセットしました",
            "saved" => "保存しました",
            "loaded" => "読み込みました",
            "edit" => "編集:",
            "remove_relation" => "関係を削除",
            "selected_family" => "選択した家族:",
            "new_person" => "New Person",
            "new_family" => "New Family",
            "tooltip_name" => "名前",
            "tooltip_birth" => "生年月日",
            "tooltip_death" => "没年月日",
            "tooltip_age" => "歳",
            "tooltip_died_at" => "享年",
            "tooltip_deceased" => "死亡",
            "tooltip_yes" => "はい",
            "tooltip_memo" => "メモ",
            "help_menu" => "ヘルプ",
            "about" => "バージョン情報",
            "license" => "ライセンス情報",
            "app_name" => "家系図作成ツール",
            "version" => "バージョン",
            "description" => "このアプリケーションは家系図を作成・管理するためのツールです。",
            "license_text" => include_str!("../../LICENSE"),
            "close" => "閉じる",
            _ => {
                if cfg!(debug_assertions) {
                    eprintln!("[i18n Warning] Unknown translation key (ja): '{}'", key);
                }
                key
            }
        }.to_string()
    }
    
    fn en(key: &str) -> String {
        match key {
            "title" => "Family Tree (MVP)",
            "persons" => "👤 Persons",
            "families" => "👪 Families",
            "settings" => "⚙ Settings",
            "file_menu" => "File",
            "new" => "New",
            "open" => "Open",
            "save" => "Save",
            "save_as" => "Save As...",
            "new_tree_created" => "New tree created",
            "add_new_person" => "➕ Add New Person",
            "person_editor" => "Person Editor",
            "name" => "Name:",
            "gender" => "Gender:",
            "male" => "Male",
            "female" => "Female",
            "unknown" => "Unknown",
            "birth" => "Birth:",
            "deceased" => "Deceased",
            "death" => "Death:",
            "memo" => "Memo:",
            "update" => "Update",
            "cancel" => "Cancel",
            "delete" => "Delete",
            "relations" => "Relations:",
            "father" => "Father:",
            "mother" => "Mother:",
            "parent" => "Parent:",
            "spouses" => "Spouses:",
            "add_relations" => "Add Relations:",
            "add_parent" => "Add Parent:",
            "add_child" => "Add Child:",
            "add_spouse" => "Add Spouse:",
            "kind" => "Kind:",
            "add" => "Add",
            "select" => "(select)",
            "view_controls" => "View controls: Drag on canvas to pan, Ctrl+Wheel to zoom",
            "drag_nodes" => "Drag nodes to manually adjust positions",
            "manage_families" => "Manage Families",
            "add_new_family" => "➕ Add New Family",
            "family_editor" => "Family Editor",
            "color" => "Color:",
            "members" => "Members",
            "no_members" => "(No members)",
            "no_family_selected" => "(No family selected)",
            "add_member" => "Add member:",
            "delete_family" => "Delete Family",
            "grid" => "Grid:",
            "show_grid" => "Show Grid",
            "grid_size" => "Grid Size:",
            "layout" => "Layout:",
            "reset_positions" => "Reset All Positions",
            "language" => "Language:",
            "japanese" => "日本語",
            "english" => "English",
            "new_person_added" => "New person added",
            "person_updated" => "Person updated",
            "name_required" => "Name is required",
            "person_deleted" => "Person deleted",
            "relation_removed" => "Relation removed",
            "parent_added" => "Parent added",
            "child_added" => "Child added",
            "spouse_added" => "Spouse added",
            "spouse_memo_updated" => "Spouse memo updated",
            "edit_memo" => "Edit memo",
            "edit_kind" => "Edit kind",
            "relation_kind_updated" => "Relation kind updated",
            "new_family_added" => "New family added",
            "member_removed" => "Member removed",
            "member_added" => "Member added",
            "family_updated" => "Family updated",
            "family_deleted" => "Family deleted",
            "positions_reset" => "All positions reset",
            "saved" => "Saved",
            "loaded" => "Loaded",
            "edit" => "Edit:",
            "remove_relation" => "Remove relation",
            "selected_family" => "Selected family:",
            "new_person" => "New Person",
            "new_family" => "New Family",
            "tooltip_name" => "Name",
            "tooltip_birth" => "Birth",
            "tooltip_death" => "Death",
            "tooltip_age" => "years old",
            "tooltip_died_at" => "died at",
            "tooltip_deceased" => "Deceased",
            "tooltip_yes" => "Yes",
            "tooltip_memo" => "Memo",
            "help_menu" => "Help",
            "about" => "About",
            "license" => "License",
            "app_name" => "Family Tree Creator",
            "version" => "Version",
            "description" => "This application is a tool for creating and managing family trees.",
            "license_text" => include_str!("../../LICENSE"),
            "close" => "Close",
            _ => {
                if cfg!(debug_assertions) {
                    eprintln!("[i18n Warning] Unknown translation key (en): '{}'", key);
                }
                key
            }
        }.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_japanese_translation() {
        assert_eq!(Texts::get("title", Language::Japanese), "家系図 (MVP)");
        assert_eq!(Texts::get("save", Language::Japanese), "保存");
        assert_eq!(Texts::get("male", Language::Japanese), "男性");
        assert_eq!(Texts::get("female", Language::Japanese), "女性");
    }

    #[test]
    fn test_english_translation() {
        assert_eq!(Texts::get("title", Language::English), "Family Tree (MVP)");
        assert_eq!(Texts::get("save", Language::English), "Save");
        assert_eq!(Texts::get("male", Language::English), "Male");
        assert_eq!(Texts::get("female", Language::English), "Female");
    }

    #[test]
    fn test_unknown_key() {
        assert_eq!(Texts::get("nonexistent_key", Language::Japanese), "nonexistent_key");
        assert_eq!(Texts::get("nonexistent_key", Language::English), "nonexistent_key");
    }

    #[test]
    fn test_language_equality() {
        assert_eq!(Language::Japanese, Language::Japanese);
        assert_eq!(Language::English, Language::English);
        assert_ne!(Language::Japanese, Language::English);
    }

    #[test]
    fn test_all_common_keys() {
        let keys = vec!["title", "save", "persons", "families", "settings"];
        
        for key in keys {
            let ja = Texts::get(key, Language::Japanese);
            let en = Texts::get(key, Language::English);
            
            assert_ne!(ja, key, "Japanese translation missing for key: {}", key);
            assert_ne!(en, key, "English translation missing for key: {}", key);
        }
    }
}

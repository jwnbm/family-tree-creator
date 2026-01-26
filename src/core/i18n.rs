/// 多言語対応モジュール
/// 
/// このモジュールはアプリケーションの多言語対応を提供します。
/// 現在、日本語と英語をサポートしています。

use std::sync::Mutex;

static I18N_WARNINGS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// i18n警告をバッファに追加
fn add_warning(message: String) {
    if let Ok(mut warnings) = I18N_WARNINGS.lock() {
        warnings.push(message);
    }
}

/// 警告を取得してバッファをクリア
pub fn take_warnings() -> Vec<String> {
    if let Ok(mut warnings) = I18N_WARNINGS.lock() {
        std::mem::take(&mut *warnings)
    } else {
        Vec::new()
    }
}

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
            "persons" => "👤 人物",
            "families" => "👪 家族",
            "settings" => "⚙ 設定",
            "file_menu" => "ファイル",
            "new" => "新規",
            "open" => "開く",
            "save" => "保存",
            "save_as" => "名前を付けて保存",
            "new_tree_created" => "新しい家系図を作成しました",
            "add_new_person" => "➕ 新しい人物を追加",
            "person_editor" => "人物エディタ",
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
            "manage_persons" => "人物管理",
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
            "new_person_added" => "新しい人物を追加しました",
            "person_updated" => "人物情報を更新しました",
            "name_required" => "名前は必須です",
            "person_deleted" => "人物を削除しました",
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
            "app_description" => "このアプリケーションは家系図を作成・管理するためのツールです。",
            "license_text" => include_str!("../../LICENSE"),
            "close" => "閉じる",
            "events" => "📅 イベント",
            "manage_events" => "イベント管理",
            "add_new_event" => "➕ 新しいイベントを追加",
            "event_editor" => "イベントエディタ",
            "new_event" => "New Event",
            "date" => "日付:",
            "description" => "説明:",
            "event_relations" => "イベントと人物の関係:",
            "add_person_to_event" => "イベントに人物を追加:",
            "relation_type" => "線の種類:",
            "line" => "直線",
            "arrow_to_person" => "矢印 → 人物",
            "arrow_to_event" => "矢印 ← 人物",
            "new_event_added" => "新しいイベントを追加しました",
            "event_updated" => "イベント情報を更新しました",
            "event_deleted" => "イベントを削除しました",
            "relation_added" => "関係を追加しました",
            "photo_path" => "写真パス:",
            "display_mode" => "表示モード:",
            "name_only" => "名前のみ",
            "name_and_photo" => "名前と写真",
            "choose_photo" => "写真を選択...",
            "clear_photo" => "写真をクリア",
            "photo_scale" => "写真倍率:",
            // Log messages
            "log_app_started" => "アプリケーションを起動しました",
            "log_file_saved" => "ファイルを保存しました",
            "log_file_loaded" => "ファイルを読み込みました",
            "log_node_selected" => "ノードを選択",
            "log_node_deselected" => "選択を解除",
            "log_node_added_to_selection" => "追加選択",
            "log_total" => "合計",
            "log_nodes_selected" => "個のノードを選択しました",
            "log_node_drag_start" => "ノードのドラッグを開始",
            "log_nodes_moved" => "個のノードを移動完了",
            "log_distance" => "移動距離",
            "log_person_added" => "人物を追加しました",
            "log_person_deleted" => "人物を削除しました",
            "log_event_added" => "新しいイベントを追加しました",
            "log_event_updated" => "イベント情報を更新しました",
            "log_event_deleted" => "イベントを削除しました",
            "log_event_relation_added" => "イベントに人物を関連付けました",
            "log_event_relation_removed" => "イベントから関連を削除しました",
            "log_event_selected" => "イベントを選択",
            "log_event_drag_started" => "イベントノードをドラッグ開始",
            "log_event_moved" => "イベントノードを移動しました",
            "log_family_added" => "新しい家族を追加しました",
            "log_family_updated" => "家族情報を更新しました",
            "log_family_deleted" => "家族を削除しました",
            "log_family_selected" => "家族を選択",
            "log_family_member_added" => "家族にメンバーを追加しました",
            "log_family_member_removed" => "家族からメンバーを削除しました",
            "log_from" => "から",
            "log_to" => "へ",
            _ => {
                if cfg!(debug_assertions) {
                    let warning = format!("[i18n Warning] Unknown translation key (ja): '{}'", key);
                    eprintln!("{}", warning);
                    add_warning(warning);
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
            "manage_persons" => "Manage Persons",
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
            "app_description" => "This application is a tool for creating and managing family trees.",
            "license_text" => include_str!("../../LICENSE"),
            "close" => "Close",
            "events" => "📅 Events",
            "manage_events" => "Manage Events",
            "add_new_event" => "➕ Add New Event",
            "event_editor" => "Event Editor",
            "new_event" => "New Event",
            "date" => "Date:",
            "description" => "Description:",
            "event_relations" => "Event-Person Relations:",
            "add_person_to_event" => "Add Person to Event:",
            "relation_type" => "Relation Type:",
            "line" => "Line",
            "arrow_to_person" => "Arrow → Person",
            "arrow_to_event" => "Arrow ← Person",
            "new_event_added" => "New event added",
            "event_updated" => "Event updated",
            "event_deleted" => "Event deleted",
            "relation_added" => "Relation added",
            "photo_path" => "Photo Path:",
            "display_mode" => "Display Mode:",
            "name_only" => "Name Only",
            "name_and_photo" => "Name and Photo",
            "choose_photo" => "Choose Photo...",
            "clear_photo" => "Clear Photo",
            "photo_scale" => "Photo Scale:",
            // Log messages
            "log_app_started" => "Application started",
            "log_file_saved" => "File saved",
            "log_file_loaded" => "File loaded",
            "log_node_selected" => "Node selected",
            "log_node_deselected" => "Node deselected",
            "log_node_added_to_selection" => "Added to selection",
            "log_total" => "total",
            "log_nodes_selected" => "nodes selected",
            "log_node_drag_start" => "Started dragging node",
            "log_nodes_moved" => "nodes moved",
            "log_distance" => "distance",
            "log_person_added" => "Person added",
            "log_person_deleted" => "Person deleted",
            "log_event_added" => "New event added",
            "log_event_updated" => "Event updated",
            "log_event_deleted" => "Event deleted",
            "log_event_relation_added" => "Person added to event",
            "log_event_relation_removed" => "Relation removed from event",
            "log_event_selected" => "Event selected",
            "log_event_drag_started" => "Started dragging event node",
            "log_event_moved" => "Event node moved",
            "log_family_added" => "New family added",
            "log_family_updated" => "Family updated",
            "log_family_deleted" => "Family deleted",
            "log_family_selected" => "Family selected",
            "log_family_member_added" => "Member added to family",
            "log_family_member_removed" => "Member removed from family",
            "log_from" => "from",
            "log_to" => "to",
            _ => {
                if cfg!(debug_assertions) {
                    let warning = format!("[i18n Warning] Unknown translation key (en): '{}'", key);
                    eprintln!("{}", warning);
                    add_warning(warning);
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

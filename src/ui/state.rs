use eframe::egui;
use serde::{Deserialize, Serialize};
use crate::core::tree::{Gender, PersonId, EventId, EventRelationType, PersonDisplayMode};
use crate::core::i18n::Language;
use crate::infrastructure::PhotoTextureCache;
use uuid::Uuid;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// ログレベル
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Critical,
    Error,
    Warning,
    Information,
    Debug,
}

impl LogLevel {
    pub fn as_str(&self) -> &str {
        match self {
            LogLevel::Critical => "CRITICAL",
            LogLevel::Error => "ERROR",
            LogLevel::Warning => "WARNING",
            LogLevel::Information => "INFO",
            LogLevel::Debug => "DEBUG",
        }
    }
    
    pub fn color(&self) -> egui::Color32 {
        match self {
            LogLevel::Critical => egui::Color32::from_rgb(255, 0, 0),      // 赤
            LogLevel::Error => egui::Color32::from_rgb(255, 100, 100),     // オレンジ赤
            LogLevel::Warning => egui::Color32::from_rgb(255, 165, 0),     // オレンジ
            LogLevel::Information => egui::Color32::from_rgb(100, 150, 255), // 青
            LogLevel::Debug => egui::Color32::from_rgb(128, 128, 128),     // 灰色
        }
    }
}

/// ログメッセージ
#[derive(Clone)]
pub struct LogMessage {
    pub message: String,
    pub timestamp: String,
    pub level: LogLevel,
}

/// ログ状態
pub struct LogState {
    pub messages: Vec<LogMessage>,
    pub max_messages: usize,
    pub log_file_path: Option<PathBuf>,
}

impl Default for LogState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            max_messages: 100,
            log_file_path: None,
        }
    }
}

impl LogState {
    pub fn add(&mut self, message: String) {
        self.add_with_level(message, LogLevel::Information);
    }
    
    pub fn add_with_level(&mut self, message: String, level: LogLevel) {
        let now = chrono::Local::now();
        let timestamp = now.format("%H:%M:%S").to_string();
        
        self.messages.push(LogMessage {
            message: message.clone(),
            timestamp: timestamp.clone(),
            level,
        });
        
        // ファイルに出力
        self.write_to_file(&timestamp, level, &message);
        
        // 最大数を超えた場合は古いものから削除
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }
    
    pub fn clear(&mut self) {
        self.messages.clear();
    }
    
    /// ログファイルパスを設定
    pub fn set_log_file(&mut self, log_dir: &str) -> std::io::Result<()> {
        // logディレクトリを作成
        fs::create_dir_all(log_dir)?;
        
        // ログファイル名（日時を含む）
        let now = chrono::Local::now();
        let filename = format!("{}.log", now.format("%Y%m%d_%H%M%S"));
        let log_path = PathBuf::from(log_dir).join(filename);
        
        self.log_file_path = Some(log_path);
        Ok(())
    }
    
    /// ログをファイルに書き込み
    fn write_to_file(&self, timestamp: &str, level: LogLevel, message: &str) {
        if let Some(path) = &self.log_file_path {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let _ = writeln!(file, "[{}] [{}] {}", timestamp, level.as_str(), message);
            }
        }
    }
}

/// 人物編集フォームの状態
#[derive(Default)]
pub struct PersonEditorState {
    pub selected: Option<PersonId>,
    /// 複数選択されたノードのID（選択順序を保持）
    pub selected_ids: Vec<PersonId>,
    pub new_name: String,
    pub new_gender: Gender,
    pub new_birth: String,
    pub new_memo: String,
    pub new_deceased: bool,
    pub new_death: String,
    pub new_photo_path: String,
    pub new_display_mode: PersonDisplayMode,
    pub new_photo_scale: f32,
}

impl PersonEditorState {
    pub fn clear(&mut self) {
        self.new_name.clear();
        self.new_gender = Gender::Unknown;
        self.new_birth.clear();
        self.new_memo.clear();
        self.new_deceased = false;
        self.new_death.clear();
        self.new_photo_path.clear();
        self.new_display_mode = PersonDisplayMode::NameOnly;
        self.new_photo_scale = 1.0;
    }
}

/// 関係編集フォームの状態
#[derive(Default)]
pub struct RelationEditorState {
    // 親子関係追加
    pub parent_pick: Option<PersonId>,
    pub child_pick: Option<PersonId>,
    pub relation_kind: String,
    
    // 配偶者関係追加
    pub spouse_pick: Option<PersonId>,
    pub spouse_memo: String,
    
    // 配偶者メモ編集
    pub editing_spouse_memo: Option<(PersonId, PersonId)>,
    pub temp_spouse_memo: String,
    
    // 親子関係の種類編集
    pub editing_parent_kind: Option<(PersonId, PersonId)>,
    pub temp_kind: String,
}

impl RelationEditorState {
    pub fn new() -> Self {
        Self {
            relation_kind: "biological".to_string(),
            ..Default::default()
        }
    }
}

/// 家族管理の状態
#[derive(Default)]
pub struct FamilyEditorState {
    pub selected_family: Option<Uuid>,
    pub new_family_name: String,
    pub new_family_color: [f32; 3],
    pub family_member_pick: Option<PersonId>,
}

impl FamilyEditorState {
    pub fn new() -> Self {
        Self {
            new_family_color: [0.8, 0.8, 1.0],
            ..Default::default()
        }
    }
}

/// イベント管理の状態
#[derive(Default)]
pub struct EventEditorState {
    pub selected: Option<EventId>,
    pub new_event_name: String,
    pub new_event_date: String,
    pub new_event_description: String,
    pub new_event_color: [f32; 3],
    
    // イベントと人物の関係追加
    pub person_pick: Option<PersonId>,
    pub relation_type: EventRelationType,
    pub relation_memo: String,
}

impl EventEditorState {
    pub fn clear(&mut self) {
        self.new_event_name.clear();
        self.new_event_date.clear();
        self.new_event_description.clear();
        self.new_event_color = [1.0, 1.0, 0.8]; // デフォルトの淡い黄色
    }
}

/// キャンバスの表示・操作状態
pub struct CanvasState {
    // 表示
    pub zoom: f32,
    pub pan: egui::Vec2,
    pub dragging_pan: bool,
    pub last_pointer_pos: Option<egui::Pos2>,
    
    // ノードドラッグ
    pub dragging_node: Option<PersonId>,
    pub node_drag_start: Option<egui::Pos2>,
    /// 複数選択されたノードのドラッグ開始位置（各ノードごと）
    pub multi_drag_starts: std::collections::HashMap<PersonId, (f32, f32)>,
    
    // イベントノードドラッグ
    pub dragging_event: Option<EventId>,
    pub event_drag_start: Option<egui::Pos2>,
    
    // グリッド
    pub show_grid: bool,
    pub grid_size: f32,
    
    // キャンバス情報
    pub canvas_rect: egui::Rect,
    pub canvas_origin: egui::Pos2,

    // 写真テクスチャキャッシュ
    pub photo_texture_cache: PhotoTextureCache,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            dragging_pan: false,
            last_pointer_pos: None,
            dragging_node: None,
            node_drag_start: None,
            multi_drag_starts: std::collections::HashMap::new(),
            dragging_event: None,
            event_drag_start: None,
            show_grid: true,
            grid_size: 50.0,
            canvas_rect: egui::Rect::NOTHING,
            canvas_origin: egui::Pos2::ZERO,
            photo_texture_cache: PhotoTextureCache::default(),
        }
    }
}

/// ファイル操作の状態
#[derive(Default)]
pub struct FileState {
    pub file_path: String,
    pub status: String,
}

impl FileState {
    pub fn new() -> Self {
        Self {
            file_path: String::new(),
            status: String::new(),
        }
    }
}

/// UI全般の状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideTab {
    Persons,
    Families,
    Events,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeColorThemePreset {
    Default,
    HighContrast,
}

pub struct UiState {
    pub side_tab: SideTab,
    pub language: Language,
    pub node_color_theme: NodeColorThemePreset,
    pub show_about_dialog: bool,
    pub show_license_dialog: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            side_tab: SideTab::Persons,
            language: Language::Japanese,
            node_color_theme: NodeColorThemePreset::Default,
            show_about_dialog: false,
            show_license_dialog: false,
        }
    }
}

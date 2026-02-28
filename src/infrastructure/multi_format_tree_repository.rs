use std::path::Path;

use crate::application::{TreeRepository, TreeRepositoryError};
use crate::core::tree::FamilyTree;

use super::json_tree_repository::JsonTreeRepository;
use super::sqlite_tree_repository::SqliteTreeRepository;

/// ファイル拡張子に応じてJSON/SQLiteを切り替えるリポジトリ。
pub struct MultiFormatTreeRepository {
    json_repository: JsonTreeRepository,
    sqlite_repository: SqliteTreeRepository,
}

impl MultiFormatTreeRepository {
    /// マルチフォーマット対応リポジトリを生成する。
    pub fn new() -> Self {
        Self {
            json_repository: JsonTreeRepository,
            sqlite_repository: SqliteTreeRepository,
        }
    }

    fn detect_format(file_path: &str) -> StorageFormat {
        let extension = Path::new(file_path)
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase());

        match extension.as_deref() {
            Some("db") | Some("sqlite") => StorageFormat::Sqlite,
            _ => StorageFormat::Json,
        }
    }
}

impl Default for MultiFormatTreeRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeRepository for MultiFormatTreeRepository {
    fn load(&self, file_path: &str) -> Result<FamilyTree, TreeRepositoryError> {
        match Self::detect_format(file_path) {
            StorageFormat::Json => self.json_repository.load(file_path),
            StorageFormat::Sqlite => self.sqlite_repository.load(file_path),
        }
    }

    fn save(&self, file_path: &str, tree: &FamilyTree) -> Result<(), TreeRepositoryError> {
        match Self::detect_format(file_path) {
            StorageFormat::Json => self.json_repository.save(file_path, tree),
            StorageFormat::Sqlite => self.sqlite_repository.save(file_path, tree),
        }
    }
}

enum StorageFormat {
    Json,
    Sqlite,
}

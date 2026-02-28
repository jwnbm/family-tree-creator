use std::error::Error;
use std::fmt;

use crate::core::tree::FamilyTree;

/// 永続化レイヤから返されるエラーを表す。
#[derive(Debug)]
pub enum TreeRepositoryError {
    Read(String),
    Write(String),
    Serialize(String),
    Deserialize(String),
}

impl fmt::Display for TreeRepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TreeRepositoryError::Read(message) => write!(f, "Read error: {message}"),
            TreeRepositoryError::Write(message) => write!(f, "Write error: {message}"),
            TreeRepositoryError::Serialize(message) => write!(f, "Serialize error: {message}"),
            TreeRepositoryError::Deserialize(message) => write!(f, "Parse error: {message}"),
        }
    }
}

impl Error for TreeRepositoryError {}

/// 家系図データの入出力を抽象化するリポジトリ。
pub trait TreeRepository {
    /// 指定パスから家系図を読み込む。
    fn load(&self, file_path: &str) -> Result<FamilyTree, TreeRepositoryError>;

    /// 指定パスへ家系図を書き込む。
    fn save(&self, file_path: &str, tree: &FamilyTree) -> Result<(), TreeRepositoryError>;
}
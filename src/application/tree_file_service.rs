use crate::application::tree_repository::{TreeRepository, TreeRepositoryError};
use crate::core::tree::FamilyTree;

/// 家系図ファイルの保存・読込ユースケースを提供するアプリケーションサービス。
pub struct TreeFileService<R: TreeRepository> {
    repository: R,
}

impl<R: TreeRepository> TreeFileService<R> {
    /// リポジトリ実装を受け取りサービスを生成する。
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// 指定パスの家系図を読み込む。
    pub fn load_tree(&self, file_path: &str) -> Result<FamilyTree, TreeRepositoryError> {
        self.repository.load(file_path)
    }

    /// 指定パスへ家系図を保存する。
    pub fn save_tree(&self, file_path: &str, tree: &FamilyTree) -> Result<(), TreeRepositoryError> {
        self.repository.save(file_path, tree)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use super::TreeFileService;
    use crate::application::tree_repository::{TreeRepository, TreeRepositoryError};
    use crate::core::tree::FamilyTree;

    struct InMemoryTreeRepository {
        store: RefCell<HashMap<String, FamilyTree>>,
    }

    impl InMemoryTreeRepository {
        fn new() -> Self {
            Self {
                store: RefCell::new(HashMap::new()),
            }
        }
    }

    impl TreeRepository for InMemoryTreeRepository {
        fn load(&self, file_path: &str) -> Result<FamilyTree, TreeRepositoryError> {
            self.store
                .borrow()
                .get(file_path)
                .cloned()
                .ok_or_else(|| TreeRepositoryError::Read("not found".to_string()))
        }

        fn save(&self, file_path: &str, tree: &FamilyTree) -> Result<(), TreeRepositoryError> {
            self.store
                .borrow_mut()
                .insert(file_path.to_string(), tree.clone());
            Ok(())
        }
    }

    #[test]
    fn save_then_load_returns_same_tree() {
        let repository = InMemoryTreeRepository::new();
        let service = TreeFileService::new(repository);
        let tree = FamilyTree::default();

        let save_result = service.save_tree("test.json", &tree);
        assert!(save_result.is_ok());

        let loaded_tree_result = service.load_tree("test.json");
        assert!(loaded_tree_result.is_ok());
        let loaded_tree = loaded_tree_result.expect("saved tree should be readable");
        assert_eq!(loaded_tree.persons.len(), 0);
        assert_eq!(loaded_tree.events.len(), 0);
    }
}
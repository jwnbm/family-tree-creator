use std::fs;

use crate::application::{TreeRepository, TreeRepositoryError};
use crate::core::tree::FamilyTree;

/// `FamilyTree`をJSONファイルとして保存・読込するリポジトリ実装。
pub struct JsonTreeRepository;

impl TreeRepository for JsonTreeRepository {
    fn load(&self, file_path: &str) -> Result<FamilyTree, TreeRepositoryError> {
        let content = fs::read_to_string(file_path)
            .map_err(|error| TreeRepositoryError::Read(error.to_string()))?;

        serde_json::from_str::<FamilyTree>(&content)
            .map_err(|error| TreeRepositoryError::Deserialize(error.to_string()))
    }

    fn save(&self, file_path: &str, tree: &FamilyTree) -> Result<(), TreeRepositoryError> {
        let serialized = serde_json::to_string_pretty(tree)
            .map_err(|error| TreeRepositoryError::Serialize(error.to_string()))?;

        fs::write(file_path, serialized)
            .map_err(|error| TreeRepositoryError::Write(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;

    use uuid::Uuid;

    use super::JsonTreeRepository;
    use crate::application::TreeRepository;
    use crate::core::tree::FamilyTree;

    #[test]
    fn save_and_load_round_trip() {
        let repository = JsonTreeRepository;
        let file_name = format!("family_tree_test_{}.json", Uuid::new_v4());
        let file_path = env::temp_dir().join(file_name);
        let file_path_str = file_path.to_string_lossy().to_string();
        let tree = FamilyTree::default();

        let save_result = repository.save(&file_path_str, &tree);
        assert!(save_result.is_ok());

        let loaded_tree_result = repository.load(&file_path_str);
        assert!(loaded_tree_result.is_ok());
        let loaded_tree = loaded_tree_result.expect("json file should load");
        assert_eq!(loaded_tree.persons.len(), 0);

        let remove_result = fs::remove_file(file_path);
        assert!(remove_result.is_ok());
    }
}
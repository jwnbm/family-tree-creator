use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type PersonId = Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Gender {
    Male,
    Female,
    Unknown,
}

impl Default for Gender {
    fn default() -> Self {
        Gender::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: PersonId,
    pub name: String,
    #[serde(default)]
    pub gender: Gender,
    pub birth: Option<String>, // "YYYY-MM-DD" など
    pub memo: String,
    #[serde(default)]
    pub manual_offset: Option<(f32, f32)>, // 手動配置のオフセット
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentChild {
    pub parent: PersonId,
    pub child: PersonId,
    pub kind: String, // "biological" / "adoptive" 等、今は自由文字列
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spouse {
    pub person1: PersonId,
    pub person2: PersonId,
    pub memo: String, // 結婚年月日などのメモ
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FamilyTree {
    pub persons: HashMap<PersonId, Person>,
    pub edges: Vec<ParentChild>,
    #[serde(default)]
    pub spouses: Vec<Spouse>,
}

impl FamilyTree {
    pub fn add_person(&mut self, name: String, gender: Gender, birth: Option<String>, memo: String) -> PersonId {
        let id = Uuid::new_v4();
        self.persons.insert(
            id,
            Person {
                id,
                name,
                gender,
                birth,
                memo,
                manual_offset: None,
            },
        );
        id
    }

    pub fn remove_person(&mut self, id: PersonId) {
        self.persons.remove(&id);
        self.edges.retain(|e| e.parent != id && e.child != id);
        self.spouses.retain(|s| s.person1 != id && s.person2 != id);
    }

    pub fn add_parent_child(&mut self, parent: PersonId, child: PersonId, kind: String) {
        // 重複エッジ防止（同じ親子・同じkindなら追加しない）
        if self
            .edges
            .iter()
            .any(|e| e.parent == parent && e.child == child && e.kind == kind)
        {
            return;
        }
        self.edges.push(ParentChild { parent, child, kind });
    }

    pub fn add_spouse(&mut self, person1: PersonId, person2: PersonId, memo: String) {
        // 重複防止（順序に関わらず同じペアなら追加しない）
        if self.spouses.iter().any(|s| {
            (s.person1 == person1 && s.person2 == person2)
                || (s.person1 == person2 && s.person2 == person1)
        }) {
            return;
        }
        self.spouses.push(Spouse {
            person1,
            person2,
            memo,
        });
    }

    pub fn remove_parent_child(&mut self, parent: PersonId, child: PersonId) {
        self.edges.retain(|e| !(e.parent == parent && e.child == child));
    }

    pub fn remove_spouse(&mut self, person1: PersonId, person2: PersonId) {
        self.spouses.retain(|s| {
            !((s.person1 == person1 && s.person2 == person2)
                || (s.person1 == person2 && s.person2 == person1))
        });
    }

    pub fn parents_of(&self, child: PersonId) -> Vec<PersonId> {
        self.edges
            .iter()
            .filter(|e| e.child == child)
            .map(|e| e.parent)
            .collect()
    }

    pub fn children_of(&self, parent: PersonId) -> Vec<PersonId> {
        self.edges
            .iter()
            .filter(|e| e.parent == parent)
            .map(|e| e.child)
            .collect()
    }

    pub fn spouses_of(&self, person: PersonId) -> Vec<PersonId> {
        self.spouses
            .iter()
            .filter_map(|s| {
                if s.person1 == person {
                    Some(s.person2)
                } else if s.person2 == person {
                    Some(s.person1)
                } else {
                    None
                }
            })
            .collect()
    }

    /// ルート（親がいない人物）を返す
    pub fn roots(&self) -> Vec<PersonId> {
        let mut has_parent = HashMap::<PersonId, bool>::new();
        for id in self.persons.keys() {
            has_parent.insert(*id, false);
        }
        for e in &self.edges {
            has_parent.insert(e.child, true);
        }
        has_parent
            .into_iter()
            .filter_map(|(id, hp)| (!hp).then_some(id))
            .collect()
    }
}

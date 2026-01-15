use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type PersonId = Uuid;
pub type EventId = Uuid;

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
    pub position: (f32, f32), // 手動配置の座標（左上）
    #[serde(default)]
    pub deceased: bool, // 死亡フラグ
    #[serde(default)]
    pub death: Option<String>, // 死亡年月日 "YYYY-MM-DD" など
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Family {
    pub id: Uuid,
    pub name: String,
    pub members: Vec<PersonId>,
    pub color: Option<(u8, u8, u8)>, // RGB色
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub name: String,
    pub date: Option<String>, // "YYYY-MM-DD" など
    pub description: String,
    #[serde(default)]
    pub position: (f32, f32), // 手動配置の座標（左上）
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EventRelationType {
    Line,   // 直線
    Arrow,  // 矢印
}

impl Default for EventRelationType {
    fn default() -> Self {
        EventRelationType::Line
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRelation {
    pub event: EventId,
    pub person: PersonId,
    #[serde(default)]
    pub relation_type: EventRelationType,
    pub memo: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FamilyTree {
    pub persons: HashMap<PersonId, Person>,
    pub edges: Vec<ParentChild>,
    #[serde(default)]
    pub spouses: Vec<Spouse>,
    #[serde(default)]
    pub families: Vec<Family>,
    #[serde(default)]
    pub events: HashMap<EventId, Event>,
    #[serde(default)]
    pub event_relations: Vec<EventRelation>,
}

impl FamilyTree {
    pub fn add_person(&mut self, name: String, gender: Gender, birth: Option<String>, memo: String, deceased: bool, death: Option<String>, position: (f32, f32)) -> PersonId {
        let id = Uuid::new_v4();
        self.persons.insert(
            id,
            Person {
                id,
                name,
                gender,
                birth,
                memo,
                position,
                deceased,
                death,
            },
        );
        id
    }

    pub fn remove_person(&mut self, id: PersonId) {
        self.persons.remove(&id);
        self.edges.retain(|e| e.parent != id && e.child != id);
        self.spouses.retain(|s| s.person1 != id && s.person2 != id);
        
        // 家族グループからも削除
        for family in &mut self.families {
            family.members.retain(|member_id| *member_id != id);
        }
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

    // ===== 家族操作メソッド =====

    pub fn add_family(&mut self, name: String, color: Option<(u8, u8, u8)>) -> Uuid {
        let family = Family {
            id: Uuid::new_v4(),
            name,
            members: Vec::new(),
            color,
        };
        let id = family.id;
        self.families.push(family);
        id
    }

    pub fn remove_family(&mut self, family_id: Uuid) {
        self.families.retain(|f| f.id != family_id);
    }

    pub fn add_member_to_family(&mut self, family_id: Uuid, person_id: PersonId) {
        if let Some(family) = self.families.iter_mut().find(|f| f.id == family_id) {
            if !family.members.contains(&person_id) {
                family.members.push(person_id);
            }
        }
    }

    // ===== イベント操作メソッド =====

    pub fn add_event(&mut self, name: String, date: Option<String>, description: String, position: (f32, f32)) -> EventId {
        let id = Uuid::new_v4();
        self.events.insert(
            id,
            Event {
                id,
                name,
                date,
                description,
                position,
            },
        );
        id
    }

    pub fn remove_event(&mut self, id: EventId) {
        self.events.remove(&id);
        self.event_relations.retain(|r| r.event != id);
    }

    pub fn add_event_relation(&mut self, event: EventId, person: PersonId, relation_type: EventRelationType, memo: String) {
        // 重複防止
        if self.event_relations.iter().any(|r| r.event == event && r.person == person) {
            return;
        }
        self.event_relations.push(EventRelation {
            event,
            person,
            relation_type,
            memo,
        });
    }

    pub fn remove_event_relation(&mut self, event: EventId, person: PersonId) {
        self.event_relations.retain(|r| !(r.event == event && r.person == person));
    }

    pub fn event_relations_of(&self, event: EventId) -> Vec<&EventRelation> {
        self.event_relations
            .iter()
            .filter(|r| r.event == event)
            .collect()
    }

    pub fn remove_member_from_family(&mut self, family_id: Uuid, person_id: PersonId) {
        if let Some(family) = self.families.iter_mut().find(|f| f.id == family_id) {
            family.members.retain(|&id| id != person_id);
        }
    }

    #[allow(dead_code)]
    pub fn update_family_name(&mut self, family_id: Uuid, name: String) {
        if let Some(family) = self.families.iter_mut().find(|f| f.id == family_id) {
            family.name = name;
        }
    }

    #[allow(dead_code)]
    pub fn update_family_color(&mut self, family_id: Uuid, color: Option<(u8, u8, u8)>) {
        if let Some(family) = self.families.iter_mut().find(|f| f.id == family_id) {
            family.color = color;
        }
    }

    #[allow(dead_code)]
    pub fn get_family(&self, family_id: Uuid) -> Option<&Family> {
        self.families.iter().find(|f| f.id == family_id)
    }

    #[allow(dead_code)]
    pub fn get_families_containing(&self, person_id: PersonId) -> Vec<&Family> {
        self.families
            .iter()
            .filter(|f| f.members.contains(&person_id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_person() {
        let mut tree = FamilyTree::default();
        let id = tree.add_person(
            "Test Person".to_string(),
            Gender::Male,
            Some("2000-01-01".to_string()),
            "Test memo".to_string(),
            false,
            None,
            (100.0, 50.0),
        );

        assert_eq!(tree.persons.len(), 1);
        let person = tree.persons.get(&id).unwrap();
        assert_eq!(person.name, "Test Person");
        assert_eq!(person.gender, Gender::Male);
        assert_eq!(person.birth, Some("2000-01-01".to_string()));
        assert_eq!(person.memo, "Test memo");
        assert_eq!(person.deceased, false);
        assert_eq!(person.death, None);
    }

    #[test]
    fn test_remove_person() {
        let mut tree = FamilyTree::default();
        let parent = tree.add_person("Parent".to_string(), Gender::Female, None, "".to_string(), false, None, (0.0, 0.0));
        let child = tree.add_person("Child".to_string(), Gender::Male, None, "".to_string(), false, None, (0.0, 100.0));
        let spouse = tree.add_person("Spouse".to_string(), Gender::Male, None, "".to_string(), false, None, (200.0, 0.0));

        tree.add_parent_child(parent, child, "biological".to_string());
        tree.add_spouse(parent, spouse, "".to_string());

        tree.remove_person(parent);

        assert_eq!(tree.persons.len(), 2);
        assert!(tree.persons.get(&parent).is_none());
        assert_eq!(tree.edges.len(), 0);
        assert_eq!(tree.spouses.len(), 0);
    }

    #[test]
    fn test_add_parent_child() {
        let mut tree = FamilyTree::default();
        let parent = tree.add_person("Parent".to_string(), Gender::Female, None, "".to_string(), false, None, (0.0, 0.0));
        let child = tree.add_person("Child".to_string(), Gender::Male, None, "".to_string(), false, None, (0.0, 100.0));

        tree.add_parent_child(parent, child, "biological".to_string());
        assert_eq!(tree.edges.len(), 1);

        // 重複追加は無視される
        tree.add_parent_child(parent, child, "biological".to_string());
        assert_eq!(tree.edges.len(), 1);

        // 異なるkindなら追加される
        tree.add_parent_child(parent, child, "adoptive".to_string());
        assert_eq!(tree.edges.len(), 2);
    }

    #[test]
    fn test_remove_parent_child() {
        let mut tree = FamilyTree::default();
        let parent = tree.add_person("Parent".to_string(), Gender::Female, None, "".to_string(), false, None, (0.0, 0.0));
        let child = tree.add_person("Child".to_string(), Gender::Male, None, "".to_string(), false, None, (0.0, 100.0));

        tree.add_parent_child(parent, child, "biological".to_string());
        assert_eq!(tree.edges.len(), 1);

        tree.remove_parent_child(parent, child);
        assert_eq!(tree.edges.len(), 0);
    }

    #[test]
    fn test_add_spouse() {
        let mut tree = FamilyTree::default();
        let person1 = tree.add_person("Person1".to_string(), Gender::Male, None, "".to_string(), false, None, (0.0, 0.0));
        let person2 = tree.add_person("Person2".to_string(), Gender::Female, None, "".to_string(), false, None, (200.0, 0.0));

        tree.add_spouse(person1, person2, "1990".to_string());
        assert_eq!(tree.spouses.len(), 1);

        // 重複追加は無視される
        tree.add_spouse(person1, person2, "1990".to_string());
        assert_eq!(tree.spouses.len(), 1);

        // 順序を入れ替えても重複と見なされる
        tree.add_spouse(person2, person1, "1990".to_string());
        assert_eq!(tree.spouses.len(), 1);
    }

    #[test]
    fn test_remove_spouse() {
        let mut tree = FamilyTree::default();
        let person1 = tree.add_person("Person1".to_string(), Gender::Male, None, "".to_string(), false, None, (0.0, 0.0));
        let person2 = tree.add_person("Person2".to_string(), Gender::Female, None, "".to_string(), false, None, (200.0, 0.0));

        tree.add_spouse(person1, person2, "1990".to_string());
        assert_eq!(tree.spouses.len(), 1);

        tree.remove_spouse(person1, person2);
        assert_eq!(tree.spouses.len(), 0);

        // 再度追加して順序を逆にして削除
        tree.add_spouse(person1, person2, "1990".to_string());
        tree.remove_spouse(person2, person1);
        assert_eq!(tree.spouses.len(), 0);
    }

    #[test]
    fn test_parents_of() {
        let mut tree = FamilyTree::default();
        let father = tree.add_person("Father".to_string(), Gender::Male, None, "".to_string(), false, None, (0.0, 0.0));
        let mother = tree.add_person("Mother".to_string(), Gender::Female, None, "".to_string(), false, None, (200.0, 0.0));
        let child = tree.add_person("Child".to_string(), Gender::Unknown, None, "".to_string(), false, None, (100.0, 100.0));

        tree.add_parent_child(father, child, "biological".to_string());
        tree.add_parent_child(mother, child, "biological".to_string());

        let parents = tree.parents_of(child);
        assert_eq!(parents.len(), 2);
        assert!(parents.contains(&father));
        assert!(parents.contains(&mother));
    }

    #[test]
    fn test_children_of() {
        let mut tree = FamilyTree::default();
        let parent = tree.add_person("Parent".to_string(), Gender::Female, None, "".to_string(), false, None, (0.0, 0.0));
        let child1 = tree.add_person("Child1".to_string(), Gender::Male, None, "".to_string(), false, None, (0.0, 100.0));
        let child2 = tree.add_person("Child2".to_string(), Gender::Female, None, "".to_string(), false, None, (200.0, 100.0));

        tree.add_parent_child(parent, child1, "biological".to_string());
        tree.add_parent_child(parent, child2, "biological".to_string());

        let children = tree.children_of(parent);
        assert_eq!(children.len(), 2);
        assert!(children.contains(&child1));
        assert!(children.contains(&child2));
    }

    #[test]
    fn test_spouses_of() {
        let mut tree = FamilyTree::default();
        let person1 = tree.add_person("Person1".to_string(), Gender::Male, None, "".to_string(), false, None, (0.0, 0.0));
        let person2 = tree.add_person("Person2".to_string(), Gender::Female, None, "".to_string(), false, None, (200.0, 0.0));
        let person3 = tree.add_person("Person3".to_string(), Gender::Female, None, "".to_string(), false, None, (400.0, 0.0));

        tree.add_spouse(person1, person2, "1990".to_string());
        tree.add_spouse(person1, person3, "2000".to_string());

        let spouses = tree.spouses_of(person1);
        assert_eq!(spouses.len(), 2);
        assert!(spouses.contains(&person2));
        assert!(spouses.contains(&person3));

        let spouses2 = tree.spouses_of(person2);
        assert_eq!(spouses2.len(), 1);
        assert!(spouses2.contains(&person1));
    }

    #[test]
    fn test_roots() {
        let mut tree = FamilyTree::default();
        let grandparent = tree.add_person("Grandparent".to_string(), Gender::Female, None, "".to_string(), false, None, (0.0, 0.0));
        let parent = tree.add_person("Parent".to_string(), Gender::Male, None, "".to_string(), false, None, (0.0, 100.0));
        let child = tree.add_person("Child".to_string(), Gender::Unknown, None, "".to_string(), false, None, (0.0, 200.0));
        let orphan = tree.add_person("Orphan".to_string(), Gender::Unknown, None, "".to_string(), false, None, (300.0, 0.0));

        tree.add_parent_child(grandparent, parent, "biological".to_string());
        tree.add_parent_child(parent, child, "biological".to_string());

        let roots = tree.roots();
        assert_eq!(roots.len(), 2);
        assert!(roots.contains(&grandparent));
        assert!(roots.contains(&orphan));
        assert!(!roots.contains(&parent));
        assert!(!roots.contains(&child));
    }

    #[test]
    fn test_family_management() {
        let mut tree = FamilyTree::default();
        let person1 = tree.add_person("Father".to_string(), Gender::Male, None, "".to_string(), false, None, (0.0, 0.0));
        let person2 = tree.add_person("Mother".to_string(), Gender::Female, None, "".to_string(), false, None, (200.0, 0.0));
        let person3 = tree.add_person("Child".to_string(), Gender::Unknown, None, "".to_string(), false, None, (100.0, 100.0));

        // 家族グループを追加
        let family_id = tree.add_family("Test Family".to_string(), Some((100, 150, 200)));
        assert_eq!(tree.families.len(), 1);
        assert_eq!(tree.families[0].name, "Test Family");
        assert_eq!(tree.families[0].color, Some((100, 150, 200)));

        // メンバーを追加
        tree.add_member_to_family(family_id, person1);
        tree.add_member_to_family(family_id, person2);
        tree.add_member_to_family(family_id, person3);
        
        let family = tree.families.iter().find(|f| f.id == family_id).unwrap();
        assert_eq!(family.members.len(), 3);
        assert!(family.members.contains(&person1));
        assert!(family.members.contains(&person2));
        assert!(family.members.contains(&person3));

        // メンバーを削除
        tree.remove_member_from_family(family_id, person3);
        let family = tree.families.iter().find(|f| f.id == family_id).unwrap();
        assert_eq!(family.members.len(), 2);
        assert!(!family.members.contains(&person3));

        // 家族グループを削除
        tree.remove_family(family_id);
        assert_eq!(tree.families.len(), 0);
    }

    #[test]
    fn test_remove_person_updates_families() {
        let mut tree = FamilyTree::default();
        let person1 = tree.add_person("Person1".to_string(), Gender::Male, None, "".to_string(), false, None, (0.0, 0.0));
        let person2 = tree.add_person("Person2".to_string(), Gender::Female, None, "".to_string(), false, None, (200.0, 0.0));
        
        let family_id = tree.add_family("Family".to_string(), None);
        tree.add_member_to_family(family_id, person1);
        tree.add_member_to_family(family_id, person2);
        
        // 人物を削除すると家族からも削除される
        tree.remove_person(person1);
        
        let family = tree.families.iter().find(|f| f.id == family_id).unwrap();
        assert_eq!(family.members.len(), 1);
        assert!(!family.members.contains(&person1));
        assert!(family.members.contains(&person2));
    }

    #[test]
    fn test_deceased_flag() {
        let mut tree = FamilyTree::default();
        let person = tree.add_person(
            "Test Person".to_string(),
            Gender::Male,
            Some("1950-01-01".to_string()),
            "Test memo".to_string(),
            true,
            Some("2020-12-31".to_string()),
            (0.0, 0.0)
        );

        let p = tree.persons.get(&person).unwrap();
        assert!(p.deceased);
        assert_eq!(p.death, Some("2020-12-31".to_string()));
        assert_eq!(p.birth, Some("1950-01-01".to_string()));
    }

    #[test]
    fn test_position_persistence() {
        let mut tree = FamilyTree::default();
        let person = tree.add_person(
            "Test".to_string(),
            Gender::Unknown,
            None,
            "".to_string(),
            false,
            None,
            (123.45, 678.90)
        );

        let p = tree.persons.get(&person).unwrap();
        assert_eq!(p.position, (123.45, 678.90));

        // 位置を更新
        if let Some(p) = tree.persons.get_mut(&person) {
            p.position = (999.0, 111.0);
        }

        let p = tree.persons.get(&person).unwrap();
        assert_eq!(p.position, (999.0, 111.0));
    }
}

use std::collections::HashMap;

use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use uuid::Uuid;

use crate::application::{TreeRepository, TreeRepositoryError};
use crate::core::tree::{
    Event, EventId, EventRelation, EventRelationType, Family, FamilyTree, Gender, ParentChild,
    Person, PersonDisplayMode, PersonId, Spouse,
};

/// `FamilyTree`をSQLiteファイルとして保存・読込するリポジトリ実装。
///
/// 人物・関係・家族・イベントを正規化したスキーマで保存する。
pub struct SqliteTreeRepository;

const SCHEMA_VERSION: i64 = 1;

impl SqliteTreeRepository {
    fn open_connection(file_path: &str) -> Result<Connection, TreeRepositoryError> {
        Connection::open(file_path).map_err(|error| TreeRepositoryError::Read(error.to_string()))
    }

    fn initialize_schema(connection: &Connection) -> Result<(), TreeRepositoryError> {
        connection
            .execute_batch(
                "
                PRAGMA foreign_keys = ON;

                CREATE TABLE IF NOT EXISTS tree_metadata (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    schema_version INTEGER NOT NULL,
                    updated_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS persons (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    gender INTEGER NOT NULL,
                    birth TEXT,
                    memo TEXT NOT NULL,
                    position_x REAL NOT NULL,
                    position_y REAL NOT NULL,
                    deceased INTEGER NOT NULL,
                    death TEXT,
                    photo_path TEXT,
                    display_mode INTEGER NOT NULL,
                    photo_scale REAL NOT NULL
                );

                CREATE TABLE IF NOT EXISTS parent_child_edges (
                    parent_id TEXT NOT NULL,
                    child_id TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    FOREIGN KEY(parent_id) REFERENCES persons(id) ON DELETE CASCADE,
                    FOREIGN KEY(child_id) REFERENCES persons(id) ON DELETE CASCADE
                );

                CREATE TABLE IF NOT EXISTS spouses (
                    person1_id TEXT NOT NULL,
                    person2_id TEXT NOT NULL,
                    memo TEXT NOT NULL,
                    FOREIGN KEY(person1_id) REFERENCES persons(id) ON DELETE CASCADE,
                    FOREIGN KEY(person2_id) REFERENCES persons(id) ON DELETE CASCADE
                );

                CREATE TABLE IF NOT EXISTS families (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    color_r INTEGER,
                    color_g INTEGER,
                    color_b INTEGER
                );

                CREATE TABLE IF NOT EXISTS family_members (
                    family_id TEXT NOT NULL,
                    person_id TEXT NOT NULL,
                    PRIMARY KEY(family_id, person_id),
                    FOREIGN KEY(family_id) REFERENCES families(id) ON DELETE CASCADE,
                    FOREIGN KEY(person_id) REFERENCES persons(id) ON DELETE CASCADE
                );

                CREATE TABLE IF NOT EXISTS events (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    date TEXT,
                    description TEXT NOT NULL,
                    position_x REAL NOT NULL,
                    position_y REAL NOT NULL,
                    color_r INTEGER NOT NULL,
                    color_g INTEGER NOT NULL,
                    color_b INTEGER NOT NULL
                );

                CREATE TABLE IF NOT EXISTS event_relations (
                    event_id TEXT NOT NULL,
                    person_id TEXT NOT NULL,
                    relation_type INTEGER NOT NULL,
                    memo TEXT NOT NULL,
                    FOREIGN KEY(event_id) REFERENCES events(id) ON DELETE CASCADE,
                    FOREIGN KEY(person_id) REFERENCES persons(id) ON DELETE CASCADE
                );

                CREATE INDEX IF NOT EXISTS idx_parent_child_parent ON parent_child_edges(parent_id);
                CREATE INDEX IF NOT EXISTS idx_parent_child_child ON parent_child_edges(child_id);
                CREATE INDEX IF NOT EXISTS idx_family_members_person ON family_members(person_id);
                CREATE INDEX IF NOT EXISTS idx_event_relations_event ON event_relations(event_id);
                CREATE INDEX IF NOT EXISTS idx_event_relations_person ON event_relations(person_id);
                ",
            )
            .map_err(|error| TreeRepositoryError::Write(error.to_string()))
    }

    fn has_saved_tree(connection: &Connection) -> Result<bool, TreeRepositoryError> {
        connection
            .query_row(
                "SELECT schema_version FROM tree_metadata WHERE id = 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|error| TreeRepositoryError::Read(error.to_string()))
            .map(|version| version.is_some())
    }

    fn parse_uuid(value: &str, field_name: &str) -> Result<Uuid, TreeRepositoryError> {
        Uuid::parse_str(value)
            .map_err(|error| TreeRepositoryError::Deserialize(format!("invalid {field_name}: {error}")))
    }

    fn to_bool(value: i64, field_name: &str) -> Result<bool, TreeRepositoryError> {
        match value {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(TreeRepositoryError::Deserialize(format!(
                "invalid boolean value for {field_name}: {value}"
            ))),
        }
    }

    fn to_gender(value: i64) -> Result<Gender, TreeRepositoryError> {
        match value {
            0 => Ok(Gender::Male),
            1 => Ok(Gender::Female),
            2 => Ok(Gender::Unknown),
            _ => Err(TreeRepositoryError::Deserialize(format!(
                "invalid gender value: {value}"
            ))),
        }
    }

    fn to_display_mode(value: i64) -> Result<PersonDisplayMode, TreeRepositoryError> {
        match value {
            0 => Ok(PersonDisplayMode::NameOnly),
            1 => Ok(PersonDisplayMode::NameAndPhoto),
            _ => Err(TreeRepositoryError::Deserialize(format!(
                "invalid display_mode value: {value}"
            ))),
        }
    }

    fn to_event_relation_type(value: i64) -> Result<EventRelationType, TreeRepositoryError> {
        match value {
            0 => Ok(EventRelationType::Line),
            1 => Ok(EventRelationType::ArrowToPerson),
            2 => Ok(EventRelationType::ArrowToEvent),
            _ => Err(TreeRepositoryError::Deserialize(format!(
                "invalid event relation type value: {value}"
            ))),
        }
    }

    fn from_gender(value: Gender) -> i64 {
        match value {
            Gender::Male => 0,
            Gender::Female => 1,
            Gender::Unknown => 2,
        }
    }

    fn from_display_mode(value: PersonDisplayMode) -> i64 {
        match value {
            PersonDisplayMode::NameOnly => 0,
            PersonDisplayMode::NameAndPhoto => 1,
        }
    }

    fn from_event_relation_type(value: EventRelationType) -> i64 {
        match value {
            EventRelationType::Line => 0,
            EventRelationType::ArrowToPerson => 1,
            EventRelationType::ArrowToEvent => 2,
        }
    }

    fn clear_all_tables(transaction: &Transaction<'_>) -> Result<(), TreeRepositoryError> {
        transaction
            .execute_batch(
                "
                DELETE FROM event_relations;
                DELETE FROM events;
                DELETE FROM family_members;
                DELETE FROM families;
                DELETE FROM spouses;
                DELETE FROM parent_child_edges;
                DELETE FROM persons;
                ",
            )
            .map_err(|error| TreeRepositoryError::Write(error.to_string()))
    }

    fn load_persons(connection: &Connection) -> Result<HashMap<PersonId, Person>, TreeRepositoryError> {
        let mut statement = connection
            .prepare(
                "
                SELECT
                    id, name, gender, birth, memo,
                    position_x, position_y, deceased, death,
                    photo_path, display_mode, photo_scale
                FROM persons
                ",
            )
            .map_err(|error| TreeRepositoryError::Read(error.to_string()))?;

        let person_rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, f32>(5)?,
                    row.get::<_, f32>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, i64>(10)?,
                    row.get::<_, f32>(11)?,
                ))
            })
            .map_err(|error| TreeRepositoryError::Read(error.to_string()))?;

        let mut persons = HashMap::new();
        for person_row in person_rows {
            let (
                id_text,
                name,
                gender_value,
                birth,
                memo,
                position_x,
                position_y,
                deceased_value,
                death,
                photo_path,
                display_mode_value,
                photo_scale,
            ) = person_row.map_err(|error| TreeRepositoryError::Read(error.to_string()))?;

            let id = Self::parse_uuid(&id_text, "person id")?;
            let gender = Self::to_gender(gender_value)?;
            let deceased = Self::to_bool(deceased_value, "deceased")?;
            let display_mode = Self::to_display_mode(display_mode_value)?;

            persons.insert(
                id,
                Person {
                    id,
                    name,
                    gender,
                    birth,
                    memo,
                    position: (position_x, position_y),
                    deceased,
                    death,
                    photo_path,
                    display_mode,
                    photo_scale,
                },
            );
        }

        Ok(persons)
    }

    fn load_parent_child_edges(connection: &Connection) -> Result<Vec<ParentChild>, TreeRepositoryError> {
        let mut statement = connection
            .prepare("SELECT parent_id, child_id, kind FROM parent_child_edges")
            .map_err(|error| TreeRepositoryError::Read(error.to_string()))?;

        let edge_rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|error| TreeRepositoryError::Read(error.to_string()))?;

        let mut edges = Vec::new();
        for edge_row in edge_rows {
            let (parent_text, child_text, kind) =
                edge_row.map_err(|error| TreeRepositoryError::Read(error.to_string()))?;
            edges.push(ParentChild {
                parent: Self::parse_uuid(&parent_text, "edge parent_id")?,
                child: Self::parse_uuid(&child_text, "edge child_id")?,
                kind,
            });
        }

        Ok(edges)
    }

    fn load_spouses(connection: &Connection) -> Result<Vec<Spouse>, TreeRepositoryError> {
        let mut statement = connection
            .prepare("SELECT person1_id, person2_id, memo FROM spouses")
            .map_err(|error| TreeRepositoryError::Read(error.to_string()))?;

        let spouse_rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|error| TreeRepositoryError::Read(error.to_string()))?;

        let mut spouses = Vec::new();
        for spouse_row in spouse_rows {
            let (person1_text, person2_text, memo) =
                spouse_row.map_err(|error| TreeRepositoryError::Read(error.to_string()))?;
            spouses.push(Spouse {
                person1: Self::parse_uuid(&person1_text, "spouse person1_id")?,
                person2: Self::parse_uuid(&person2_text, "spouse person2_id")?,
                memo,
            });
        }

        Ok(spouses)
    }

    fn load_families(connection: &Connection) -> Result<Vec<Family>, TreeRepositoryError> {
        let mut statement = connection
            .prepare("SELECT id, name, color_r, color_g, color_b FROM families")
            .map_err(|error| TreeRepositoryError::Read(error.to_string()))?;

        let family_rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                ))
            })
            .map_err(|error| TreeRepositoryError::Read(error.to_string()))?;

        let mut families = Vec::new();
        let mut family_index = HashMap::new();

        for family_row in family_rows {
            let (id_text, name, color_r, color_g, color_b) =
                family_row.map_err(|error| TreeRepositoryError::Read(error.to_string()))?;

            let id = Self::parse_uuid(&id_text, "family id")?;
            let color = match (color_r, color_g, color_b) {
                (Some(red), Some(green), Some(blue)) => Some((red as u8, green as u8, blue as u8)),
                (None, None, None) => None,
                _ => {
                    return Err(TreeRepositoryError::Deserialize(
                        "invalid family color columns".to_string(),
                    ))
                }
            };

            family_index.insert(id, families.len());
            families.push(Family {
                id,
                name,
                members: Vec::new(),
                color,
            });
        }

        let mut member_statement = connection
            .prepare("SELECT family_id, person_id FROM family_members")
            .map_err(|error| TreeRepositoryError::Read(error.to_string()))?;

        let member_rows = member_statement
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .map_err(|error| TreeRepositoryError::Read(error.to_string()))?;

        for member_row in member_rows {
            let (family_id_text, person_id_text) =
                member_row.map_err(|error| TreeRepositoryError::Read(error.to_string()))?;
            let family_id = Self::parse_uuid(&family_id_text, "family_member family_id")?;
            let person_id = Self::parse_uuid(&person_id_text, "family_member person_id")?;

            if let Some(index) = family_index.get(&family_id) {
                families[*index].members.push(person_id);
            } else {
                return Err(TreeRepositoryError::Deserialize(format!(
                    "family_members references unknown family: {family_id}"
                )));
            }
        }

        Ok(families)
    }

    fn load_events(connection: &Connection) -> Result<HashMap<EventId, Event>, TreeRepositoryError> {
        let mut statement = connection
            .prepare(
                "
                SELECT
                    id, name, date, description,
                    position_x, position_y, color_r, color_g, color_b
                FROM events
                ",
            )
            .map_err(|error| TreeRepositoryError::Read(error.to_string()))?;

        let event_rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, f32>(4)?,
                    row.get::<_, f32>(5)?,
                    row.get::<_, u8>(6)?,
                    row.get::<_, u8>(7)?,
                    row.get::<_, u8>(8)?,
                ))
            })
            .map_err(|error| TreeRepositoryError::Read(error.to_string()))?;

        let mut events = HashMap::new();
        for event_row in event_rows {
            let (id_text, name, date, description, position_x, position_y, red, green, blue) =
                event_row.map_err(|error| TreeRepositoryError::Read(error.to_string()))?;

            let id = Self::parse_uuid(&id_text, "event id")?;
            events.insert(
                id,
                Event {
                    id,
                    name,
                    date,
                    description,
                    position: (position_x, position_y),
                    color: (red, green, blue),
                },
            );
        }

        Ok(events)
    }

    fn load_event_relations(connection: &Connection) -> Result<Vec<EventRelation>, TreeRepositoryError> {
        let mut statement = connection
            .prepare("SELECT event_id, person_id, relation_type, memo FROM event_relations")
            .map_err(|error| TreeRepositoryError::Read(error.to_string()))?;

        let relation_rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|error| TreeRepositoryError::Read(error.to_string()))?;

        let mut relations = Vec::new();
        for relation_row in relation_rows {
            let (event_id_text, person_id_text, relation_type_value, memo) =
                relation_row.map_err(|error| TreeRepositoryError::Read(error.to_string()))?;
            relations.push(EventRelation {
                event: Self::parse_uuid(&event_id_text, "event_relation event_id")?,
                person: Self::parse_uuid(&person_id_text, "event_relation person_id")?,
                relation_type: Self::to_event_relation_type(relation_type_value)?,
                memo,
            });
        }

        Ok(relations)
    }

    fn insert_persons(
        transaction: &Transaction<'_>,
        persons: &HashMap<PersonId, Person>,
    ) -> Result<(), TreeRepositoryError> {
        let mut statement = transaction
            .prepare(
                "
                INSERT INTO persons (
                    id, name, gender, birth, memo,
                    position_x, position_y, deceased, death,
                    photo_path, display_mode, photo_scale
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                ",
            )
            .map_err(|error| TreeRepositoryError::Write(error.to_string()))?;

        for person in persons.values() {
            statement
                .execute(params![
                    person.id.to_string(),
                    &person.name,
                    Self::from_gender(person.gender),
                    &person.birth,
                    &person.memo,
                    person.position.0,
                    person.position.1,
                    if person.deceased { 1_i64 } else { 0_i64 },
                    &person.death,
                    &person.photo_path,
                    Self::from_display_mode(person.display_mode),
                    person.photo_scale
                ])
                .map_err(|error| TreeRepositoryError::Write(error.to_string()))?;
        }

        Ok(())
    }

    fn insert_parent_child_edges(
        transaction: &Transaction<'_>,
        edges: &[ParentChild],
    ) -> Result<(), TreeRepositoryError> {
        let mut statement = transaction
            .prepare("INSERT INTO parent_child_edges (parent_id, child_id, kind) VALUES (?1, ?2, ?3)")
            .map_err(|error| TreeRepositoryError::Write(error.to_string()))?;

        for edge in edges {
            statement
                .execute(params![edge.parent.to_string(), edge.child.to_string(), &edge.kind])
                .map_err(|error| TreeRepositoryError::Write(error.to_string()))?;
        }

        Ok(())
    }

    fn insert_spouses(transaction: &Transaction<'_>, spouses: &[Spouse]) -> Result<(), TreeRepositoryError> {
        let mut statement = transaction
            .prepare("INSERT INTO spouses (person1_id, person2_id, memo) VALUES (?1, ?2, ?3)")
            .map_err(|error| TreeRepositoryError::Write(error.to_string()))?;

        for spouse in spouses {
            statement
                .execute(params![
                    spouse.person1.to_string(),
                    spouse.person2.to_string(),
                    &spouse.memo
                ])
                .map_err(|error| TreeRepositoryError::Write(error.to_string()))?;
        }

        Ok(())
    }

    fn insert_families(transaction: &Transaction<'_>, families: &[Family]) -> Result<(), TreeRepositoryError> {
        let mut family_statement = transaction
            .prepare("INSERT INTO families (id, name, color_r, color_g, color_b) VALUES (?1, ?2, ?3, ?4, ?5)")
            .map_err(|error| TreeRepositoryError::Write(error.to_string()))?;

        let mut member_statement = transaction
            .prepare("INSERT INTO family_members (family_id, person_id) VALUES (?1, ?2)")
            .map_err(|error| TreeRepositoryError::Write(error.to_string()))?;

        for family in families {
            let (color_r, color_g, color_b) = match family.color {
                Some((red, green, blue)) => (Some(red as i64), Some(green as i64), Some(blue as i64)),
                None => (None, None, None),
            };

            family_statement
                .execute(params![family.id.to_string(), &family.name, color_r, color_g, color_b])
                .map_err(|error| TreeRepositoryError::Write(error.to_string()))?;

            for member_id in &family.members {
                member_statement
                    .execute(params![family.id.to_string(), member_id.to_string()])
                    .map_err(|error| TreeRepositoryError::Write(error.to_string()))?;
            }
        }

        Ok(())
    }

    fn insert_events(
        transaction: &Transaction<'_>,
        events: &HashMap<EventId, Event>,
    ) -> Result<(), TreeRepositoryError> {
        let mut statement = transaction
            .prepare(
                "
                INSERT INTO events (
                    id, name, date, description, position_x, position_y, color_r, color_g, color_b
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                ",
            )
            .map_err(|error| TreeRepositoryError::Write(error.to_string()))?;

        for event in events.values() {
            statement
                .execute(params![
                    event.id.to_string(),
                    &event.name,
                    &event.date,
                    &event.description,
                    event.position.0,
                    event.position.1,
                    event.color.0 as i64,
                    event.color.1 as i64,
                    event.color.2 as i64
                ])
                .map_err(|error| TreeRepositoryError::Write(error.to_string()))?;
        }

        Ok(())
    }

    fn insert_event_relations(
        transaction: &Transaction<'_>,
        relations: &[EventRelation],
    ) -> Result<(), TreeRepositoryError> {
        let mut statement = transaction
            .prepare(
                "
                INSERT INTO event_relations (event_id, person_id, relation_type, memo)
                VALUES (?1, ?2, ?3, ?4)
                ",
            )
            .map_err(|error| TreeRepositoryError::Write(error.to_string()))?;

        for relation in relations {
            statement
                .execute(params![
                    relation.event.to_string(),
                    relation.person.to_string(),
                    Self::from_event_relation_type(relation.relation_type),
                    &relation.memo
                ])
                .map_err(|error| TreeRepositoryError::Write(error.to_string()))?;
        }

        Ok(())
    }

    fn upsert_metadata(transaction: &Transaction<'_>) -> Result<(), TreeRepositoryError> {
        let updated_at = Utc::now().to_rfc3339();

        transaction
            .execute(
                "
                INSERT INTO tree_metadata (id, schema_version, updated_at)
                VALUES (1, ?1, ?2)
                ON CONFLICT(id) DO UPDATE SET
                    schema_version = excluded.schema_version,
                    updated_at = excluded.updated_at
                
                ",
                params![SCHEMA_VERSION, updated_at],
            )
            .map_err(|error| TreeRepositoryError::Write(error.to_string()))?;

        Ok(())
    }
}

impl TreeRepository for SqliteTreeRepository {
    fn load(&self, file_path: &str) -> Result<FamilyTree, TreeRepositoryError> {
        let connection = Self::open_connection(file_path)?;
        Self::initialize_schema(&connection)?;
        let has_saved_tree = Self::has_saved_tree(&connection)?;
        if !has_saved_tree {
            return Err(TreeRepositoryError::Read(
                "no tree data found in sqlite file".to_string(),
            ));
        }

        let persons = Self::load_persons(&connection)?;
        let edges = Self::load_parent_child_edges(&connection)?;
        let spouses = Self::load_spouses(&connection)?;
        let families = Self::load_families(&connection)?;
        let events = Self::load_events(&connection)?;
        let event_relations = Self::load_event_relations(&connection)?;

        Ok(FamilyTree {
            persons,
            edges,
            spouses,
            families,
            events,
            event_relations,
        })
    }

    fn save(&self, file_path: &str, tree: &FamilyTree) -> Result<(), TreeRepositoryError> {
        let mut connection = Self::open_connection(file_path)
            .map_err(|error| TreeRepositoryError::Write(error.to_string()))?;
        Self::initialize_schema(&connection)?;
        let transaction = connection
            .transaction()
            .map_err(|error| TreeRepositoryError::Write(error.to_string()))?;

        Self::clear_all_tables(&transaction)?;
        Self::insert_persons(&transaction, &tree.persons)?;
        Self::insert_parent_child_edges(&transaction, &tree.edges)?;
        Self::insert_spouses(&transaction, &tree.spouses)?;
        Self::insert_families(&transaction, &tree.families)?;
        Self::insert_events(&transaction, &tree.events)?;
        Self::insert_event_relations(&transaction, &tree.event_relations)?;
        Self::upsert_metadata(&transaction)?;

        transaction
            .commit()
            .map_err(|error| TreeRepositoryError::Write(error.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;

    use uuid::Uuid;

    use super::SqliteTreeRepository;
    use crate::application::TreeRepository;
    use crate::core::tree::{EventRelationType, FamilyTree, Gender, PersonDisplayMode};

    #[test]
    fn save_and_load_round_trip() {
        let repository = SqliteTreeRepository;
        let file_name = format!("family_tree_test_{}.sqlite", Uuid::new_v4());
        let file_path = env::temp_dir().join(file_name);
        let file_path_str = file_path.to_string_lossy().to_string();
        let tree = FamilyTree::default();

        let save_result = repository.save(&file_path_str, &tree);
        assert!(save_result.is_ok(), "{save_result:?}");

        let loaded_tree_result = repository.load(&file_path_str);
        assert!(loaded_tree_result.is_ok(), "{loaded_tree_result:?}");
        let loaded_tree = loaded_tree_result.expect("sqlite file should load");
        assert_eq!(loaded_tree.persons.len(), 0);

        let remove_result = fs::remove_file(file_path);
        assert!(remove_result.is_ok());
    }

    #[test]
    fn save_and_load_round_trip_with_entities() {
        let repository = SqliteTreeRepository;
        let file_name = format!("family_tree_test_full_{}.sqlite", Uuid::new_v4());
        let file_path = env::temp_dir().join(file_name);
        let file_path_str = file_path.to_string_lossy().to_string();

        let mut tree = FamilyTree::default();
        let parent_id = tree.add_person(
            "Parent".to_string(),
            Gender::Male,
            Some("1970-01-01".to_string()),
            "memo parent".to_string(),
            false,
            None,
            (100.0, 120.0),
        );
        let child_id = tree.add_person(
            "Child".to_string(),
            Gender::Female,
            Some("2000-02-02".to_string()),
            "memo child".to_string(),
            false,
            None,
            (220.0, 240.0),
        );
        tree.add_parent_child(parent_id, child_id, "biological".to_string());
        tree.add_spouse(parent_id, child_id, "test spouse".to_string());

        if let Some(parent) = tree.persons.get_mut(&parent_id) {
            parent.display_mode = PersonDisplayMode::NameAndPhoto;
        }

        let family_id = tree.add_family("Main Family".to_string(), Some((1, 2, 3)));
        tree.add_member_to_family(family_id, parent_id);
        tree.add_member_to_family(family_id, child_id);

        let event_id = tree.add_event(
            "Test Event".to_string(),
            Some("2020-03-03".to_string()),
            "event memo".to_string(),
            (300.0, 320.0),
            (10, 20, 30),
        );
        tree.add_event_relation(
            event_id,
            parent_id,
            EventRelationType::ArrowToPerson,
            "event relation memo".to_string(),
        );

        let save_result = repository.save(&file_path_str, &tree);
        assert!(save_result.is_ok(), "{save_result:?}");

        let loaded_tree_result = repository.load(&file_path_str);
        assert!(loaded_tree_result.is_ok(), "{loaded_tree_result:?}");
        let loaded_tree = loaded_tree_result.expect("sqlite file should load");

        assert_eq!(loaded_tree.persons.len(), 2);
        assert_eq!(loaded_tree.edges.len(), 1);
        assert_eq!(loaded_tree.spouses.len(), 1);
        assert_eq!(loaded_tree.families.len(), 1);
        assert_eq!(loaded_tree.events.len(), 1);
        assert_eq!(loaded_tree.event_relations.len(), 1);

        let loaded_parent = loaded_tree
            .persons
            .get(&parent_id)
            .expect("parent should exist after load");
        assert_eq!(loaded_parent.display_mode, PersonDisplayMode::NameAndPhoto);

        let loaded_family = loaded_tree
            .families
            .iter()
            .find(|family| family.id == family_id)
            .expect("family should exist after load");
        assert_eq!(loaded_family.members.len(), 2);
        assert_eq!(loaded_family.color, Some((1, 2, 3)));

        let loaded_relation = loaded_tree
            .event_relations
            .first()
            .expect("event relation should exist after load");
        assert_eq!(loaded_relation.relation_type, EventRelationType::ArrowToPerson);

        let remove_result = fs::remove_file(file_path);
        assert!(remove_result.is_ok());
    }
}

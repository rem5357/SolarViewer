// Association and tagging system for StellarForge

use crate::stellar_forge::core::Id;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use time::OffsetDateTime;

// Tag for simple categorization
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Tag(pub String);

impl Tag {
    pub fn new(tag: impl Into<String>) -> Self {
        Self(tag.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Tag {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Tag {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

// Association for complex relationships
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Association {
    pub id: String,
    pub association_type: AssociationType,
    pub role: String,
    pub group: String,
    pub since_epoch: Option<OffsetDateTime>,
    pub until_epoch: Option<OffsetDateTime>,
    pub properties: HashMap<String, serde_json::Value>,
}

impl Association {
    pub fn new(
        association_type: AssociationType,
        role: impl Into<String>,
        group: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            association_type,
            role: role.into(),
            group: group.into(),
            since_epoch: Some(OffsetDateTime::now_utc()),
            until_epoch: None,
            properties: HashMap::new(),
        }
    }

    pub fn is_active(&self, at: OffsetDateTime) -> bool {
        let after_start = self.since_epoch.map_or(true, |s| at >= s);
        let before_end = self.until_epoch.map_or(true, |e| at <= e);
        after_start && before_end
    }

    pub fn terminate(&mut self, at: OffsetDateTime) {
        self.until_epoch = Some(at);
    }

    pub fn add_property(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.properties.insert(key.into(), value);
    }
}

// Types of associations
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AssociationType {
    Political,      // Nation, empire, federation membership
    Economic,       // Trade alliance, economic zone
    Military,       // Military alliance, defense pact
    Scientific,     // Research consortium, data sharing
    Cultural,       // Cultural group, religious affiliation
    Ownership,      // Who owns what
    Route,          // Part of a trade/travel route
    Historical,     // Historical significance
    Discovery,      // Who discovered what and when
    Custom(String),
}

// Association manager for tracking relationships
pub struct AssociationManager {
    associations_by_entity: HashMap<Id, Vec<Association>>,
    associations_by_group: HashMap<String, Vec<Id>>,
    tags_by_entity: HashMap<Id, HashSet<Tag>>,
    entities_by_tag: HashMap<Tag, HashSet<Id>>,
}

impl AssociationManager {
    pub fn new() -> Self {
        Self {
            associations_by_entity: HashMap::new(),
            associations_by_group: HashMap::new(),
            tags_by_entity: HashMap::new(),
            entities_by_tag: HashMap::new(),
        }
    }

    // Add an association
    pub fn add_association(&mut self, entity_id: Id, association: Association) {
        let group = association.group.clone();

        self.associations_by_entity
            .entry(entity_id)
            .or_insert_with(Vec::new)
            .push(association);

        self.associations_by_group
            .entry(group)
            .or_insert_with(Vec::new)
            .push(entity_id);
    }

    // Remove an association
    pub fn remove_association(&mut self, entity_id: Id, association_id: &str) -> bool {
        if let Some(associations) = self.associations_by_entity.get_mut(&entity_id) {
            if let Some(pos) = associations.iter().position(|a| a.id == association_id) {
                let removed = associations.remove(pos);

                // Update group index
                if let Some(group_entities) = self.associations_by_group.get_mut(&removed.group) {
                    group_entities.retain(|&id| id != entity_id);
                }

                return true;
            }
        }
        false
    }

    // Get all associations for an entity
    pub fn get_associations(&self, entity_id: Id) -> Option<&Vec<Association>> {
        self.associations_by_entity.get(&entity_id)
    }

    // Get all active associations at a specific time
    pub fn get_active_associations(&self, entity_id: Id, at: OffsetDateTime) -> Vec<&Association> {
        self.associations_by_entity
            .get(&entity_id)
            .map(|associations| {
                associations
                    .iter()
                    .filter(|a| a.is_active(at))
                    .collect()
            })
            .unwrap_or_default()
    }

    // Get all entities in a group
    pub fn get_group_members(&self, group: &str) -> Vec<Id> {
        self.associations_by_group
            .get(group)
            .cloned()
            .unwrap_or_default()
    }

    // Add a tag to an entity
    pub fn add_tag(&mut self, entity_id: Id, tag: Tag) {
        self.tags_by_entity
            .entry(entity_id)
            .or_insert_with(HashSet::new)
            .insert(tag.clone());

        self.entities_by_tag
            .entry(tag)
            .or_insert_with(HashSet::new)
            .insert(entity_id);
    }

    // Remove a tag from an entity
    pub fn remove_tag(&mut self, entity_id: Id, tag: &Tag) -> bool {
        let mut removed = false;

        if let Some(tags) = self.tags_by_entity.get_mut(&entity_id) {
            removed = tags.remove(tag);
        }

        if removed {
            if let Some(entities) = self.entities_by_tag.get_mut(tag) {
                entities.remove(&entity_id);
            }
        }

        removed
    }

    // Get all tags for an entity
    pub fn get_tags(&self, entity_id: Id) -> Vec<Tag> {
        self.tags_by_entity
            .get(&entity_id)
            .map(|tags| tags.iter().cloned().collect())
            .unwrap_or_default()
    }

    // Get all entities with a specific tag
    pub fn get_entities_with_tag(&self, tag: &Tag) -> Vec<Id> {
        self.entities_by_tag
            .get(tag)
            .map(|entities| entities.iter().cloned().collect())
            .unwrap_or_default()
    }

    // Check if entity has a specific tag
    pub fn has_tag(&self, entity_id: Id, tag: &Tag) -> bool {
        self.tags_by_entity
            .get(&entity_id)
            .map(|tags| tags.contains(tag))
            .unwrap_or(false)
    }

    // Check if entity is member of a group
    pub fn is_member_of(&self, entity_id: Id, group: &str) -> bool {
        self.associations_by_entity
            .get(&entity_id)
            .map(|associations| associations.iter().any(|a| a.group == group))
            .unwrap_or(false)
    }

    // Find common groups between entities
    pub fn find_common_groups(&self, entity1: Id, entity2: Id) -> Vec<String> {
        let groups1 = self.get_groups_for_entity(entity1);
        let groups2 = self.get_groups_for_entity(entity2);

        let set1: HashSet<_> = groups1.into_iter().collect();
        let set2: HashSet<_> = groups2.into_iter().collect();

        set1.intersection(&set2).cloned().collect()
    }

    fn get_groups_for_entity(&self, entity_id: Id) -> Vec<String> {
        self.associations_by_entity
            .get(&entity_id)
            .map(|associations| {
                associations
                    .iter()
                    .map(|a| a.group.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    // Query associations by type
    pub fn find_by_type(&self, association_type: &AssociationType) -> Vec<(Id, &Association)> {
        let mut results = Vec::new();

        for (entity_id, associations) in &self.associations_by_entity {
            for association in associations {
                if association.association_type == *association_type {
                    results.push((*entity_id, association));
                }
            }
        }

        results
    }

    // Bulk operations for tags
    pub fn add_tags_bulk(&mut self, entity_id: Id, tags: Vec<Tag>) {
        for tag in tags {
            self.add_tag(entity_id, tag);
        }
    }

    pub fn remove_tags_bulk(&mut self, entity_id: Id, tags: Vec<Tag>) {
        for tag in tags {
            self.remove_tag(entity_id, &tag);
        }
    }

    // Clear all associations and tags for an entity
    pub fn clear_entity(&mut self, entity_id: Id) {
        // Clear associations
        if let Some(associations) = self.associations_by_entity.remove(&entity_id) {
            for association in associations {
                if let Some(group_entities) = self.associations_by_group.get_mut(&association.group) {
                    group_entities.retain(|&id| id != entity_id);
                }
            }
        }

        // Clear tags
        if let Some(tags) = self.tags_by_entity.remove(&entity_id) {
            for tag in tags {
                if let Some(entities) = self.entities_by_tag.get_mut(&tag) {
                    entities.remove(&entity_id);
                }
            }
        }
    }
}

// Predefined tag categories for common use
pub struct TagCategories;

impl TagCategories {
    pub fn exploration() -> Vec<Tag> {
        vec![
            Tag::from("unexplored"),
            Tag::from("surveyed"),
            Tag::from("mapped"),
            Tag::from("colonizable"),
            Tag::from("hostile"),
        ]
    }

    pub fn resources() -> Vec<Tag> {
        vec![
            Tag::from("resource_rich"),
            Tag::from("mining_site"),
            Tag::from("gas_giant_mining"),
            Tag::from("rare_minerals"),
            Tag::from("water_source"),
        ]
    }

    pub fn danger() -> Vec<Tag> {
        vec![
            Tag::from("hazardous"),
            Tag::from("radiation"),
            Tag::from("asteroid_field"),
            Tag::from("unstable"),
            Tag::from("quarantine"),
        ]
    }

    pub fn civilization() -> Vec<Tag> {
        vec![
            Tag::from("inhabited"),
            Tag::from("colony"),
            Tag::from("outpost"),
            Tag::from("capital"),
            Tag::from("trade_hub"),
        ]
    }

    pub fn strategic() -> Vec<Tag> {
        vec![
            Tag::from("strategic_importance"),
            Tag::from("choke_point"),
            Tag::from("border_system"),
            Tag::from("disputed"),
            Tag::from("neutral_zone"),
        ]
    }
}

// Relationship types for common associations
pub struct RelationshipTypes;

impl RelationshipTypes {
    pub fn political_membership(nation: &str) -> Association {
        Association::new(
            AssociationType::Political,
            "member",
            format!("Nation:{}", nation),
        )
    }

    pub fn trade_route(route_name: &str, position: u32) -> Association {
        let mut assoc = Association::new(
            AssociationType::Economic,
            "waypoint",
            format!("Route:{}", route_name),
        );
        assoc.add_property("position", serde_json::json!(position));
        assoc
    }

    pub fn discovery(discoverer: &str, date: OffsetDateTime) -> Association {
        let mut assoc = Association::new(
            AssociationType::Discovery,
            "discovered_by",
            format!("Explorer:{}", discoverer),
        );
        assoc.since_epoch = Some(date);
        assoc
    }

    pub fn ownership(owner: &str) -> Association {
        Association::new(
            AssociationType::Ownership,
            "owned_by",
            format!("Entity:{}", owner),
        )
    }

    pub fn military_alliance(alliance_name: &str) -> Association {
        Association::new(
            AssociationType::Military,
            "member",
            format!("Alliance:{}", alliance_name),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_association_manager() {
        let mut manager = AssociationManager::new();
        let entity_id = Id::new_v4();

        // Test adding tags
        manager.add_tag(entity_id, Tag::from("inhabited"));
        manager.add_tag(entity_id, Tag::from("trade_hub"));

        assert!(manager.has_tag(entity_id, &Tag::from("inhabited")));
        assert_eq!(manager.get_tags(entity_id).len(), 2);

        // Test adding association
        let assoc = RelationshipTypes::political_membership("Federation");
        manager.add_association(entity_id, assoc);

        assert!(manager.is_member_of(entity_id, "Nation:Federation"));
        assert_eq!(manager.get_group_members("Nation:Federation").len(), 1);
    }

    #[test]
    fn test_tag_categories() {
        let exploration_tags = TagCategories::exploration();
        assert!(exploration_tags.contains(&Tag::from("unexplored")));
        assert!(exploration_tags.contains(&Tag::from("colonizable")));
    }

    #[test]
    fn test_association_active_period() {
        let mut assoc = Association::new(
            AssociationType::Political,
            "member",
            "TestGroup",
        );

        let now = OffsetDateTime::now_utc();
        let past = now - time::Duration::days(10);
        let future = now + time::Duration::days(10);

        assoc.since_epoch = Some(past);
        assoc.until_epoch = Some(future);

        assert!(assoc.is_active(now));
        assert!(!assoc.is_active(past - time::Duration::days(1)));
        assert!(!assoc.is_active(future + time::Duration::days(1)));
    }
}
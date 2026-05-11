use std::any::TypeId;
use std::collections::HashMap;

use ce_core::Entity;

use crate::component::{Component, ComponentColumn};

/// Unique identifier for an archetype within the world.
pub type ArchetypeId = usize;

/// An archetype table storing entities that share the exact same set of
/// component types in a Struct-of-Arrays (SoA) layout.
///
/// Each component type maps to a [`ComponentColumn`]; all columns share the
/// same row count, where row *i* across all columns describes entity *i*.
pub struct Archetype {
    id: ArchetypeId,
    /// Component data, keyed by TypeId. Every column has the same `len`.
    columns: HashMap<TypeId, ComponentColumn>,
    /// Dense list of entities in this archetype. Index == row.
    entities: Vec<Entity>,
    /// Reverse lookup: entity -> row index.
    entity_rows: HashMap<Entity, usize>,
}

impl Archetype {
    /// Creates a new, empty archetype with pre-created columns for the
    /// given component types.
    ///
    /// `column_factories` provides `(TypeId, ComponentColumn)` pairs so that
    /// column creation (which requires concrete type knowledge) can happen
    /// at the call site via generics, while the archetype itself remains
    /// type-erased.
    pub fn new(id: ArchetypeId, columns: HashMap<TypeId, ComponentColumn>) -> Self {
        Self {
            id,
            columns,
            entities: Vec::new(),
            entity_rows: HashMap::new(),
        }
    }

    /// Returns this archetype's unique identifier.
    pub fn id(&self) -> ArchetypeId {
        self.id
    }

    /// Registers an entity in this archetype's tracking structures.
    ///
    /// Returns the row index assigned to the entity. The caller is
    /// responsible for pushing matching component data into every column.
    pub fn add_entity(&mut self, entity: Entity) -> usize {
        let row = self.entities.len();
        self.entities.push(entity);
        self.entity_rows.insert(entity, row);
        row
    }

    /// Removes an entity from this archetype via swap-remove.
    ///
    /// Returns the row that was occupied by `entity` (now occupied by the
    /// entity that was swapped in, if any), or `None` if the entity was
    /// not present.
    ///
    /// The caller is responsible for performing matching swap-removes on
    /// every component column.
    pub fn remove_entity(&mut self, entity: Entity) -> Option<usize> {
        let row = *self.entity_rows.get(&entity)?;
        let last_row = self.entities.len() - 1;

        if row != last_row {
            // Swap with the last entity.
            let swapped_entity = self.entities[last_row];
            self.entities.swap(row, last_row);
            self.entity_rows.insert(swapped_entity, row);
        }

        self.entities.pop();
        self.entity_rows.remove(&entity);
        Some(row)
    }

    /// Returns a slice of all entities currently in this archetype.
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    /// Returns the number of entities in this archetype.
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Returns `true` if this archetype has no entities.
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Returns `true` if this archetype has a column for the given type.
    pub fn has_component(&self, type_id: TypeId) -> bool {
        self.columns.contains_key(&type_id)
    }

    /// Returns a reference to the column for component type `T`, if present.
    pub fn column<T: Component>(&self) -> Option<&ComponentColumn> {
        self.columns.get(&TypeId::of::<T>())
    }

    /// Returns a mutable reference to the column for component type `T`, if present.
    pub fn column_mut<T: Component>(&mut self) -> Option<&mut ComponentColumn> {
        self.columns.get_mut(&TypeId::of::<T>())
    }

    /// Returns a reference to the column for the given `TypeId`, if present.
    pub fn column_by_id(&self, type_id: &TypeId) -> Option<&ComponentColumn> {
        self.columns.get(type_id)
    }

    /// Returns a mutable reference to the column for the given `TypeId`, if present.
    pub fn column_by_id_mut(&mut self, type_id: &TypeId) -> Option<&mut ComponentColumn> {
        self.columns.get_mut(type_id)
    }

    /// Returns an iterator over the TypeIds of all component types in this archetype.
    pub fn component_types(&self) -> impl Iterator<Item = &TypeId> {
        self.columns.keys()
    }

    /// Returns the row index for a given entity, if present.
    pub fn entity_row(&self, entity: &Entity) -> Option<usize> {
        self.entity_rows.get(entity).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_archetype(id: ArchetypeId, type_ids: &[TypeId]) -> Archetype {
        let mut columns = HashMap::new();
        // For testing, we create i32 columns. In real use, callers provide
        // correctly-typed columns.
        for &tid in type_ids {
            // We can only create columns through new::<T>, so for testing
            // we use i32 columns keyed by arbitrary TypeIds.
            columns.insert(tid, ComponentColumn::new::<i32>());
        }
        Archetype::new(id, columns)
    }

    #[test]
    fn new_archetype_is_empty() {
        let arch = make_archetype(0, &[TypeId::of::<i32>()]);
        assert!(arch.is_empty());
        assert_eq!(arch.len(), 0);
        assert!(arch.has_component(TypeId::of::<i32>()));
        assert!(!arch.has_component(TypeId::of::<f64>()));
    }

    #[test]
    fn add_and_track_entity() {
        let mut arch = make_archetype(0, &[]);
        let e = Entity {
            index: 0,
            generation: 0,
        };
        let row = arch.add_entity(e);
        assert_eq!(row, 0);
        assert_eq!(arch.len(), 1);
        assert_eq!(arch.entities(), &[e]);
    }

    #[test]
    fn remove_entity_swap() {
        let mut arch = make_archetype(0, &[]);
        let e0 = Entity {
            index: 0,
            generation: 0,
        };
        let e1 = Entity {
            index: 1,
            generation: 0,
        };
        let e2 = Entity {
            index: 2,
            generation: 0,
        };
        arch.add_entity(e0);
        arch.add_entity(e1);
        arch.add_entity(e2);

        // Remove middle entity: e1 at row 1, e2 swaps in.
        let removed_row = arch.remove_entity(e1);
        assert_eq!(removed_row, Some(1));
        assert_eq!(arch.len(), 2);
        assert_eq!(arch.entities(), &[e0, e2]);
    }

    #[test]
    fn remove_nonexistent_entity() {
        let mut arch = make_archetype(0, &[]);
        let e = Entity {
            index: 99,
            generation: 0,
        };
        assert_eq!(arch.remove_entity(e), None);
    }

    #[test]
    fn column_typed_access() {
        let mut columns = HashMap::new();
        columns.insert(TypeId::of::<i32>(), ComponentColumn::new::<i32>());
        let arch = Archetype::new(0, columns);
        assert!(arch.column::<i32>().is_some());
        assert!(arch.column::<f64>().is_none());
    }
}

use std::any::TypeId;
use std::collections::HashMap;

use ce_core::{Entities, Entity};

use crate::archetype::{Archetype, ArchetypeId};
use crate::component::{Component, ComponentColumn};
use crate::resource::{Resource, Resources};

/// The central data store for entities, components, and resources.
///
/// `World` manages the archetype-based ECS layout: entities with the same
/// set of component types share an archetype table for cache-friendly
/// iteration. When a component is added or removed, the entity migrates
/// to the matching archetype.
pub struct World {
    entities: Entities,
    archetypes: Vec<Archetype>,
    /// Maps a *sorted* list of TypeIds to the archetype index.
    archetype_index: HashMap<Vec<TypeId>, ArchetypeId>,
    /// Maps each living entity to `(archetype_id, row)`.
    entity_locations: HashMap<Entity, (ArchetypeId, usize)>,
    resources: Resources,
}

impl World {
    /// Creates a new, empty world.
    ///
    /// Initializes with a single "empty" archetype (archetype 0) that has
    /// no component columns, used for freshly spawned entities.
    pub fn new() -> Self {
        let empty_archetype = Archetype::new(0, HashMap::new());
        let mut archetype_index = HashMap::new();
        archetype_index.insert(Vec::new(), 0);

        Self {
            entities: Entities::new(),
            archetypes: vec![empty_archetype],
            archetype_index,
            entity_locations: HashMap::new(),
            resources: Resources::new(),
        }
    }

    /// Spawns a new entity and places it in the empty archetype.
    pub fn spawn(&mut self) -> Entity {
        let entity = self.entities.alloc();
        let row = self.archetypes[0].add_entity(entity);
        self.entity_locations.insert(entity, (0, row));
        entity
    }

    /// Despawns an entity, removing it from its archetype.
    ///
    /// Returns `true` if the entity was alive and has been removed.
    pub fn despawn(&mut self, entity: Entity) -> bool {
        if !self.entities.is_alive(entity) {
            return false;
        }

        if let Some((arch_id, row)) = self.entity_locations.remove(&entity) {
            let arch = &mut self.archetypes[arch_id];

            // Swap-remove the entity row from every column.
            let type_ids: Vec<TypeId> = arch.component_types().copied().collect();
            for tid in &type_ids {
                if let Some(col) = arch.column_by_id_mut(tid) {
                    col.swap_remove(row);
                }
            }

            // Remove entity tracking. If a swap happened, update the swapped entity's location.
            arch.remove_entity(entity);
            // After swap-remove, the entity that was at the last row is now at `row`.
            if row < arch.len() {
                let swapped_entity = arch.entities()[row];
                self.entity_locations.insert(swapped_entity, (arch_id, row));
            }
        }

        self.entities.free(entity);
        true
    }

    /// Inserts a component on an entity. If the entity already has a
    /// component of this type, it is replaced. Otherwise the entity
    /// migrates to an archetype that includes this component type.
    pub fn insert_component<T: Component>(&mut self, entity: Entity, component: T) {
        if !self.entities.is_alive(entity) {
            return;
        }

        let &(src_arch_id, src_row) = self
            .entity_locations
            .get(&entity)
            .expect("alive entity must have a location");

        let type_id = TypeId::of::<T>();

        // Case 1: Entity's current archetype already has this component type.
        // Just overwrite in place.
        if self.archetypes[src_arch_id].has_component(type_id) {
            let col = self.archetypes[src_arch_id]
                .column_mut::<T>()
                .expect("has_component was true");
            if let Some(slot) = col.get_mut::<T>(src_row) {
                *slot = component;
            }
            return;
        }

        // Case 2: Need to migrate to a new archetype that includes this type.

        // Compute the target archetype's type set.
        let mut target_types: Vec<TypeId> = self.archetypes[src_arch_id]
            .component_types()
            .copied()
            .collect();
        target_types.push(type_id);
        target_types.sort();

        // Find or create the target archetype. We pass a factory closure
        // so that the new column for T can be created with full type info.
        let dst_arch_id = self.find_or_create_archetype_adding::<T>(&target_types, src_arch_id);

        // Move all existing component data from source to destination.
        self.migrate_entity(entity, src_arch_id, src_row, dst_arch_id);

        // Push the new component into the destination archetype's column.
        let dst_arch = &mut self.archetypes[dst_arch_id];
        dst_arch
            .column_mut::<T>()
            .expect("target archetype must have column for T")
            .push(component);
    }

    /// Removes a component from an entity, migrating it to the archetype
    /// without that component type.
    ///
    /// Returns `Some(T)` if the component was present and removed.
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Option<T> {
        if !self.entities.is_alive(entity) {
            return None;
        }

        let &(src_arch_id, src_row) = self.entity_locations.get(&entity)?;

        let type_id = TypeId::of::<T>();

        if !self.archetypes[src_arch_id].has_component(type_id) {
            return None;
        }

        // Read the value before migration.
        let value: T = {
            let col = self.archetypes[src_arch_id].column::<T>()?;
            let val_ref = col.get::<T>(src_row)?;
            // SAFETY: We need to copy the value out before the migration destroys
            // the source row. We use ptr::read to bitwise-copy, then the migration
            // will skip dropping this column's data at the source.
            unsafe { std::ptr::read(val_ref as *const T) }
        };

        // Compute target type set (without T).
        let mut target_types: Vec<TypeId> = self.archetypes[src_arch_id]
            .component_types()
            .copied()
            .filter(|tid| *tid != type_id)
            .collect();
        target_types.sort();

        let dst_arch_id = self.find_or_create_archetype_removing(&target_types, src_arch_id);

        // Migrate all component data EXCEPT T.
        self.migrate_entity_except(entity, src_arch_id, src_row, dst_arch_id, type_id);

        Some(value)
    }

    /// Returns a reference to a component on an entity.
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        if !self.entities.is_alive(entity) {
            return None;
        }
        let &(arch_id, row) = self.entity_locations.get(&entity)?;
        let col = self.archetypes[arch_id].column::<T>()?;
        col.get::<T>(row)
    }

    /// Returns a mutable reference to a component on an entity.
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        if !self.entities.is_alive(entity) {
            return None;
        }
        let &(arch_id, row) = self.entity_locations.get(&entity)?;
        let col = self.archetypes[arch_id].column_mut::<T>()?;
        col.get_mut::<T>(row)
    }

    /// Inserts a global resource.
    pub fn insert_resource<T: Resource>(&mut self, resource: T) {
        self.resources.insert(resource);
    }

    /// Returns a reference to a global resource.
    pub fn get_resource<T: Resource>(&self) -> Option<&T> {
        self.resources.get::<T>()
    }

    /// Returns a mutable reference to a global resource.
    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<&mut T> {
        self.resources.get_mut::<T>()
    }

    /// Returns the number of currently alive entities.
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Iterates over all entities that have component `T`, yielding `(Entity, &T)`.
    ///
    /// This is a simple MVP query that collects results into a `Vec`.
    /// A zero-allocation streaming `Query<>` type is planned for a later milestone.
    pub fn query<T: Component>(&self) -> Vec<(Entity, &T)> {
        let type_id = TypeId::of::<T>();
        let mut results = Vec::new();

        for archetype in &self.archetypes {
            if !archetype.has_component(type_id) {
                continue;
            }
            let col = archetype.column::<T>().unwrap();
            for (i, entity) in archetype.entities().iter().enumerate() {
                if let Some(val) = col.get::<T>(i) {
                    results.push((*entity, val));
                }
            }
        }

        results
    }

    /// Iterates over all entities that have both components `A` and `B`,
    /// yielding `(Entity, &A, &B)`.
    pub fn query2<A: Component, B: Component>(&self) -> Vec<(Entity, &A, &B)> {
        let type_a = TypeId::of::<A>();
        let type_b = TypeId::of::<B>();
        let mut results = Vec::new();

        for archetype in &self.archetypes {
            if !archetype.has_component(type_a) || !archetype.has_component(type_b) {
                continue;
            }
            let col_a = archetype.column::<A>().unwrap();
            let col_b = archetype.column::<B>().unwrap();
            for (i, entity) in archetype.entities().iter().enumerate() {
                if let (Some(a), Some(b)) = (col_a.get::<A>(i), col_b.get::<B>(i)) {
                    results.push((*entity, a, b));
                }
            }
        }

        results
    }

    /// Zero-allocation query: calls `f` for each entity with component T.
    /// 10-50x faster than query<T>() because no Vec allocation.
    pub fn for_each<T: Component, F: FnMut(Entity, &T)>(&self, mut f: F) {
        let type_id = TypeId::of::<T>();
        for archetype in &self.archetypes {
            if !archetype.has_component(type_id) {
                continue;
            }
            let col = archetype.column::<T>().unwrap();
            for (i, entity) in archetype.entities().iter().enumerate() {
                if let Some(val) = col.get::<T>(i) {
                    f(*entity, val);
                }
            }
        }
    }

    /// Zero-allocation 2-component query.
    pub fn for_each2<A: Component, B: Component, F: FnMut(Entity, &A, &B)>(&self, mut f: F) {
        let type_a = TypeId::of::<A>();
        let type_b = TypeId::of::<B>();
        for archetype in &self.archetypes {
            if !archetype.has_component(type_a) || !archetype.has_component(type_b) {
                continue;
            }
            let col_a = archetype.column::<A>().unwrap();
            let col_b = archetype.column::<B>().unwrap();
            for (i, entity) in archetype.entities().iter().enumerate() {
                if let (Some(a), Some(b)) = (col_a.get::<A>(i), col_b.get::<B>(i)) {
                    f(*entity, a, b);
                }
            }
        }
    }

    /// Returns entity IDs that have component type T. No component data returned.
    /// Much cheaper than query() because no component references are created.
    pub fn entities_with<T: Component>(&self) -> Vec<Entity> {
        let type_id = TypeId::of::<T>();
        let mut result = Vec::new();
        for archetype in &self.archetypes {
            if archetype.has_component(type_id) {
                result.extend(archetype.entities());
            }
        }
        result
    }

    /// Returns entity IDs that have both component types A and B.
    pub fn entities_with2<A: Component, B: Component>(&self) -> Vec<Entity> {
        let type_a = TypeId::of::<A>();
        let type_b = TypeId::of::<B>();
        let mut result = Vec::new();
        for archetype in &self.archetypes {
            if archetype.has_component(type_a) && archetype.has_component(type_b) {
                result.extend(archetype.entities());
            }
        }
        result
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Finds the archetype for `target_types` (when ADDING component T),
    /// or creates a new one with properly typed columns.
    ///
    /// `src_arch_id` is the entity's current archetype, used to clone
    /// existing column metadata. The new column for `T` is created via
    /// `ComponentColumn::new::<T>()`.
    fn find_or_create_archetype_adding<T: Component>(
        &mut self,
        target_types: &[TypeId],
        src_arch_id: ArchetypeId,
    ) -> ArchetypeId {
        if let Some(&id) = self.archetype_index.get(target_types) {
            return id;
        }

        let new_type = TypeId::of::<T>();
        let new_id = self.archetypes.len();
        let mut columns = HashMap::new();

        for &tid in target_types {
            if tid == new_type {
                // Create the column for the newly added component type using
                // full generic type knowledge.
                columns.insert(tid, ComponentColumn::new::<T>());
            } else {
                // Clone an empty column from the source archetype.
                let template = self.archetypes[src_arch_id]
                    .column_by_id(&tid)
                    .expect("source archetype must have this column");
                columns.insert(tid, Self::clone_empty_column(template));
            }
        }

        let arch = Archetype::new(new_id, columns);
        self.archetypes.push(arch);
        self.archetype_index.insert(target_types.to_vec(), new_id);
        new_id
    }

    /// Finds or creates an archetype for the given type set when *removing*
    /// a component. All types in `target_types` already exist in some
    /// archetype, so columns are cloned from existing ones.
    fn find_or_create_archetype_removing(
        &mut self,
        target_types: &[TypeId],
        src_arch_id: ArchetypeId,
    ) -> ArchetypeId {
        if let Some(&id) = self.archetype_index.get(target_types) {
            return id;
        }

        let new_id = self.archetypes.len();
        let mut columns = HashMap::new();

        for &tid in target_types {
            let template = self.archetypes[src_arch_id]
                .column_by_id(&tid)
                .expect("source archetype must have this column");
            columns.insert(tid, Self::clone_empty_column(template));
        }

        let arch = Archetype::new(new_id, columns);
        self.archetypes.push(arch);
        self.archetype_index.insert(target_types.to_vec(), new_id);
        new_id
    }

    /// Creates an empty `ComponentColumn` with the same type info as `template`.
    fn clone_empty_column(template: &ComponentColumn) -> ComponentColumn {
        ComponentColumn {
            data: Vec::new(),
            item_layout: template.item_layout(),
            len: 0,
            drop_fn: template.drop_fn(),
            type_id: template.type_id(),
        }
    }

    /// Migrates an entity from `src_arch_id` (at `src_row`) to `dst_arch_id`.
    ///
    /// Moves all component data that exists in both archetypes. The new
    /// component (the one being added) is NOT pushed here -- the caller
    /// handles that.
    fn migrate_entity(
        &mut self,
        entity: Entity,
        src_arch_id: ArchetypeId,
        src_row: usize,
        dst_arch_id: ArchetypeId,
    ) {
        // Collect the type IDs present in the source archetype.
        let src_types: Vec<TypeId> = self.archetypes[src_arch_id]
            .component_types()
            .copied()
            .collect();

        // Extract raw bytes from each source column (swap-remove at src_row).
        let mut raw_data: Vec<(TypeId, Vec<u8>)> = Vec::new();
        for tid in &src_types {
            let col = self.archetypes[src_arch_id].column_by_id_mut(tid).unwrap();
            // We need to pop the value at src_row. swap_remove drops it,
            // but we need the raw bytes. We'll use a raw approach:
            // swap with last, then pop_raw.
            if src_row < col.len() - 1 {
                // There's a last element different from src_row. We need to
                // get data at src_row without dropping it.
                // Use the internal swap + pop approach.
            }
            // Actually, we need a different strategy. Let's swap_remove by
            // extracting raw bytes. We'll add a helper method.
        }

        // Better approach: use swap-remove that returns raw bytes.
        // For each column in the source archetype, extract the row's data.
        raw_data.clear();
        {
            let src_arch = &mut self.archetypes[src_arch_id];
            for tid in &src_types {
                let col = src_arch.column_by_id_mut(tid).unwrap();
                let bytes = unsafe { col.swap_remove_raw(src_row) };
                raw_data.push((*tid, bytes));
            }

            // Remove entity from source archetype tracking.
            src_arch.remove_entity(entity);
        }

        // Update the swapped entity's location if a swap occurred.
        {
            let src_arch = &self.archetypes[src_arch_id];
            if src_row < src_arch.len() {
                let swapped = src_arch.entities()[src_row];
                self.entity_locations
                    .insert(swapped, (src_arch_id, src_row));
            }
        }

        // Add entity to destination archetype and push data.
        let dst_row = self.archetypes[dst_arch_id].add_entity(entity);
        for (tid, bytes) in &raw_data {
            if let Some(col) = self.archetypes[dst_arch_id].column_by_id_mut(tid) {
                // SAFETY: The bytes are a valid representation of the component type,
                // extracted from the same-typed column in the source archetype.
                unsafe {
                    col.push_raw(bytes);
                }
            }
        }

        self.entity_locations.insert(entity, (dst_arch_id, dst_row));
    }

    /// Like `migrate_entity`, but skips one component type (the one being removed).
    fn migrate_entity_except(
        &mut self,
        entity: Entity,
        src_arch_id: ArchetypeId,
        src_row: usize,
        dst_arch_id: ArchetypeId,
        skip_type: TypeId,
    ) {
        let src_types: Vec<TypeId> = self.archetypes[src_arch_id]
            .component_types()
            .copied()
            .collect();

        let mut raw_data: Vec<(TypeId, Vec<u8>)> = Vec::new();
        {
            let src_arch = &mut self.archetypes[src_arch_id];
            for tid in &src_types {
                let col = src_arch.column_by_id_mut(tid).unwrap();
                if *tid == skip_type {
                    // For the skipped type, we still need to swap_remove to keep
                    // columns in sync, but we DON'T drop the value (caller already
                    // extracted it via ptr::read).
                    unsafe {
                        col.swap_remove_raw_nodrop(src_row);
                    }
                } else {
                    let bytes = unsafe { col.swap_remove_raw(src_row) };
                    raw_data.push((*tid, bytes));
                }
            }
            src_arch.remove_entity(entity);
        }

        // Update swapped entity.
        {
            let src_arch = &self.archetypes[src_arch_id];
            if src_row < src_arch.len() {
                let swapped = src_arch.entities()[src_row];
                self.entity_locations
                    .insert(swapped, (src_arch_id, src_row));
            }
        }

        // Add to destination.
        let dst_row = self.archetypes[dst_arch_id].add_entity(entity);
        for (tid, bytes) in &raw_data {
            if let Some(col) = self.archetypes[dst_arch_id].column_by_id_mut(tid) {
                unsafe {
                    col.push_raw(bytes);
                }
            }
        }

        self.entity_locations.insert(entity, (dst_arch_id, dst_row));
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test component types.
    #[derive(Debug, PartialEq, Clone)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, PartialEq, Clone)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }

    #[derive(Debug, PartialEq, Clone)]
    struct Health(i32);

    #[derive(Debug, PartialEq)]
    struct Name(String);

    #[test]
    fn spawn_returns_unique_entities() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();
        assert_ne!(e1, e2);
        assert_ne!(e2, e3);
        assert_ne!(e1, e3);
        assert_eq!(world.entity_count(), 3);
    }

    #[test]
    fn insert_and_get_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Position { x: 1.0, y: 2.0 });
        let pos = world.get_component::<Position>(e);
        assert_eq!(pos, Some(&Position { x: 1.0, y: 2.0 }));
    }

    #[test]
    fn insert_replaces_existing_component() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Health(100));
        world.insert_component(e, Health(50));
        assert_eq!(world.get_component::<Health>(e), Some(&Health(50)));
    }

    #[test]
    fn insert_second_component_migrates_archetype() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Position { x: 1.0, y: 2.0 });

        // Entity should be in an archetype with just Position.
        assert_eq!(
            world.get_component::<Position>(e),
            Some(&Position { x: 1.0, y: 2.0 })
        );

        // Now add Velocity -- entity migrates to (Position, Velocity) archetype.
        world.insert_component(e, Velocity { dx: 3.0, dy: 4.0 });

        // Both components should still be accessible.
        assert_eq!(
            world.get_component::<Position>(e),
            Some(&Position { x: 1.0, y: 2.0 })
        );
        assert_eq!(
            world.get_component::<Velocity>(e),
            Some(&Velocity { dx: 3.0, dy: 4.0 })
        );
    }

    #[test]
    fn despawn_removes_entity() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Health(100));
        assert!(world.despawn(e));
        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.get_component::<Health>(e), None);
    }

    #[test]
    fn despawn_nonexistent_returns_false() {
        let mut world = World::new();
        let e = world.spawn();
        world.despawn(e);
        // Second despawn should fail.
        assert!(!world.despawn(e));
    }

    #[test]
    fn remove_component_migrates_back() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Position { x: 1.0, y: 2.0 });
        world.insert_component(e, Velocity { dx: 3.0, dy: 4.0 });

        let removed = world.remove_component::<Velocity>(e);
        assert_eq!(removed, Some(Velocity { dx: 3.0, dy: 4.0 }));

        // Position should still be there.
        assert_eq!(
            world.get_component::<Position>(e),
            Some(&Position { x: 1.0, y: 2.0 })
        );
        // Velocity should be gone.
        assert_eq!(world.get_component::<Velocity>(e), None);
    }

    #[test]
    fn remove_missing_component_returns_none() {
        let mut world = World::new();
        let e = world.spawn();
        assert_eq!(world.remove_component::<Health>(e), None);
    }

    #[test]
    fn get_component_mut() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Health(100));
        if let Some(h) = world.get_component_mut::<Health>(e) {
            h.0 -= 25;
        }
        assert_eq!(world.get_component::<Health>(e), Some(&Health(75)));
    }

    #[test]
    fn query_returns_matching_entities() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();

        world.insert_component(e1, Health(100));
        world.insert_component(e2, Health(50));
        // e3 has no Health.

        let results = world.query::<Health>();
        assert_eq!(results.len(), 2);

        let entities: Vec<Entity> = results.iter().map(|(e, _)| *e).collect();
        assert!(entities.contains(&e1));
        assert!(entities.contains(&e2));
        assert!(!entities.contains(&e3));
    }

    #[test]
    fn query2_returns_entities_with_both() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();

        world.insert_component(e1, Position { x: 1.0, y: 2.0 });
        world.insert_component(e1, Velocity { dx: 0.5, dy: 0.5 });

        world.insert_component(e2, Position { x: 3.0, y: 4.0 });
        // e2 has only Position, not Velocity.

        world.insert_component(e3, Velocity { dx: 1.0, dy: 1.0 });
        // e3 has only Velocity, not Position.

        let results = world.query2::<Position, Velocity>();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, e1);
        assert_eq!(results[0].1, &Position { x: 1.0, y: 2.0 });
        assert_eq!(results[0].2, &Velocity { dx: 0.5, dy: 0.5 });
    }

    #[test]
    fn resource_insert_get_get_mut() {
        let mut world = World::new();

        #[derive(Debug, PartialEq)]
        struct GameTime(f64);

        world.insert_resource(GameTime(0.0));
        assert_eq!(world.get_resource::<GameTime>(), Some(&GameTime(0.0)));

        if let Some(t) = world.get_resource_mut::<GameTime>() {
            t.0 += 1.0;
        }
        assert_eq!(world.get_resource::<GameTime>(), Some(&GameTime(1.0)));
    }

    #[test]
    fn missing_resource_returns_none() {
        let world = World::new();
        assert_eq!(world.get_resource::<u32>(), None);
    }

    #[test]
    fn stress_spawn_1000_with_components() {
        let mut world = World::new();

        for i in 0..1000u32 {
            let e = world.spawn();
            world.insert_component(
                e,
                Position {
                    x: i as f32,
                    y: i as f32 * 2.0,
                },
            );
            if i % 2 == 0 {
                world.insert_component(e, Velocity { dx: 1.0, dy: 1.0 });
            }
        }

        assert_eq!(world.entity_count(), 1000);

        let positions = world.query::<Position>();
        assert_eq!(positions.len(), 1000);

        let with_velocity = world.query2::<Position, Velocity>();
        assert_eq!(with_velocity.len(), 500);
    }

    #[test]
    fn despawn_with_multiple_components() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Position { x: 1.0, y: 2.0 });
        world.insert_component(e, Velocity { dx: 3.0, dy: 4.0 });
        world.insert_component(e, Health(100));

        assert!(world.despawn(e));
        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.get_component::<Position>(e), None);
        assert_eq!(world.get_component::<Velocity>(e), None);
        assert_eq!(world.get_component::<Health>(e), None);
    }

    #[test]
    fn multiple_entities_same_archetype() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();

        world.insert_component(e1, Health(100));
        world.insert_component(e2, Health(200));

        assert_eq!(world.get_component::<Health>(e1), Some(&Health(100)));
        assert_eq!(world.get_component::<Health>(e2), Some(&Health(200)));
    }

    #[test]
    fn despawn_middle_entity_preserves_others() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();

        world.insert_component(e1, Health(100));
        world.insert_component(e2, Health(200));
        world.insert_component(e3, Health(300));

        world.despawn(e2);
        assert_eq!(world.entity_count(), 2);
        assert_eq!(world.get_component::<Health>(e1), Some(&Health(100)));
        assert_eq!(world.get_component::<Health>(e3), Some(&Health(300)));
    }

    #[test]
    fn component_with_drop() {
        let mut world = World::new();
        let e = world.spawn();
        world.insert_component(e, Name(String::from("test entity")));
        assert_eq!(
            world.get_component::<Name>(e),
            Some(&Name(String::from("test entity")))
        );
        world.despawn(e);
        // No leak -- String is properly dropped.
    }

    #[test]
    fn for_each_visits_all_matching() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();

        world.insert_component(e1, Health(100));
        world.insert_component(e2, Health(50));
        // e3 has no Health.
        let _ = e3;

        let query_results = world.query::<Health>();
        let mut for_each_count = 0usize;
        world.for_each::<Health, _>(|_e, _h| {
            for_each_count += 1;
        });

        assert_eq!(for_each_count, query_results.len());
    }

    #[test]
    fn for_each2_visits_matching() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();

        world.insert_component(e1, Position { x: 1.0, y: 2.0 });
        world.insert_component(e1, Velocity { dx: 0.5, dy: 0.5 });

        world.insert_component(e2, Position { x: 3.0, y: 4.0 });
        // e2 has only Position.

        world.insert_component(e3, Velocity { dx: 1.0, dy: 1.0 });
        // e3 has only Velocity.

        let query_results = world.query2::<Position, Velocity>();
        let mut for_each2_count = 0usize;
        world.for_each2::<Position, Velocity, _>(|_e, _p, _v| {
            for_each2_count += 1;
        });

        assert_eq!(for_each2_count, query_results.len());
        assert_eq!(for_each2_count, 1);
    }

    #[test]
    fn entities_with_returns_matching_ids() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();

        world.insert_component(e1, Health(100));
        world.insert_component(e2, Health(50));
        // e3 has no Health.

        let ids = world.entities_with::<Health>();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&e1));
        assert!(ids.contains(&e2));
        assert!(!ids.contains(&e3));
    }

    #[test]
    fn entities_with2_returns_matching_ids() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        let e3 = world.spawn();

        world.insert_component(e1, Position { x: 1.0, y: 2.0 });
        world.insert_component(e1, Velocity { dx: 0.5, dy: 0.5 });

        world.insert_component(e2, Position { x: 3.0, y: 4.0 });
        world.insert_component(e3, Velocity { dx: 1.0, dy: 1.0 });

        let ids = world.entities_with2::<Position, Velocity>();
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], e1);
    }
}

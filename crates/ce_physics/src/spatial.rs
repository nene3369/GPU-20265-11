use ce_core::Entity;
use ce_math::Vec3;
use std::collections::HashMap;

/// Uniform spatial grid for broad-phase collision detection.
pub struct SpatialGrid {
    cell_size: f32,
    cells: HashMap<(i32, i32, i32), Vec<Entity>>,
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    fn cell_key(&self, pos: Vec3) -> (i32, i32, i32) {
        let inv = 1.0 / self.cell_size;
        (
            (pos.x * inv).floor() as i32,
            (pos.y * inv).floor() as i32,
            (pos.z * inv).floor() as i32,
        )
    }

    pub fn insert(&mut self, entity: Entity, position: Vec3) {
        let key = self.cell_key(position);
        self.cells.entry(key).or_default().push(entity);
    }

    /// Find all entities within `radius` of `position`.
    pub fn query_radius(&self, position: Vec3, radius: f32) -> Vec<Entity> {
        let mut result = Vec::new();
        let cells_to_check = (radius / self.cell_size).ceil() as i32 + 1;
        let center = self.cell_key(position);

        for dx in -cells_to_check..=cells_to_check {
            for dy in -cells_to_check..=cells_to_check {
                for dz in -cells_to_check..=cells_to_check {
                    let key = (center.0 + dx, center.1 + dy, center.2 + dz);
                    if let Some(entities) = self.cells.get(&key) {
                        result.extend(entities);
                    }
                }
            }
        }
        result
    }

    /// Get potential collision pairs (entities in same or adjacent cells).
    pub fn broad_phase_pairs(&self) -> Vec<(Entity, Entity)> {
        let mut pairs = Vec::new();
        for (key, entities) in &self.cells {
            // Pairs within same cell
            for i in 0..entities.len() {
                for j in (i + 1)..entities.len() {
                    pairs.push((entities[i], entities[j]));
                }
            }
            // Pairs with adjacent cells (only check positive direction to avoid duplicates)
            for &(dx, dy, dz) in &[
                (1, 0, 0),
                (0, 1, 0),
                (0, 0, 1),
                (1, 1, 0),
                (1, 0, 1),
                (0, 1, 1),
                (1, 1, 1),
            ] {
                let adj_key = (key.0 + dx, key.1 + dy, key.2 + dz);
                if let Some(adj_entities) = self.cells.get(&adj_key) {
                    for &a in entities {
                        for &b in adj_entities {
                            pairs.push((a, b));
                        }
                    }
                }
            }
        }
        pairs
    }

    pub fn entity_count(&self) -> usize {
        self.cells.values().map(|v| v.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ce_core::Entities;

    fn make_entity(entities: &mut Entities) -> Entity {
        entities.alloc()
    }

    #[test]
    fn insert_and_query_radius_finds_nearby_entities() {
        let mut entities = Entities::new();
        let e1 = make_entity(&mut entities);
        let e2 = make_entity(&mut entities);

        let mut grid = SpatialGrid::new(10.0);
        grid.insert(e1, Vec3::new(5.0, 5.0, 5.0));
        grid.insert(e2, Vec3::new(8.0, 5.0, 5.0));

        let found = grid.query_radius(Vec3::new(5.0, 5.0, 5.0), 5.0);
        assert!(found.contains(&e1));
        assert!(found.contains(&e2));
    }

    #[test]
    fn query_radius_does_not_find_distant_entities() {
        let mut entities = Entities::new();
        let e1 = make_entity(&mut entities);
        let e2 = make_entity(&mut entities);

        let mut grid = SpatialGrid::new(10.0);
        grid.insert(e1, Vec3::new(0.0, 0.0, 0.0));
        grid.insert(e2, Vec3::new(100.0, 100.0, 100.0));

        let found = grid.query_radius(Vec3::new(0.0, 0.0, 0.0), 5.0);
        assert!(found.contains(&e1));
        assert!(!found.contains(&e2));
    }

    #[test]
    fn broad_phase_pairs_returns_pairs_in_same_cell() {
        let mut entities = Entities::new();
        let e1 = make_entity(&mut entities);
        let e2 = make_entity(&mut entities);

        let mut grid = SpatialGrid::new(10.0);
        grid.insert(e1, Vec3::new(1.0, 1.0, 1.0));
        grid.insert(e2, Vec3::new(2.0, 2.0, 2.0));

        let pairs = grid.broad_phase_pairs();
        assert!(pairs.contains(&(e1, e2)) || pairs.contains(&(e2, e1)));
    }

    #[test]
    fn clear_empties_the_grid() {
        let mut entities = Entities::new();
        let e1 = make_entity(&mut entities);

        let mut grid = SpatialGrid::new(10.0);
        grid.insert(e1, Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(grid.entity_count(), 1);

        grid.clear();
        assert_eq!(grid.entity_count(), 0);
    }
}

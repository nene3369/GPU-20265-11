/// Lightweight entity handle with ABA-safety via generational indexing.
///
/// Two `Entity` values with the same `index` but different `generation`
/// fields refer to different logical entities, preventing use-after-free
/// style bugs where a stale handle accidentally addresses a recycled slot.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Entity {
    pub index: u32,
    pub generation: u32,
}

/// Entity allocator with free-list recycling and generation tracking.
///
/// Allocates new [`Entity`] handles via [`alloc`](Entities::alloc) and
/// reclaims them via [`free`](Entities::free). Freed indices are pushed
/// onto an internal free list so that subsequent allocations reuse slots,
/// while a bumped generation field ensures stale handles are detected.
pub struct Entities {
    /// Current generation for each index that has ever been allocated.
    generations: Vec<u32>,
    /// Stack of previously-freed indices available for reuse.
    free_list: Vec<u32>,
    /// Number of currently alive entities.
    alive_count: usize,
}

impl Entities {
    /// Creates a new, empty allocator.
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_list: Vec::new(),
            alive_count: 0,
        }
    }

    /// Allocates a fresh [`Entity`].
    ///
    /// If the free list is non-empty the most recently freed index is
    /// recycled (with its already-bumped generation). Otherwise a brand-new
    /// index is appended.
    pub fn alloc(&mut self) -> Entity {
        self.alive_count += 1;

        if let Some(index) = self.free_list.pop() {
            Entity {
                index,
                generation: self.generations[index as usize],
            }
        } else {
            let index = self.generations.len() as u32;
            self.generations.push(0);
            Entity {
                index,
                generation: 0,
            }
        }
    }

    /// Frees an entity, making its index available for future reuse.
    ///
    /// Returns `true` if the entity was alive and has now been freed.
    /// Returns `false` if the entity was already dead (stale generation
    /// or double-free).
    pub fn free(&mut self, entity: Entity) -> bool {
        let idx = entity.index as usize;
        if idx >= self.generations.len() {
            return false;
        }
        if self.generations[idx] != entity.generation {
            return false;
        }
        // Bump generation so the old handle is invalidated.
        self.generations[idx] = self.generations[idx].wrapping_add(1);
        self.free_list.push(entity.index);
        self.alive_count -= 1;
        true
    }

    /// Returns `true` if `entity` refers to a currently alive entity.
    pub fn is_alive(&self, entity: Entity) -> bool {
        let idx = entity.index as usize;
        idx < self.generations.len() && self.generations[idx] == entity.generation
    }

    /// Returns the number of currently alive entities.
    pub fn len(&self) -> usize {
        self.alive_count
    }

    /// Returns `true` if no entities are currently alive.
    pub fn is_empty(&self) -> bool {
        self.alive_count == 0
    }
}

impl Default for Entities {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_alloc() {
        let mut entities = Entities::new();
        let e = entities.alloc();
        assert_eq!(e.index, 0);
        assert_eq!(e.generation, 0);
        assert_eq!(entities.len(), 1);
    }

    #[test]
    fn sequential_alloc() {
        let mut entities = Entities::new();
        let a = entities.alloc();
        let b = entities.alloc();
        let c = entities.alloc();
        assert_eq!(a.index, 0);
        assert_eq!(b.index, 1);
        assert_eq!(c.index, 2);
        assert_eq!(entities.len(), 3);
    }

    #[test]
    fn free_returns_true_for_alive() {
        let mut entities = Entities::new();
        let e = entities.alloc();
        assert!(entities.free(e));
        assert_eq!(entities.len(), 0);
    }

    #[test]
    fn generation_increments_on_free() {
        let mut entities = Entities::new();
        let e = entities.alloc();
        assert_eq!(e.generation, 0);
        entities.free(e);
        // Allocate again -- should reuse index 0 with generation 1.
        let e2 = entities.alloc();
        assert_eq!(e2.index, 0);
        assert_eq!(e2.generation, 1);
    }

    #[test]
    fn is_alive_after_free_returns_false() {
        let mut entities = Entities::new();
        let e = entities.alloc();
        assert!(entities.is_alive(e));
        entities.free(e);
        assert!(!entities.is_alive(e));
    }

    #[test]
    fn free_list_recycling() {
        let mut entities = Entities::new();
        let a = entities.alloc();
        let _b = entities.alloc();
        entities.free(a);
        // Next alloc should reuse index 0.
        let c = entities.alloc();
        assert_eq!(c.index, a.index);
        assert_eq!(c.generation, a.generation + 1);
    }

    #[test]
    fn double_free_returns_false() {
        let mut entities = Entities::new();
        let e = entities.alloc();
        assert!(entities.free(e));
        assert!(!entities.free(e));
    }

    #[test]
    fn stale_handle_is_not_alive() {
        let mut entities = Entities::new();
        let old = entities.alloc();
        entities.free(old);
        let _new = entities.alloc();
        // old handle has generation 0, slot now has generation 1.
        assert!(!entities.is_alive(old));
    }

    #[test]
    fn alloc_many_free_all_alloc_again() {
        let mut entities = Entities::new();
        let count = 100;
        let mut handles: Vec<Entity> = (0..count).map(|_| entities.alloc()).collect();
        assert_eq!(entities.len(), count);

        // Free all.
        for &e in &handles {
            assert!(entities.free(e));
        }
        assert_eq!(entities.len(), 0);

        // All old handles should be dead.
        for &e in &handles {
            assert!(!entities.is_alive(e));
        }

        // Allocate the same number again -- all should reuse slots.
        handles.clear();
        for _ in 0..count {
            handles.push(entities.alloc());
        }
        assert_eq!(entities.len(), count);

        // Every recycled handle should have generation 1.
        for e in &handles {
            assert_eq!(e.generation, 1);
        }
    }

    #[test]
    fn free_entity_with_out_of_range_index() {
        let mut entities = Entities::new();
        let bogus = Entity {
            index: 999,
            generation: 0,
        };
        assert!(!entities.free(bogus));
    }

    #[test]
    fn is_empty_on_new_allocator() {
        let entities = Entities::new();
        assert!(entities.is_empty());
        assert_eq!(entities.len(), 0);
    }

    #[test]
    fn len_tracks_correctly_through_mixed_ops() {
        let mut entities = Entities::new();
        let a = entities.alloc();
        let b = entities.alloc();
        let c = entities.alloc();
        assert_eq!(entities.len(), 3);

        entities.free(b);
        assert_eq!(entities.len(), 2);

        let _d = entities.alloc();
        assert_eq!(entities.len(), 3);

        entities.free(a);
        entities.free(c);
        assert_eq!(entities.len(), 1);
    }
}

use ce_core::Entity;

/// Marks an entity as having a parent.
#[derive(Debug, Clone, Copy)]
pub struct Parent(pub Entity);

/// Marks an entity as having children.
#[derive(Debug, Clone)]
pub struct Children(pub Vec<Entity>);

impl Children {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, child: Entity) {
        if !self.0.contains(&child) {
            self.0.push(child);
        }
    }

    pub fn remove(&mut self, child: Entity) {
        self.0.retain(|e| *e != child);
    }

    pub fn contains(&self, child: Entity) -> bool {
        self.0.contains(&child)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.0.iter()
    }
}

impl Default for Children {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entity(index: u32) -> Entity {
        Entity {
            index,
            generation: 0,
        }
    }

    #[test]
    fn children_add_and_contains() {
        let mut children = Children::new();
        let e = make_entity(1);
        assert!(!children.contains(e));
        children.add(e);
        assert!(children.contains(e));
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn children_remove() {
        let mut children = Children::new();
        let e1 = make_entity(1);
        let e2 = make_entity(2);
        children.add(e1);
        children.add(e2);
        assert_eq!(children.len(), 2);

        children.remove(e1);
        assert!(!children.contains(e1));
        assert!(children.contains(e2));
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn children_no_duplicates() {
        let mut children = Children::new();
        let e = make_entity(5);
        children.add(e);
        children.add(e);
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn parent_stores_entity() {
        let e = make_entity(42);
        let parent = Parent(e);
        assert_eq!(parent.0.index, 42);
        assert_eq!(parent.0.generation, 0);
    }

    #[test]
    fn children_is_empty() {
        let children = Children::new();
        assert!(children.is_empty());
    }

    #[test]
    fn children_iter() {
        let mut children = Children::new();
        let e1 = make_entity(1);
        let e2 = make_entity(2);
        let e3 = make_entity(3);
        children.add(e1);
        children.add(e2);
        children.add(e3);

        let collected: Vec<&Entity> = children.iter().collect();
        assert_eq!(collected.len(), 3);
    }

    #[test]
    fn children_default_is_empty() {
        let children = Children::default();
        assert!(children.is_empty());
        assert_eq!(children.len(), 0);
    }

    #[test]
    fn children_remove_nonexistent_is_noop() {
        let mut children = Children::new();
        let e1 = make_entity(1);
        let e2 = make_entity(2);
        children.add(e1);
        children.remove(e2); // should not panic
        assert_eq!(children.len(), 1);
        assert!(children.contains(e1));
    }
}

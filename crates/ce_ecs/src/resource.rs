use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Marker trait for resources (global singleton data).
/// Auto-implemented for any `'static + Send + Sync` type.
pub trait Resource: 'static + Send + Sync {}
impl<T: 'static + Send + Sync> Resource for T {}

/// Type-keyed storage for global resources.
///
/// Resources are singleton values stored by their concrete type, providing
/// shared or exclusive access through the `World`. Unlike components,
/// resources are not associated with any particular entity.
pub struct Resources {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Resources {
    /// Creates a new, empty resource storage.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Inserts a resource value, replacing any previous value of the same type.
    pub fn insert<T: Resource>(&mut self, resource: T) {
        self.map.insert(TypeId::of::<T>(), Box::new(resource));
    }

    /// Returns a reference to the resource of type `T`, if it exists.
    pub fn get<T: Resource>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }

    /// Returns a mutable reference to the resource of type `T`, if it exists.
    pub fn get_mut<T: Resource>(&mut self) -> Option<&mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }

    /// Returns `true` if a resource of type `T` has been inserted.
    pub fn contains<T: Resource>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<T>())
    }

    /// Removes the resource of type `T`, returning it if it existed.
    pub fn remove<T: Resource>(&mut self) -> Option<T> {
        self.map
            .remove(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut res = Resources::new();
        res.insert(42i32);
        assert_eq!(res.get::<i32>(), Some(&42));
    }

    #[test]
    fn get_missing_returns_none() {
        let res = Resources::new();
        assert_eq!(res.get::<i32>(), None);
    }

    #[test]
    fn get_mut_modifies() {
        let mut res = Resources::new();
        res.insert(String::from("hello"));
        if let Some(s) = res.get_mut::<String>() {
            s.push_str(" world");
        }
        assert_eq!(res.get::<String>().map(|s| s.as_str()), Some("hello world"));
    }

    #[test]
    fn insert_replaces() {
        let mut res = Resources::new();
        res.insert(10u32);
        res.insert(20u32);
        assert_eq!(res.get::<u32>(), Some(&20));
    }

    #[test]
    fn contains() {
        let mut res = Resources::new();
        assert!(!res.contains::<f64>());
        res.insert(3.14f64);
        assert!(res.contains::<f64>());
    }

    #[test]
    fn remove_returns_value() {
        let mut res = Resources::new();
        res.insert(99u64);
        let removed = res.remove::<u64>();
        assert_eq!(removed, Some(99));
        assert!(!res.contains::<u64>());
    }

    #[test]
    fn remove_missing_returns_none() {
        let mut res = Resources::new();
        assert_eq!(res.remove::<i32>(), None);
    }

    #[test]
    fn multiple_types() {
        let mut res = Resources::new();
        res.insert(1i32);
        res.insert(2.0f64);
        res.insert(String::from("three"));

        assert_eq!(res.get::<i32>(), Some(&1));
        assert_eq!(res.get::<f64>(), Some(&2.0));
        assert_eq!(res.get::<String>().map(|s| s.as_str()), Some("three"));
    }
}

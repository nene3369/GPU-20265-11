use std::any::TypeId;

/// Strongly-typed wrapper around [`std::any::TypeId`] used to identify
/// component types within the ECS at runtime.
///
/// `ComponentTypeId` provides a thin, zero-cost abstraction that makes
/// intent explicit when passing type identifiers through the engine.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct ComponentTypeId(TypeId);

impl ComponentTypeId {
    /// Constructs the [`ComponentTypeId`] for the concrete type `T`.
    ///
    /// `T` must be `'static` because [`TypeId`] requires it.
    pub fn of<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_type_produces_equal_ids() {
        assert_eq!(ComponentTypeId::of::<u32>(), ComponentTypeId::of::<u32>());
    }

    #[test]
    fn different_types_produce_different_ids() {
        assert_ne!(ComponentTypeId::of::<u32>(), ComponentTypeId::of::<i32>());
    }

    #[test]
    fn copy_semantics() {
        let a = ComponentTypeId::of::<String>();
        let b = a; // Copy
        assert_eq!(a, b);
    }

    #[test]
    fn hash_is_consistent() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ComponentTypeId::of::<f64>());
        assert!(set.contains(&ComponentTypeId::of::<f64>()));
        assert!(!set.contains(&ComponentTypeId::of::<f32>()));
    }

    #[test]
    fn ord_is_deterministic() {
        let a = ComponentTypeId::of::<u8>();
        let b = ComponentTypeId::of::<u16>();
        // We don't know which is less, but the ordering must be total
        // and consistent across invocations within the same process.
        let first = a.cmp(&b);
        let second = a.cmp(&b);
        assert_eq!(first, second);
    }

    #[test]
    fn debug_format_is_non_empty() {
        let id = ComponentTypeId::of::<Vec<u8>>();
        let debug = format!("{:?}", id);
        assert!(!debug.is_empty());
    }
}

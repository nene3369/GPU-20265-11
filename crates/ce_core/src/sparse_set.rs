/// A generic sparse set providing O(1) insert, remove, look-up, and
/// membership testing while keeping values packed in a dense array for
/// cache-friendly iteration.
///
/// The data structure maintains three parallel collections:
///
/// * `sparse` -- maps a sparse index to the corresponding position in the
///   dense array (`None` if the index is absent).
/// * `dense`  -- tightly packed values.
/// * `indices` -- reverse mapping from dense position back to the sparse
///   index, enabling O(1) removal via swap-remove.
pub struct SparseSet<T> {
    sparse: Vec<Option<usize>>,
    dense: Vec<T>,
    indices: Vec<usize>,
}

impl<T> SparseSet<T> {
    /// Creates a new, empty sparse set.
    pub fn new() -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Inserts `value` at the given sparse `index`.
    ///
    /// If the index already exists its value is overwritten in place and
    /// the previous value is returned. Otherwise the value is appended to
    /// the dense storage and `None` is returned.
    pub fn insert(&mut self, index: usize, value: T) -> Option<T> {
        // Grow the sparse array if necessary.
        if index >= self.sparse.len() {
            self.sparse.resize_with(index + 1, || None);
        }

        if let Some(dense_idx) = self.sparse[index] {
            // Overwrite existing value.
            let old = std::mem::replace(&mut self.dense[dense_idx], value);
            Some(old)
        } else {
            let dense_idx = self.dense.len();
            self.dense.push(value);
            self.indices.push(index);
            self.sparse[index] = Some(dense_idx);
            None
        }
    }

    /// Removes the value at the given sparse `index`, returning it if it
    /// was present.
    ///
    /// Internally this performs a swap-remove on the dense array so that it
    /// remains tightly packed.
    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.sparse.len() {
            return None;
        }

        let dense_idx = self.sparse[index]?;

        // Clear the sparse slot for the removed index.
        self.sparse[index] = None;

        let last_dense = self.dense.len() - 1;

        if dense_idx != last_dense {
            // Swap the target with the last element in both dense + indices.
            self.dense.swap(dense_idx, last_dense);
            self.indices.swap(dense_idx, last_dense);

            // Update the sparse pointer for the element that was moved.
            let moved_sparse_idx = self.indices[dense_idx];
            self.sparse[moved_sparse_idx] = Some(dense_idx);
        }

        self.indices.pop();
        // `pop` returns the value that is now at the tail (our target after
        // the swap, or the only element if there was no swap).
        self.dense.pop()
    }

    /// Returns a reference to the value at the given sparse `index`, or
    /// `None` if the index is absent.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.sparse.len() {
            return None;
        }
        let dense_idx = self.sparse[index]?;
        Some(&self.dense[dense_idx])
    }

    /// Returns a mutable reference to the value at the given sparse
    /// `index`, or `None` if the index is absent.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.sparse.len() {
            return None;
        }
        let dense_idx = self.sparse[index]?;
        Some(&mut self.dense[dense_idx])
    }

    /// Returns `true` if the sparse set contains a value at `index`.
    pub fn contains(&self, index: usize) -> bool {
        index < self.sparse.len() && self.sparse[index].is_some()
    }

    /// Returns the number of values currently stored.
    pub fn len(&self) -> usize {
        self.dense.len()
    }

    /// Returns `true` if the set contains no values.
    pub fn is_empty(&self) -> bool {
        self.dense.is_empty()
    }

    /// Returns an iterator over `(sparse_index, &value)` pairs.
    ///
    /// The iteration order corresponds to the internal dense layout and is
    /// **not** guaranteed to follow insertion order after removals.
    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.indices.iter().copied().zip(self.dense.iter())
    }
}

impl<T> Default for SparseSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut set = SparseSet::new();
        assert!(set.insert(5, "hello").is_none());
        assert_eq!(set.get(5), Some(&"hello"));
    }

    #[test]
    fn insert_overwrites_existing() {
        let mut set = SparseSet::new();
        set.insert(3, 10);
        let old = set.insert(3, 20);
        assert_eq!(old, Some(10));
        assert_eq!(set.get(3), Some(&20));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn remove_returns_value() {
        let mut set = SparseSet::new();
        set.insert(7, 42);
        assert_eq!(set.remove(7), Some(42));
        assert_eq!(set.get(7), None);
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn remove_absent_returns_none() {
        let mut set: SparseSet<i32> = SparseSet::new();
        assert_eq!(set.remove(0), None);
        assert_eq!(set.remove(999), None);
    }

    #[test]
    fn insert_after_remove_reuses_dense_slot() {
        let mut set = SparseSet::new();
        set.insert(0, "a");
        set.insert(1, "b");
        set.insert(2, "c");
        set.remove(1);
        // Dense should have length 2 now.
        assert_eq!(set.len(), 2);

        set.insert(3, "d");
        // Dense should have length 3; the internal layout is packed.
        assert_eq!(set.len(), 3);
        assert_eq!(set.get(0), Some(&"a"));
        assert_eq!(set.get(2), Some(&"c"));
        assert_eq!(set.get(3), Some(&"d"));
        assert!(!set.contains(1));
    }

    #[test]
    fn contains() {
        let mut set = SparseSet::new();
        assert!(!set.contains(0));
        set.insert(0, 1);
        assert!(set.contains(0));
        assert!(!set.contains(1));
        set.remove(0);
        assert!(!set.contains(0));
    }

    #[test]
    fn get_mut() {
        let mut set = SparseSet::new();
        set.insert(2, 100);
        if let Some(val) = set.get_mut(2) {
            *val = 200;
        }
        assert_eq!(set.get(2), Some(&200));
    }

    #[test]
    fn empty_set_operations() {
        let mut set: SparseSet<i32> = SparseSet::new();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
        assert_eq!(set.get(0), None);
        assert_eq!(set.get_mut(0), None);
        assert_eq!(set.remove(0), None);
        assert!(!set.contains(0));
        assert_eq!(set.iter().count(), 0);
    }

    #[test]
    fn many_elements() {
        let mut set = SparseSet::new();
        let count = 1_000;

        for i in 0..count {
            set.insert(i, i * 2);
        }
        assert_eq!(set.len(), count);

        for i in 0..count {
            assert_eq!(set.get(i), Some(&(i * 2)));
        }

        // Remove every other element.
        for i in (0..count).step_by(2) {
            assert_eq!(set.remove(i), Some(i * 2));
        }
        assert_eq!(set.len(), count / 2);

        // Verify remaining elements.
        for i in 0..count {
            if i % 2 == 0 {
                assert!(!set.contains(i));
            } else {
                assert!(set.contains(i));
                assert_eq!(set.get(i), Some(&(i * 2)));
            }
        }
    }

    #[test]
    fn iter_yields_all_entries() {
        let mut set = SparseSet::new();
        set.insert(10, "a");
        set.insert(20, "b");
        set.insert(30, "c");

        let mut collected: Vec<(usize, &str)> = set.iter().map(|(i, v)| (i, *v)).collect();
        collected.sort_by_key(|&(i, _)| i);
        assert_eq!(collected, vec![(10, "a"), (20, "b"), (30, "c")]);
    }

    #[test]
    fn iter_after_removal() {
        let mut set = SparseSet::new();
        set.insert(1, 10);
        set.insert(2, 20);
        set.insert(3, 30);
        set.remove(2);

        let mut collected: Vec<(usize, i32)> = set.iter().map(|(i, &v)| (i, v)).collect();
        collected.sort_by_key(|&(i, _)| i);
        assert_eq!(collected, vec![(1, 10), (3, 30)]);
    }

    #[test]
    fn remove_last_element() {
        let mut set = SparseSet::new();
        set.insert(0, 1);
        assert_eq!(set.remove(0), Some(1));
        assert!(set.is_empty());
    }

    #[test]
    fn remove_first_of_many_preserves_others() {
        let mut set = SparseSet::new();
        set.insert(0, "x");
        set.insert(1, "y");
        set.insert(2, "z");
        set.remove(0);

        assert!(!set.contains(0));
        assert_eq!(set.get(1), Some(&"y"));
        assert_eq!(set.get(2), Some(&"z"));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn sparse_indices_with_gaps() {
        let mut set = SparseSet::new();
        set.insert(0, 'a');
        set.insert(1000, 'b');
        set.insert(500, 'c');

        assert_eq!(set.len(), 3);
        assert_eq!(set.get(0), Some(&'a'));
        assert_eq!(set.get(500), Some(&'c'));
        assert_eq!(set.get(1000), Some(&'b'));
        assert!(!set.contains(999));
    }

    #[test]
    fn insert_remove_insert_same_index() {
        let mut set = SparseSet::new();
        set.insert(5, 100);
        assert_eq!(set.remove(5), Some(100));
        assert!(!set.contains(5));
        set.insert(5, 200);
        assert_eq!(set.get(5), Some(&200));
        assert_eq!(set.len(), 1);
    }
}

use std::alloc::Layout;
use std::any::TypeId;

/// Marker trait for components. Auto-implemented for any `'static + Send + Sync` type.
pub trait Component: 'static + Send + Sync {}
impl<T: 'static + Send + Sync> Component for T {}

/// Type-erased column storage for one component type within an archetype.
///
/// Stores data as raw bytes with proper layout and drop handling, enabling
/// heterogeneous component storage in a single archetype table while
/// maintaining cache-friendly SoA (Struct of Arrays) access patterns.
pub struct ComponentColumn {
    pub(crate) data: Vec<u8>,
    pub(crate) item_layout: Layout,
    pub(crate) len: usize,
    pub(crate) drop_fn: Option<unsafe fn(*mut u8)>,
    pub(crate) type_id: TypeId,
}

impl ComponentColumn {
    /// Creates a new, empty column for component type `T`.
    ///
    /// Captures the layout and drop glue for `T` so that values can be
    /// stored and cleaned up correctly through type-erased byte storage.
    pub fn new<T: Component>() -> Self {
        let item_layout = Layout::new::<T>();

        // Only store a drop function if the type actually needs dropping.
        let drop_fn: Option<unsafe fn(*mut u8)> = if std::mem::needs_drop::<T>() {
            Some(|ptr: *mut u8| {
                // SAFETY: The caller guarantees that `ptr` points to a valid,
                // aligned, initialized `T` value. We cast back from `*mut u8`
                // to `*mut T` and drop it in place.
                unsafe {
                    std::ptr::drop_in_place(ptr as *mut T);
                }
            })
        } else {
            None
        };

        Self {
            data: Vec::new(),
            item_layout,
            len: 0,
            drop_fn,
            type_id: TypeId::of::<T>(),
        }
    }

    /// Returns the padded stride for each item (accounts for alignment).
    fn stride(&self) -> usize {
        if self.item_layout.size() == 0 {
            // Zero-sized types: no bytes to store, but we still track count.
            return 0;
        }
        // Pad size up to alignment so consecutive items are correctly aligned.
        let size = self.item_layout.size();
        let align = self.item_layout.align();
        size.div_ceil(align) * align
    }

    /// Appends a component value to the end of this column.
    ///
    /// # Panics
    /// Panics if `T` does not match the type this column was created for.
    pub fn push<T: Component>(&mut self, value: T) {
        assert_eq!(
            TypeId::of::<T>(),
            self.type_id,
            "ComponentColumn::push type mismatch"
        );

        let stride = self.stride();

        if stride > 0 {
            // Ensure we have enough room in the byte buffer.
            let needed = (self.len + 1) * stride;
            if self.data.len() < needed {
                self.data.resize(needed, 0);
            }

            let offset = self.len * stride;
            let dst = &mut self.data[offset] as *mut u8;

            // SAFETY: `dst` points into our Vec<u8> at a properly aligned offset
            // (since stride is a multiple of alignment and Vec<u8> data starts
            // at an alignment of 1, we realign within stride). The value is
            // written as raw bytes and we take ownership by calling `forget`.
            unsafe {
                std::ptr::copy_nonoverlapping(
                    &value as *const T as *const u8,
                    dst,
                    std::mem::size_of::<T>(),
                );
            }
        }

        // Prevent T's destructor from running since we've moved the bytes.
        std::mem::forget(value);
        self.len += 1;
    }

    /// Returns a reference to the component at `index`, or `None` if out of bounds.
    ///
    /// # Panics
    /// Panics if `T` does not match the type this column was created for.
    pub fn get<T: Component>(&self, index: usize) -> Option<&T> {
        assert_eq!(
            TypeId::of::<T>(),
            self.type_id,
            "ComponentColumn::get type mismatch"
        );

        if index >= self.len {
            return None;
        }

        let stride = self.stride();
        if stride == 0 {
            // SAFETY: For ZSTs, any properly aligned non-null pointer is valid.
            // We use a well-aligned dangling pointer.
            unsafe {
                return Some(&*(std::ptr::NonNull::<T>::dangling().as_ptr()));
            }
        }

        let offset = index * stride;
        let ptr = &self.data[offset] as *const u8;

        // SAFETY: We verified that `index < self.len`, so the data at `offset`
        // is an initialized `T`. The pointer is derived from our Vec's contiguous
        // buffer and the lifetime is tied to `&self`.
        unsafe { Some(&*(ptr as *const T)) }
    }

    /// Returns a mutable reference to the component at `index`, or `None` if out of bounds.
    ///
    /// # Panics
    /// Panics if `T` does not match the type this column was created for.
    pub fn get_mut<T: Component>(&mut self, index: usize) -> Option<&mut T> {
        assert_eq!(
            TypeId::of::<T>(),
            self.type_id,
            "ComponentColumn::get_mut type mismatch"
        );

        if index >= self.len {
            return None;
        }

        let stride = self.stride();
        if stride == 0 {
            // SAFETY: For ZSTs, any properly aligned non-null pointer is valid.
            unsafe {
                return Some(&mut *(std::ptr::NonNull::<T>::dangling().as_ptr()));
            }
        }

        let offset = index * stride;
        let ptr = &mut self.data[offset] as *mut u8;

        // SAFETY: Same as `get`, but we have `&mut self` so exclusive access is
        // guaranteed. The returned `&mut T` borrows from `self` exclusively.
        unsafe { Some(&mut *(ptr as *mut T)) }
    }

    /// Removes the element at `index` by swapping it with the last element
    /// and then dropping the removed value.
    ///
    /// This is O(1) but does not preserve ordering.
    ///
    /// # Panics
    /// Panics if `index >= self.len`.
    pub fn swap_remove(&mut self, index: usize) {
        assert!(
            index < self.len,
            "ComponentColumn::swap_remove index {} out of bounds (len {})",
            index,
            self.len
        );

        let stride = self.stride();

        if stride > 0 {
            let last = self.len - 1;

            if index != last {
                let (idx_offset, last_offset) = (index * stride, last * stride);
                // SAFETY: Both regions are within bounds, non-overlapping (index != last),
                // and properly sized. We swap the raw bytes.
                unsafe {
                    let ptr = self.data.as_mut_ptr();
                    std::ptr::swap_nonoverlapping(
                        ptr.add(idx_offset),
                        ptr.add(last_offset),
                        stride,
                    );
                }
            }

            // Drop the value now at the last position (the one being removed).
            if let Some(drop_fn) = self.drop_fn {
                let last_offset = (self.len - 1) * stride;
                let ptr = &mut self.data[last_offset] as *mut u8;
                // SAFETY: The pointer targets a valid, initialized value of our
                // stored type. After drop_in_place, the bytes are considered
                // uninitialized and we shrink `len` so they won't be accessed.
                unsafe {
                    drop_fn(ptr);
                }
            }

            // Shrink the byte buffer.
            self.data.truncate((self.len - 1) * stride);
        } else {
            // ZST: no bytes to move/drop, just decrement count.
            // But we still need to drop if the ZST has a Drop impl (unusual but possible).
            if let Some(drop_fn) = self.drop_fn {
                // SAFETY: For ZSTs, a dangling aligned pointer is valid for drop.
                unsafe {
                    let ptr = std::ptr::NonNull::<u8>::dangling().as_ptr();
                    drop_fn(ptr);
                }
            }
        }

        self.len -= 1;
    }

    /// Swap-removes the element at `index` and returns its raw bytes
    /// WITHOUT dropping the value. Used for entity migration.
    ///
    /// # Safety
    /// The caller must ensure the returned bytes are either pushed into
    /// a compatible column or properly dropped as the correct type.
    pub(crate) unsafe fn swap_remove_raw(&mut self, index: usize) -> Vec<u8> {
        assert!(
            index < self.len,
            "swap_remove_raw: index {} out of bounds (len {})",
            index,
            self.len
        );

        let stride = self.stride();

        if stride == 0 {
            self.len -= 1;
            return Vec::new();
        }

        let last = self.len - 1;

        if index != last {
            let (idx_offset, last_offset) = (index * stride, last * stride);
            // SAFETY: Both regions are within bounds and non-overlapping.
            let ptr = self.data.as_mut_ptr();
            std::ptr::swap_nonoverlapping(ptr.add(idx_offset), ptr.add(last_offset), stride);
        }

        // Copy the bytes at the last position (which is the value we removed).
        let last_offset = last * stride;
        let bytes = self.data[last_offset..last_offset + stride].to_vec();
        self.data.truncate(last * stride);
        self.len -= 1;
        bytes
    }

    /// Swap-removes the element at `index` without dropping OR returning it.
    /// Used when the caller has already extracted the value (e.g. via ptr::read)
    /// and only needs to keep columns in sync.
    ///
    /// # Safety
    /// The caller must have already taken ownership of the value at `index`
    /// (e.g. via `ptr::read`). This method will NOT drop the removed value.
    pub(crate) unsafe fn swap_remove_raw_nodrop(&mut self, index: usize) {
        assert!(
            index < self.len,
            "swap_remove_raw_nodrop: index {} out of bounds (len {})",
            index,
            self.len
        );

        let stride = self.stride();

        if stride == 0 {
            self.len -= 1;
            return;
        }

        let last = self.len - 1;

        if index != last {
            let (idx_offset, last_offset) = (index * stride, last * stride);
            let ptr = self.data.as_mut_ptr();
            std::ptr::swap_nonoverlapping(ptr.add(idx_offset), ptr.add(last_offset), stride);
        }

        // Truncate without dropping -- the value's ownership has been
        // transferred to the caller already.
        self.data.truncate(last * stride);
        self.len -= 1;
    }

    /// Pushes raw bytes as a component value without drop-check.
    /// Used internally for entity migration between archetypes.
    ///
    /// # Safety
    /// The caller must ensure `bytes` contains a valid representation of the
    /// component type this column was created for, with correct size/alignment.
    pub(crate) unsafe fn push_raw(&mut self, bytes: &[u8]) {
        let stride = self.stride();

        if stride > 0 {
            assert_eq!(
                bytes.len(),
                stride,
                "push_raw: byte length mismatch (expected {}, got {})",
                stride,
                bytes.len()
            );
            self.data.extend_from_slice(bytes);
        }

        self.len += 1;
    }

    /// Returns the item layout (for cloning empty columns).
    pub(crate) fn item_layout(&self) -> Layout {
        self.item_layout
    }

    /// Returns the drop function, if any (for cloning empty columns).
    pub(crate) fn drop_fn(&self) -> Option<unsafe fn(*mut u8)> {
        self.drop_fn
    }

    /// Returns the number of components stored in this column.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the column contains no components.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the `TypeId` of the component type this column stores.
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }
}

impl Drop for ComponentColumn {
    fn drop(&mut self) {
        if let Some(drop_fn) = self.drop_fn {
            let stride = self.stride();
            if stride > 0 {
                // SAFETY: We iterate over every initialized element and drop it.
                // Each pointer at `offset = i * stride` holds a valid value.
                for i in 0..self.len {
                    let offset = i * stride;
                    unsafe {
                        drop_fn(&mut self.data[offset] as *mut u8);
                    }
                }
            } else {
                // ZST with Drop: drop each logical element.
                for _ in 0..self.len {
                    unsafe {
                        let ptr = std::ptr::NonNull::<u8>::dangling().as_ptr();
                        drop_fn(ptr);
                    }
                }
            }
        }
        // Vec<u8> will be freed by its own Drop impl; the component values
        // have already been dropped above.
    }
}

// SAFETY: ComponentColumn is Send + Sync because:
// - It only stores Component types which are required to be Send + Sync
// - All internal state (Vec<u8>, Layout, etc.) is Send + Sync
unsafe impl Send for ComponentColumn {}
unsafe impl Sync for ComponentColumn {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn push_and_get_i32() {
        let mut col = ComponentColumn::new::<i32>();
        col.push(42i32);
        col.push(99i32);
        assert_eq!(col.len(), 2);
        assert_eq!(col.get::<i32>(0), Some(&42));
        assert_eq!(col.get::<i32>(1), Some(&99));
    }

    #[test]
    fn push_and_get_f64() {
        let mut col = ComponentColumn::new::<f64>();
        col.push(3.14f64);
        assert_eq!(col.get::<f64>(0), Some(&3.14));
    }

    #[test]
    fn push_and_get_string() {
        let mut col = ComponentColumn::new::<String>();
        col.push(String::from("hello"));
        col.push(String::from("world"));
        assert_eq!(col.get::<String>(0).map(|s| s.as_str()), Some("hello"));
        assert_eq!(col.get::<String>(1).map(|s| s.as_str()), Some("world"));
    }

    #[test]
    fn get_mut_modifies() {
        let mut col = ComponentColumn::new::<i32>();
        col.push(10i32);
        if let Some(v) = col.get_mut::<i32>(0) {
            *v = 20;
        }
        assert_eq!(col.get::<i32>(0), Some(&20));
    }

    #[test]
    fn get_out_of_bounds() {
        let col = ComponentColumn::new::<i32>();
        assert_eq!(col.get::<i32>(0), None);
    }

    #[test]
    fn swap_remove_last() {
        let mut col = ComponentColumn::new::<i32>();
        col.push(1i32);
        col.push(2i32);
        col.push(3i32);
        col.swap_remove(2);
        assert_eq!(col.len(), 2);
        assert_eq!(col.get::<i32>(0), Some(&1));
        assert_eq!(col.get::<i32>(1), Some(&2));
    }

    #[test]
    fn swap_remove_first() {
        let mut col = ComponentColumn::new::<i32>();
        col.push(1i32);
        col.push(2i32);
        col.push(3i32);
        col.swap_remove(0);
        assert_eq!(col.len(), 2);
        // Element 0 was swapped with last (3), then last dropped.
        assert_eq!(col.get::<i32>(0), Some(&3));
        assert_eq!(col.get::<i32>(1), Some(&2));
    }

    #[test]
    fn swap_remove_middle() {
        let mut col = ComponentColumn::new::<i32>();
        col.push(10i32);
        col.push(20i32);
        col.push(30i32);
        col.swap_remove(1);
        assert_eq!(col.len(), 2);
        assert_eq!(col.get::<i32>(0), Some(&10));
        assert_eq!(col.get::<i32>(1), Some(&30));
    }

    // Track drop calls for testing proper cleanup.
    static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug)]
    #[allow(dead_code)]
    struct DropTracker(u32);

    impl Drop for DropTracker {
        fn drop(&mut self) {
            DROP_COUNT.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn drop_called_on_swap_remove() {
        DROP_COUNT.store(0, Ordering::SeqCst);
        let mut col = ComponentColumn::new::<DropTracker>();
        col.push(DropTracker(1));
        col.push(DropTracker(2));
        col.push(DropTracker(3));
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 0);

        col.swap_remove(1);
        // Exactly one value should have been dropped.
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 1);

        // Remaining 2 will be dropped when column goes out of scope.
        drop(col);
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn drop_called_on_column_drop() {
        DROP_COUNT.store(0, Ordering::SeqCst);
        {
            let mut col = ComponentColumn::new::<DropTracker>();
            col.push(DropTracker(10));
            col.push(DropTracker(20));
        }
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn zero_sized_type() {
        #[derive(Debug, PartialEq)]
        struct Marker;

        let mut col = ComponentColumn::new::<Marker>();
        col.push(Marker);
        col.push(Marker);
        col.push(Marker);
        assert_eq!(col.len(), 3);
        assert_eq!(col.get::<Marker>(0), Some(&Marker));
        assert_eq!(col.get::<Marker>(2), Some(&Marker));

        col.swap_remove(1);
        assert_eq!(col.len(), 2);
        assert_eq!(col.get::<Marker>(0), Some(&Marker));
    }

    #[test]
    fn is_empty_on_new() {
        let col = ComponentColumn::new::<u64>();
        assert!(col.is_empty());
        assert_eq!(col.len(), 0);
    }

    #[test]
    fn many_pushes() {
        let mut col = ComponentColumn::new::<u64>();
        for i in 0..100u64 {
            col.push(i);
        }
        assert_eq!(col.len(), 100);
        for i in 0..100u64 {
            assert_eq!(col.get::<u64>(i as usize), Some(&i));
        }
    }

    #[test]
    #[should_panic(expected = "type mismatch")]
    fn type_mismatch_push_panics() {
        let mut col = ComponentColumn::new::<i32>();
        col.push(42u64);
    }

    #[test]
    #[should_panic(expected = "type mismatch")]
    fn type_mismatch_get_panics() {
        let mut col = ComponentColumn::new::<i32>();
        col.push(42i32);
        let _ = col.get::<u64>(0);
    }
}

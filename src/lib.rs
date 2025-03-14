//! # memflow-pointer-debug
//!
//! A crate that provides functionality for recursively printing data structures with
//! automatic pointer dereferencing, useful for debugging memory-mapped structures.
//!
//! This crate works with the `memflow` memory introspection framework to follow pointers
//! through memory and display their contents.

use memflow::mem::MemoryView;
use std::collections::HashSet;

/// Internal trait that handles the details of pointer dereferencing and printing.
///
/// This trait is marked as `#[doc(hidden)]` because it's not intended to be used directly.
/// Instead, use the [`PointerPrint`] trait which provides a more user-friendly interface
pub trait DerefDebugPrint {
    fn pointer_debug_internal<M: ::memflow::mem::MemoryView>(
        &self,
        mem: &mut M,
        depth: usize,
        max_depth: usize,
        visited_addresses: &mut ::std::collections::HashSet<u64>,
    );
}

/// High-level trait for printing data structures with automatic pointer dereferencing.
///
/// This trait is automatically implemented for any type that implements [`DerefDebugPrint`].
/// You can implement [`DerefDebugPrint`] manually, but it's recommended to use the
/// [`PointerDerefDebugPrint`] derive macro instead.
///
/// # Example
///
/// ```rust
/// use memflow_pointer_debug::{PointerPrint, PointerDerefDebugPrint};
///
/// #[derive(PointerDerefDebugPrint)]
/// struct MyStruct {
///     id: u32,
///     name: String,
///     next: Pointer64<MyStruct>,
/// }
///
/// fn example(mem: &mut impl MemoryView, my_struct: &MyStruct) {
///     // Simple usage with default depth of 5
///     my_struct.pointer_print(mem);
///
///     // With custom depth
///     my_struct.pointer_print_with_depth(mem, 10);
/// }
/// ```
pub trait PointerPrint {
    /// Print this structure with pointer dereferencing using the default max depth (5).
    ///
    /// This method will traverse the object graph, following pointers and printing
    /// their contents, up to a maximum depth of 5.
    ///
    /// # Parameters
    ///
    /// * `mem` - The memory view to read from
    fn pointer_print<M: MemoryView>(&self, mem: &mut M);

    /// Print this structure with pointer dereferencing using a custom max depth.
    ///
    /// This method will traverse the object graph, following pointers and printing
    /// their contents, up to the specified maximum depth.
    ///
    /// # Parameters
    ///
    /// * `mem` - The memory view to read from
    /// * `max_depth` - Maximum recursion depth
    fn pointer_print_with_depth<M: MemoryView>(&self, mem: &mut M, max_depth: usize);
}

/// Implement PointerPrint for any type that implements DerefDebugPrint
impl<T: DerefDebugPrint> PointerPrint for T {
    fn pointer_print<M: MemoryView>(&self, mem: &mut M) {
        // Use default max depth of 5
        self.pointer_print_with_depth(mem, 5);
    }

    fn pointer_print_with_depth<M: MemoryView>(&self, mem: &mut M, max_depth: usize) {
        // Create a new HashSet to track visited addresses
        let mut visited_addresses = HashSet::new();
        let mut is_pointer_deref = false;

        // Call the internal method with initial depth 0
        self.pointer_debug_internal(mem, 0, max_depth, &mut visited_addresses);
    }
}

/// Convenience function for printing any value that implements DerefDebugPrint.
///
/// This is equivalent to calling `value.pointer_print(mem)` but may be more
/// convenient in some contexts.
///
/// # Parameters
///
/// * `value` - The value to print, which must implement DerefDebugPrint
/// * `mem` - The memory view to read from
///
/// # Example
///
/// ```rust
/// use memflow_pointer_debug::{print_with_pointer_reading, PointerDerefDebugPrint};
///
/// #[derive(PointerDerefDebugPrint)]
/// struct MyStruct {
///     // fields...
/// }
///
/// fn example(mem: &mut impl MemoryView, my_struct: &MyStruct) {
///     print_with_pointer_reading(my_struct, mem);
/// }
/// ```

/// Re-export of the derive macro for implementing DerefDebugPrint.
///
/// This derive macro automatically implements the DerefDebugPrint trait for structs,
/// handling all the complexity of traversing fields, checking for pointers, and
/// preventing infinite recursion due to circular references.
///
/// # Example
///
/// ```rust
/// use memflow_pointer_debug::PointerDerefDebugPrint;
///
/// #[derive(Debug, PointerDerefDebugPrint)]
/// struct MyStruct {
///     id: u32,
///     name: String,
///     next_ptr: Pointer<MyStruct>,
/// }
/// ```
///
/// Fields containing "_pad" in their name will not be printed.
/// This crate is designed to be used with the `offsetter` crate
/// If you choose to manually pad just ensure your padding fields
/// contain `_pad``
pub fn print_with_pointer_reading<T: DerefDebugPrint, M: MemoryView>(value: &T, mem: &mut M) {
    value.pointer_print(mem);
}

pub use memflow_pointer_debug_derive::PointerDerefDebugPrint;

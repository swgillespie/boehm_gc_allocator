#![feature(allocator, no_std, libc)]
#![allocator]
#![crate_type = "rlib"]
#![no_std]

//! boehm_gc is an allocator crate that provides an interface to the Boehm conservative garbage
//! collector. The allocator that this crate provides ensures that all objects that it allocates
//! will root any GC'd pointers that the objects may contain. GC'd pointers are allocated using
//! the `gc_allocate` function and are freed automatically as they become unreachable.
//!
//! This crate can only be used with recent Rust nightlies due to the allocator feature being
//! brand new.

extern crate libc;

mod sys;

use core::mem;
use core::ptr;
use libc::{size_t, c_void};

/// This implementation of __rust_allocate invokes GC_malloc_uncollectable,
/// which allocates memory that is not collectable by the garbage collector
/// but is capable of rooting GC'd pointers. Any pointer that resides
/// in memory allocated by Rust's allocator will be traced for pointers,
/// and any pointers that are contained within this memory are considered
/// to be rooted.
#[no_mangle]
pub extern "C" fn __rust_allocate(size: usize, _: usize) -> *mut u8 {
    unsafe { sys::GC_malloc_uncollectable(size as size_t) as *mut u8 }
}

/// Deallocates memory allocated by GC_malloc_uncollectable. This memory isn't normally
/// collectable so we rely on Rust's drop glue to free the memory that it's allocated. Luckily,
/// it's really good at that sort of thing!
#[no_mangle]
pub extern "C" fn __rust_deallocate(ptr: *mut u8, _: usize, _: usize) {
    unsafe { sys::GC_free(ptr as *mut c_void) }
}

#[no_mangle]
pub extern "C" fn __rust_reallocate(ptr: *mut u8, _: usize, size: usize, _: usize) -> *mut u8 {
    unsafe { sys::GC_realloc(ptr as *mut c_void, size as size_t) as *mut u8 }
}

#[no_mangle]
pub extern "C" fn __rust_reallocate_inplace(ptr: *mut u8, _: usize, size: usize, _: usize) -> *mut u8 {
    unsafe { sys::GC_realloc(ptr as *mut c_void, size as size_t) as *mut u8 }
}

#[no_mangle]
pub extern "C" fn __rust_usable_size(size: usize, _: usize) -> usize {
    size
}

/// Allocates `size` bytes on the managed heap and returns a pointer to the newly-allocated
/// memory. This memory is tracked by the garbage collector and will be freed automatically
/// when it is no longer reachable.
#[inline]
pub fn gc_allocate(size: usize) -> *mut u8 {
    unsafe { sys::GC_malloc(size as size_t) as *mut u8 }
}

/// Forces the garbage collector to run, deallocating any unreachable memory. This is a full,
/// stop-the-world collection.
#[inline]
pub fn gc_collect() {
    unsafe { sys::GC_gcollect(); }
}

/// Used as an argument to register_finalizer to influence the circumstances upon which the garbage
/// collector will run finalizers.
#[derive(Clone, Copy, Debug)]
pub enum FinalizerMode {
    /// Performs the default behavior of the Boehm GC, which will not run a finalizer
    /// on an object that contains a pointer to itself.
    Standard,
    /// Performs the default behavior of the Boehm GC, but ignores any "self pointers". This
    /// mode will run finalizers on objects that contain pointers to themselves.
    IgnoreSelf,
    /// Performs the default behavior of the Boehm GC, but ignores all cycles when calculating
    /// which finalizers to run. This is a strict superset of the capabilities that IgnoreSelf
    /// provides.
    NoOrder
}

/// Attaches a finalizer function and metadata object to a garbage-collected pointer. The finalizer
/// function will be run right before an object is collected, after it has become unreachable.
/// The first argument to the finalizer function will be the object being collected (the `ptr` argument),
/// while the second argument to the finalizer function will be the metadata object passed as the
/// `data` argument.
///
/// The Boehm GC will attempt to build a graph of objects that need to be finalized (starting at ptr),
/// perform a topological sort, and then finalize the objects in the order that the topological sort
/// provides. This only works if the finalizer graph has no cycles. The Boehm GC offers a couple
/// of different approaches to dealing with cycles in the finalizer graph:
///
/// 1. The `Standard` finalizer mode will naively do a topological sort and find the order in which
///    things can be safely finalized. If there is a cycle in the graph, the Boehm GC will not finalize
///    that pointer. If an object contains a pointer to itself, it will not be finalized (as this
///    creates a cycle).
/// 2. The `IgnoreSelf` finalizer mode will behave similar to the Standard finalizer mode, but it will
///    explicitly ignore any pointers from an object to itself. Objects that contain a pointer to
///    themselves will be finalized in this mode.
/// 3. The `NoOrder` finalizer mode will throw all caution to the wind and finalize everything.
///    It is up to the programmer to ensure that finalization code doesn't accidentally access
///    finalized objects.
///
/// The Boehm GC provides mechanisms for breaking cycles in the finalizer chain but this
/// crate does not yet expose them.
#[inline]
pub fn register_finalizer(ptr: *mut u8,
                          finalizer: extern "C" fn(*mut u8, *mut u8),
                          data: *mut u8,
                          mode: FinalizerMode) {
    match mode {
        FinalizerMode::Standard => unsafe {
            sys::GC_register_finalizer(ptr as *mut c_void,
                mem::transmute(finalizer),
                data as *mut c_void,
                ptr::null_mut() as *mut _,
                ptr::null_mut() as *mut *mut _);
        },
        FinalizerMode::IgnoreSelf => unsafe {
            sys::GC_register_finalizer_ignore_self(ptr as *mut c_void,
                mem::transmute(finalizer),
                data as *mut c_void,
                ptr::null_mut() as *mut _,
                ptr::null_mut() as *mut *mut _);
        },
        FinalizerMode::NoOrder => unsafe {
            sys::GC_register_finalizer_no_order(ptr as *mut c_void,
                mem::transmute(finalizer),
                data as *mut c_void,
                ptr::null_mut() as *mut _,
                ptr::null_mut() as *mut *mut _);
        }
    }
}

/// Returns the number of bytes in the managed heap, including empty blocks
/// and fragmentation loss.
#[inline]
pub fn heap_size() -> usize {
    unsafe { sys::GC_get_heap_size() as usize }
}

/// Returns a lower bound on the number of free bytes in the heap.
#[inline]
pub fn free_bytes() -> usize {
    unsafe { sys::GC_get_free_bytes() as usize }
}

/// Returns the number of bytes allocated since the last GC.
#[inline]
pub fn bytes_since_gc() -> usize {
    unsafe { sys::GC_get_bytes_since_gc() as usize }
}

/// Returns the total number of bytes allocated in this process.
#[inline]
pub fn total_bytes() -> usize {
    unsafe { sys::GC_get_total_bytes() as usize }
}

/// Enables the garbage collector, if the number of times that gc_enable() has been
/// called is the same as the number of times that gc_disable() has been called.
#[inline]
pub fn gc_enable() {
    unsafe { sys::GC_enable() }
}

/// Disables the garbage collector and prevents gc_collect() from doing anything.
#[inline]
pub fn gc_disable() {
    unsafe { sys::GC_disable() }
}

/// Sets the function that the GC calls when all available memory is exhausted.
/// For now, this function must not return. The Boehm GC /does/ allow the function
/// to return, but it must return either null or a previously-allocated heap object.
///
/// It is imperative that the supplied oom_fn not allocate.
#[inline]
pub fn set_oom_fn(oom_fn: extern "C" fn(size_t) -> *mut u8) {
    unsafe { sys::GC_oom_fn = mem::transmute(oom_fn); }
}

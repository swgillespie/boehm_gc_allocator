use libc::{size_t, c_void};

#[link(name = "gc")]
extern {
    pub static mut GC_oom_fn : extern "C" fn(size_t) -> !;

    pub fn GC_malloc(nbytes: size_t) -> *mut c_void;
    pub fn GC_malloc_uncollectable(nbytes: size_t) -> *mut c_void;
    pub fn GC_realloc(old: *mut c_void, new_size: size_t) -> *mut c_void;
    pub fn GC_free(dead: *mut c_void);
    pub fn GC_gcollect();
    pub fn GC_register_finalizer_ignore_self(ptr: *mut c_void,
                                             finalizer: extern "C" fn(*mut c_void, *mut c_void),
                                             client_data: *mut c_void,
                                             old_finalizer: *mut extern "C" fn(*mut c_void, *mut c_void),
                                             old_client_data: *mut *mut c_void);
    pub fn GC_register_finalizer_no_order(ptr: *mut c_void,
                                 finalizer: extern "C" fn(*mut c_void, *mut c_void),
                                 client_data: *mut c_void,
                                 old_finalizer: *mut extern "C" fn(*mut c_void, *mut c_void),
                                 old_client_data: *mut *mut c_void);
    pub fn GC_register_finalizer(ptr: *mut c_void,
                                 finalizer: extern "C" fn(*mut c_void, *mut c_void),
                                 client_data: *mut c_void,
                                 old_finalizer: *mut extern "C" fn(*mut c_void, *mut c_void),
                                 old_client_data: *mut *mut c_void);
    pub fn GC_get_heap_size() -> size_t;
    pub fn GC_get_free_bytes() -> size_t;
    pub fn GC_get_bytes_since_gc() -> size_t;
    pub fn GC_get_total_bytes() -> size_t;
    pub fn GC_disable();
    pub fn GC_enable();
}

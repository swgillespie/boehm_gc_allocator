# boehm_gc for Rust

This crate is a prototype that provides an allocator to Rust that
has the ability to root garbage-collected pointers allocated by the Boehm GC.

The ability to swap out the Rust allocator is a very new feature that will require
a recently nightly to take advantage of. I've done a few tests on this crate
and it does work - pointers allocated by `boehm_gc::gc_allocate` are rooted
by anything that is allocated by the system allocator used by `std`.

This crate does require that you have `libgc` installed on your system.
It can usually be obtained through your package manager of choice
(I used homebrew on OSX just fine).

# FixedVec
A thread safe Vec-like structure that never implicitly reallocates (is of a fixed size).

## Purpose
I created `FixedVec` for [NECS](https://github.com/sandmuel/necs) (my game object management crate) so that objects
could be spawned while others get accessed. `FixedVec` functions similarly to the standard library's `Vec`, but never
implicitly reallocates (if an element can't fit, it is returned). This (along with the use of atomic operations for the
length) allows for a thread-safe and immutable `push`.
```rust
let vec = FixedVec::<&str>::new(2);
assert_eq!(Ok(()), vec.push("foo"));
assert_eq!(Some(&"foo"), vec.get(0));
assert_eq!(Ok(()), vec.push("bar"));
assert_eq!(Err("baz"), vec.push("baz"));
```

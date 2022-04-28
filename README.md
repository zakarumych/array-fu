# array-fu

[![crates](https://img.shields.io/crates/v/array-fu.svg?style=for-the-badge&label=array-fu)](https://crates.io/crates/array-fu)
[![docs](https://img.shields.io/badge/docs.rs-array--fu-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white)](https://docs.rs/array-fu)
[![actions](https://img.shields.io/github/workflow/status/zakarumych/array-fu/badge/master?style=for-the-badge)](https://github.com/zakarumych/array-fu/actions?query=workflow%3ARust)
[![MIT/Apache](https://img.shields.io/badge/license-MIT%2FApache-blue.svg?style=for-the-badge)](COPYING)
![loc](https://img.shields.io/tokei/lines/github/zakarumych/array-fu?style=for-the-badge)


`array-fu` crate defines `array` macro that can create arrays.
Use simple syntax, make it more complex as requirements change.

# Syntax

On the basic level, arrays construction happens by repeating provided expression multiple times.

```rust
# use array_fu::array;
let values = array!(1; 2);

assert_eq!(values, [1, 1]);
```

Unlike built-in syntax `[$expr; $size]` `array!` runs expression `$size` times instead of copying result.
This means that expression will exhibit its side effects for each array element,
and value can change freely.

```rust
# use array_fu::array;
# use rand::random;
let values: [f32; 2] = array!(random(); 2);
```

This also means that expression type may not be `Copy` or event `Clone`.

```rust
# use array_fu::array;
# use std::sync::Mutex;
let values = array!(Mutex::new(1); 2);
```

## Enumerate
 
`array!` macro supports enumerating while constructing array elements.

`array!($pat => $expr ; $n)` does the trick. That's it, simply add `$pat =>` before element expression.

`$pat` must be valid pattern. And it will be bound to numbers starting from 0.
Bound value can be used in the element expression.

```rust
# use array_fu::array;
let values = array!(x => x + 1; 3);

assert_eq!(values, [1, 2, 3]);
```

## Predicates

`array!` macro supports predicated that are evaluated before element expression for each constructed element.
When predicate does not pass, element expression is not executed.
Value bound to pattern will be updated before trying again.

```rust
# use array_fu::array;
let values = array!(x => x + 1; where x & 1 == 1; 3);

assert_eq!(values, [2, 4, 6]);
```

# Iterators

Creating arrays from iterators is really handy.
But it comes at price - there could be not enough values in the iterator to fill the array.

This crate provides `try_array!` macro that is unlike to `array!` returns `Option`.
`Some` array is returned if there were enough values.

```rust
# use array_fu::try_array;
let opt = try_array!(x in 1.. => x / 2; 3);

assert_eq!(opt, Some([0, 1, 1]));
```
`None` is returned otherwise.

```rust
# use array_fu::try_array;
let opt = try_array!(x in 1..3 => x / 2; 3);

assert_eq!(opt, None);
```

This macro also supports zipping multiple iterators.

```rust
# use array_fu::try_array;
let opt = try_array!(x in 1.., y in 2.. => x + y; 3);

assert_eq!(opt, Some([3, 5, 7]));
```

Surely it also supports predicates.
When predicate returns `false`, next items are taken from iterators.

```rust
# use array_fu::try_array;
let opt = try_array!(x in 1.., y in 2.. => x + y; where x * y > 10; 3);

assert_eq!(opt, Some([7, 9, 11]));
```

Patterns are typical Rust patterns, so they support destructuring.

```rust
# use array_fu::try_array;
let values = try_array!((x, y) in [(1, 2), (3, 4), (5, 6)] => x + y; 3);

assert_eq!(values, Some([3, 7, 11]));
```


Stay tuned, more macros are coming.



## License

Licensed under either of

* Apache License, Version 2.0, ([license/APACHE](license/APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([license/MIT](license/MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

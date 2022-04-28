# array-fu

[![crates](https://img.shields.io/crates/v/array-fu.svg?style=for-the-badge&label=array-fu)](https://crates.io/crates/array-fu)
[![docs](https://img.shields.io/badge/docs.rs-array--fu-66c2a5?style=for-the-badge&labelColor=555555&logoColor=white)](https://docs.rs/array-fu)
[![actions](https://img.shields.io/github/workflow/status/zakarumych/array-fu/badge/master?style=for-the-badge)](https://github.com/zakarumych/array-fu/actions?query=workflow%3ARust)
[![MIT/Apache](https://img.shields.io/badge/license-MIT%2FApache-blue.svg?style=for-the-badge)](COPYING)
![loc](https://img.shields.io/tokei/lines/github/zakarumych/array-fu?style=for-the-badge)



This crate defines [`array!`] and other macros that can construct arrays.
Use simple syntax, make it more complex as requirements change.

[`array!`] macro constructs arrays by repeating expression execution, possibly with enumeration bound to provided pattern.

# Examples

```
use array_fu::array;
use rand::random;

let values: [f32; 2] = array![random(); 2];
```

This also means that expression type may not be `Copy` or event `Clone`.

```
use array_fu::array;
# use std::sync::Mutex;
let values = array![Mutex::new(1); 2];
```

See more examples in the [`array!`] macro documentation.


[`collect_array!`] macro constructs arrays by repeating expression execution with elements from iterators bound to provided patterns.

# Examples

```
use array_fu::collect_array;

let opt = collect_array![x in 1.., y in 2.. => x + y; where x * y > 10; 3];

assert_eq!(opt, Some([7, 9, 11]));
```

```
use array_fu::collect_array;

let values = collect_array![(x, y) in [(1, 2), (3, 4), (5, 6)] => x + y; 3];

assert_eq!(values, Some([3, 7, 11]));
```

See more examples in the [`collect_array!`] macro documentation.


[`array!`]: https://docs.rs/array-fu/latest/array_fu/macro.array.html
[`collect_array!`]: https://docs.rs/array-fu/latest/array_fu/macro.collect_array.html

## License

Licensed under either of

* Apache License, Version 2.0, ([license/APACHE](license/APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([license/MIT](license/MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

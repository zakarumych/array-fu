//!
//! `array-fu` crate defines `array` macro that can create arrays.
//! Use simple syntax, make it more complex as requirements change.
//!
//! # Syntax
//!
//! On the basic level, arrays construction happens by repeating provided expression multiple times.
//!
//! ```rust
//! # use array_fu::array;
//! let values = array!(1; 2);
//!
//! assert_eq!(values, [1, 1]);
//! ```
//!
//! Unlike built-in syntax `[$expr; $size]` `array!` runs expression `$size` times instead of copying result.
//! This means that expression will exhibit its side effects for each array element,
//! and value can change freely.
//!
//! ```rust
//! # use array_fu::array;
//! # use rand::random;
//! let values: [f32; 2] = array!(random(); 2);
//! ```
//!
//! This also means that expression type may not be `Copy` or event `Clone`.
//!
//! ```rust
//! # use array_fu::array;
//! # use std::sync::Mutex;
//! let values = array!(Mutex::new(1); 2);
//! ```
//!
//! ## Enumerate
//!  
//! `array!` macro supports enumerating while constructing array elements.
//!
//! `array!($pat => $expr ; $n)` does the trick. That's it, simply add `$pat =>` before element expression.
//!
//! `$pat` must be valid pattern. And it will be bound to numbers starting from 0.
//! Bound value can be used in the element expression.
//!
//! ```rust
//! # use array_fu::array;
//! let values = array!(x => x + 1; 3);
//!
//! assert_eq!(values, [1, 2, 3]);
//! ```
//!
//! ## Predicates
//!
//! `array!` macro supports predicated that are evaluated before element expression for each constructed element.
//! When predicate does not pass, element expression is not executed.
//! Value bound to pattern will be updated before trying again.
//!
//! ```rust
//! # use array_fu::array;
//! let values = array!(x => x + 1; where x & 1 == 1; 3);
//!
//! assert_eq!(values, [2, 4, 6]);
//! ```
//!
//! # Iterators
//!
//! Creating arrays from iterators is really handy.
//! But it comes at price - there could be not enough values in the iterator to fill the array.
//!
//! This crate provides `try_array!` macro that is unlike to `array!` returns `Option`.
//! `Some` array is returned if there were enough values.
//!
//! ```rust
//! # use array_fu::try_array;
//! let opt = try_array!(x in 1.. => x / 2; 3);
//!
//! assert_eq!(opt, Some([0, 1, 1]));
//! ```
//! `None` is returned otherwise.
//!
//! ```rust
//! # use array_fu::try_array;
//! let opt = try_array!(x in 1..3 => x / 2; 3);
//!
//! assert_eq!(opt, None);
//! ```
//!
//! This macro also supports zipping multiple iterators.
//!
//! ```rust
//! # use array_fu::try_array;
//! let opt = try_array!(x in 1.., y in 2.. => x + y; 3);
//!
//! assert_eq!(opt, Some([3, 5, 7]));
//! ```
//!
//! Surely it also supports predicates.
//! When predicate returns `false`, next items are taken from iterators.
//!
//! ```rust
//! # use array_fu::try_array;
//! let opt = try_array!(x in 1.., y in 2.. => x + y; where x * y > 10; 3);
//!
//! assert_eq!(opt, Some([7, 9, 11]));
//! ```
//!
//! Patterns are typical Rust patterns, so they support destructuring.
//!
//! ```rust
//! # use array_fu::try_array;
//! let values = try_array!((x, y) in [(1, 2), (3, 4), (5, 6)] => x + y; 3);
//!
//! assert_eq!(values, Some([3, 7, 11]));
//! ```
//!
//!
//! Stay tuned, more macros are coming.
//!

#![no_std]

#[doc(hidden)]
#[macro_export]
macro_rules! pattern_list {
    ($ph:pat, $($pt:pat,)*) => {
        $crate::pattern_list!($($pt,)* | $ph )
    };
    ($ph:pat, $($pt:pat,)* | $r:pat) => {
        $crate::pattern_list!($($pt,)* | ($r, $ph) )
    };
    (| $r:pat) => {
        $r
    };
}

#[macro_export]
macro_rules! array {
    ($e:expr; $n:expr) => {{
        $crate::array!( _ => $e ; $n )
    }};
    ($p:pat => $e:expr $( ; where $( $cond:expr ),+ )? ; $n:expr) => {{
        #[allow(unused_mut)]
        let mut array: [::core::mem::MaybeUninit<_>; $n];

        array = unsafe { ::core::mem::MaybeUninit::uninit().assume_init() };
        let mut x = 0;

        for i in 0usize..$n {
            let _ = i;
            #[allow(unused_variables)]
            let elem;

            elem = loop {
                match x {
                    $p => {
                        #[allow(unused_assignments)]
                        {
                            x += 1;
                        }

                        #[allow(unreachable_code)]
                        {
                            $($(
                                #[allow(unused_variables)]
                                #[warn(unreachable_code)]
                                let cond = $cond;

                                if <bool as ::core::ops::Not>::not(cond) { continue; }
                            )+)?

                            #[allow(unused_variables)]
                            #[warn(unreachable_code)]
                            let elem = $e;

                            break elem;
                        }
                    }
                    #[allow(unreachable_patterns)]
                    _ => {}
                }
            };
            array[i].write(elem);
        }
        array.map(|elem| unsafe { elem.assume_init() })
    }};
}

#[macro_export]
macro_rules! try_array {
    ($e:expr; $ph:pat in $ih:expr $( , $pt:pat in $it:expr )* $(; where $($cond:expr),+ )? ; $n:expr) => {{
        let mut array: [::core::mem::MaybeUninit<_>; $n] = unsafe { ::core::mem::MaybeUninit::uninit().assume_init() };
        let iter = ::core::iter::IntoIterator::into_iter($ih);
        $( let iter = iter.zip($it); )*
        let mut iter = iter;

        let mut init_size = 0;
        for i in 0usize..$n
        {
            let elem;
            elem = loop {
                match iter.next() {
                    None => {
                        break None;
                    }
                    Some($crate::pattern_list!($ph, $( $pt, )*)) => {
                        #[allow(unreachable_code)]
                        {
                            $($(
                                #[allow(unused_variables)]
                                #[warn(unreachable_code)]
                                let cond = $cond;

                                if <bool as ::core::ops::Not>::not(cond) { continue; }
                            )+)?

                            #[allow(unused_variables)]
                            #[warn(unreachable_code)]
                            let elem = $e;

                            break Some(elem);
                        }
                    }
                    #[allow(unreachable_patterns)]
                    _ => continue,
                }
            };

            match elem {
                None => break,
                Some(elem) => {
                    array[i].write(elem);
                    init_size += 1;
                }
            }
        }

        if init_size == $n {
            Some(array.map(|elem| unsafe { elem.assume_init() }))
        } else {
            None
        }
    }};

    ($( $p:pat in $i:expr ),+ => $e:expr $(; where $($cond:expr),+ )? ; $n:expr) => {
        $crate::try_array!($e; $($p in $i),+ $( ; where $($cond),+ )? ; $n)
    };
}

#[test]
fn test_expression_repeat() {
    let mut i = 0;
    assert_eq!(array!({ i+=1; i }; 2), [1, 2]);
}

#[test]
fn test_comprehension_repeat() {
    assert_eq!(array!(x => x * 2; 3), [0, 2, 4]);
    assert_eq!(array!(x => x * 2; where x & 1 == 1; 3), [2, 6, 10]);
}

#[test]
fn test_comprehension_iter() {
    assert_eq!(
        try_array!(x * 2; x in 1..3; 3),
        None,
        "There's not enough elements in iterator"
    );
    assert_eq!(
        try_array!(x * 2; x in 1..; 3),
        Some([2, 4, 6]),
        "1*2, 2*2, 3*2"
    );
    assert_eq!(
        try_array!(x * y; x in 1.., y in (1..3).cycle(); where x > 3, y == 1; 3),
        Some([5, 7, 9]),
        "x = 1,2,3,4,5,6,7,8,9
         y = 1,2,1,2,1,2,1,2,1
         r = x,x,x,x,5,x,7,x,9"
    );

    assert_eq!(
        try_array!(x in 0.. => x * 2; where x & 1 == 1; 3),
        Some(array!(x => x * 2; where x & 1 == 1; 3)),
    );

    assert_eq!(
        try_array!(x in 0.., _y in 1.., _z in 2.., _w in 3..5 => x; where x & 1 == 1; 3),
        None,
    );
}

#[test]
fn test_bail() {
    array!(return; 2);
    panic!();
}

#[test]
fn test_bail_condition() {
    array!(_ => 0; where return; 1);
    panic!();
}

#[test]
fn test_bail_iter() {
    try_array!(_ in 1.. => 0; where return; 1);
    panic!();
}

#[test]
#[should_panic]
fn test_bail_panic() {
    array!(return; 0);
    panic!();
}

#[test]
#[should_panic]
fn test_bail_condition_panic() {
    array!(_ => 0; where return; 0);
    panic!();
}

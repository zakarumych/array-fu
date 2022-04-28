//!
//! `array-fu` crate defines `array` macro that can create arrays.
//! Use simple syntax, make it more complex as requirements change.
//!
//! `array!` macro constructs arrays by repeating expression execution, possibly with enumeration bound to provided pattern.
//!
//! # Examples
//!
//! ```
//! # use array_fu::array;
//! # use rand::random;
//! let values: [f32; 2] = array![random(); 2];
//! ```
//!
//! This also means that expression type may not be `Copy` or event `Clone`.
//!
//! ```
//! # use array_fu::array;
//! # use std::sync::Mutex;
//! let values = array![Mutex::new(1); 2];
//! ```
//!
//! See more examples in the [`array!`] macro documentation.
//!
//! `collect_array!` macro constructs arrays by repeating expression execution with elements from iterators bound to provided patterns.
//!
//! # Examples
//!
//! ```
//! # use array_fu::collect_array;
//! let opt = collect_array![x in 1.., y in 2.. => x + y; where x * y > 10; 3];
//!
//! assert_eq!(opt, Some([7, 9, 11]));
//! ```
//!
//! ```
//! # use array_fu::collect_array;
//! let values = collect_array![(x, y) in [(1, 2), (3, 4), (5, 6)] => x + y; 3];
//!
//! assert_eq!(values, Some([3, 7, 11]));
//! ```
//!
//! See more examples in the [`collect_array!`] macro documentation.
//!
#![no_std]

use core::{
    mem::{self, MaybeUninit},
    ptr,
};

#[doc(hidden)]
pub type Usize = usize;

#[doc(hidden)]
pub use core::{iter::IntoIterator, num::Wrapping, ops::Not};

#[doc(hidden)]
pub struct DontBreakFromElementExpressionWithoutLabel;

#[doc(hidden)]
pub fn type_name_of_val<T: ?Sized>(_val: &T) -> &'static str {
    ::core::any::type_name::<T>()
}

#[doc(hidden)]
pub struct PartiallyInitArray<T, const N: usize> {
    array: [MaybeUninit<T>; N],
    init: usize,
}

impl<T, const N: usize> PartiallyInitArray<T, N> {
    pub fn uninit() -> Self {
        PartiallyInitArray {
            // Could be written as `array![MaybeUninit::uninit(); N]`
            array: unsafe {
                // SAFETY: An uninitialized `[MaybeUninit<_>; N]` is valid.
                MaybeUninit::uninit().assume_init()
            },
            init: 0,
        }
    }

    /// # Safety
    ///
    /// Must be called at most `N` times.
    /// Or equivalently, until `is_init` returns false.
    #[inline]
    pub unsafe fn write(&mut self, value: T) {
        debug_assert!(self.init < N);
        self.array[self.init].write(value);
        self.init += 1;
    }

    #[inline]
    pub fn is_init(&self) -> bool {
        self.init == N
    }

    /// # Safety
    ///
    /// Must be called after `write` was called exactly `N` times.
    /// Or equivalently, when `is_init` returns true.
    #[inline]
    pub unsafe fn assume_init(self) -> [T; N] {
        debug_assert_eq!(self.init, N);
        let array = {
            // SAFETY: Fully initialized.
            mem::transmute_copy::<[MaybeUninit<T>; N], [T; N]>(&self.array)
        };
        mem::forget(self);
        array
    }

    #[inline]
    pub fn try_init(self) -> Option<[T; N]> {
        if self.init == N {
            let array = unsafe {
                // SAFETY: Fully initialized.
                mem::transmute_copy::<[MaybeUninit<T>; N], [T; N]>(&self.array)
            };
            mem::forget(self);
            Some(array)
        } else {
            None
        }
    }
}

impl<T, const N: usize> Drop for PartiallyInitArray<T, N> {
    fn drop(&mut self) {
        let slice = &mut self.array[..self.init];
        unsafe { ptr::drop_in_place(slice as *mut [MaybeUninit<T>] as *mut [T]) }
    }
}

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

/// Constructs arrays by repeating expression execution,
/// possibly with enumeration bound to provided pattern.
///
/// # Syntax
///
/// On the basic level, arrays construction happens by repeating execution of provided expression multiple times.
/// Note that the expression itself appears exactly once in expanded code.
/// And length expression is executed in const context exactly once.
///
/// ```
/// # use array_fu::array;
/// let values = array![1; 2];
///
/// assert_eq!(values, [1, 1]);
/// ```
///
/// Unlike built-in syntax `[$expr; $size]` `array!` runs expression `$size` times instead of copying result.
/// This means that expression will exhibit its side effects for each array element,
/// and value can change freely.
///
/// ```
/// # use array_fu::array;
/// # use rand::random;
/// let values: [f32; 2] = array![random(); 2];
/// ```
///
/// This also means that expression type may not be `Copy` or event `Clone`.
///
/// ```
/// # use array_fu::array;
/// # use std::sync::Mutex;
/// let values = array![Mutex::new(1); 2];
/// ```
///
/// ## Enumerate
///  
/// `array!` macro supports enumerating while constructing array elements.
///
/// `array!($pat => $expr ; $n)` does the trick. That's it, simply add `$pat =>` before element expression.
///
/// `$pat` must be valid pattern. And it will be bound to numbers starting from 0.
/// Bound value can be used in the element expression.
///
/// ```
/// # use array_fu::array;
/// let values = array![x => x + 1; 3];
///
/// assert_eq!(values, [1, 2, 3]);
/// ```
///
/// ## Predicates
///
/// `array!` macro supports predicated that are evaluated before element expression for each constructed element.
/// When predicate does not pass, element expression is not executed.
/// Value bound to pattern will be updated before trying again.
///
/// ```
/// # use array_fu::array;
/// let values = array![x => x + 1; where x & 1 == 1; 3];
///
/// assert_eq!(values, [2, 4, 6]);
/// ```
///
/// It is possible to make array expression infeasible.
/// For example by providing predicate that never evaluates to true.
///
/// ```should_panic
/// # use array_fu::array;
///
/// // predicate always evaluates to `false`
/// // making it impossible to construct array of size 1 or greater.
/// // This will lead to a panic with descriptive message.
/// // `[u8; 1]` type forces enumerator to be `u8` allowing it to fail faster.
/// let _: [u8; 1] = array![x => x; where false; 1];
/// ```
///
/// ## Control flow
///
/// Element expressions and conditions are executed in the inner loop scope but in the outer function.
/// This makes it possible to perform early return from macro invocation using `return` and `break` and `continue` statements.
/// `continue` and `break` won't compile without a label. If label is provided, they will behave as expected.
/// `return` would exit function where macro is called.
/// If size of the array is `0`, element and condition expressions won't be executed even once
/// and `return` statement won't exit the function.
/// This behavior is different from `[return; 0]` which performs early return regardless.
///
/// ```
/// # use array_fu::array;
/// array![return; 1];
/// ```
///
/// ```compile_fail
/// # use array_fu::array;
/// array![break; 1];
/// ```
///
/// ```compile_fail
/// # use array_fu::array;
/// array![continue; 1];
/// ```
///
/// ```
/// # use array_fu::array;
/// 'a: loop { array![break 'a; 1]; };
/// ```
///
/// ```
/// # use array_fu::array;
/// 'a: for _ in 0..3 { array![continue 'a; 1]; };
/// ```
///
/// ## List
///
/// For consistency with built-in syntax, arrays may be constructed with a list of expressions.
///
/// ```
/// # use array_fu::array;
/// let values = array![1, 2, 3];
///
/// assert_eq!(values, [1, 2, 3]);
/// ```
#[macro_export]
macro_rules! array {
    ($($e:expr),* $(,)?) => { [$($e,)*] };

    ($e:expr; $n:expr) => {{
        $crate::array!( _ => $e ; $n )
    }};

    ($p:pat => $e:expr $( ; where $( $cond:expr ),+ )? ; $n:expr) => {{
        #[allow(unused_mut)]
        let mut array = $crate::PartiallyInitArray::<_, $n>::uninit();

        let mut i = $crate::Wrapping(0);
        loop {
            let value = i.0;
            i += 1;

            if i.0 == 0 {
                panic!("Failed to initialize array using whole '{}' space", $crate::type_name_of_val(&i.0));
            }

            if array.is_init() {
                // This is the only way ouf of the loop without leaving outer scope.
                break;
            }

            match value {
                $p => {
                    #[allow(unreachable_code)]
                    {
                        $($(
                            #[allow(unused_variables)]
                            #[warn(unreachable_code)]
                            let cond = $cond;

                            if <bool as $crate::Not>::not(cond) { continue; }
                        )+)?

                        #[allow(unused_variables)]
                        let elem;

                        #[allow(unused_variables)]
                        let dont_continue_in_element_expression_without_label;

                        loop {
                            #[allow(unused)]
                            {
                                dont_continue_in_element_expression_without_label = ();
                            }

                            #[allow(unused_variables)]
                            #[warn(unreachable_code)]
                            let value = $e;

                            elem = value;

                            break $crate::DontBreakFromElementExpressionWithoutLabel;
                        };

                        unsafe {
                            array.write(elem);
                        }
                    }
                }
                #[allow(unreachable_patterns)]
                _ => continue,
            }
        }

        unsafe {
            // SAFETY: `is_init` returned true.
            array.assume_init()
        }
    }};
}

/// Constructs arrays by repeating expression
/// with elements from iterators bound to provided patterns.
///
/// Creating arrays from iterators is really handy.
/// But it comes at price - there could be not enough values in the iterator to fill the array.
///
/// Therefore this macro returns `Option`.
/// `Some` array is returned if there were enough values.
/// Otherwise `None` is returned.
///
/// ```
/// # use array_fu::collect_array;
/// let opt = collect_array![1..; 3];
///
/// assert_eq!(opt, Some([1, 2, 3]));
/// ```
///
/// `None` is returned otherwise.
///
/// ```
/// # use array_fu::collect_array;
/// let opt = collect_array![1..3; 3];
///
/// assert_eq!(opt, None, "There's only two elements in 1..3");
/// ```
///
/// Similarly to `array!` macro, `collect_array` can be given a pattern to bind iterator elements
/// and expression to produce array elements.
///
/// ```
/// # use array_fu::collect_array;
/// let opt = collect_array![x in 1.. => x / 2; 3];
///
/// assert_eq!(opt, Some([0, 1, 1]));
/// ```
///
/// But why stop there? Multiple iterators can be collected into an array!
///
/// ```
/// # use array_fu::collect_array;
/// let opt = collect_array![x in 1.., y in 2.. => x + y; 3];
///
/// assert_eq!(opt, Some([3, 5, 7]));
/// ```
///
/// Surely it also supports predicates.
/// When predicate evaluates to `false`, next items are taken from all iterators.
///
/// ```
/// # use array_fu::collect_array;
/// let opt = collect_array![x in 1.., y in 2.. => x + y; where x * y > 10; 3];
///
/// assert_eq!(opt, Some([7, 9, 11]));
/// ```
///
/// Patterns support destructuring.
///
/// ```
/// # use array_fu::collect_array;
/// let values = collect_array![(x, y) in [(1, 2), (3, 4), (5, 6)] => x + y; 3];
///
/// assert_eq!(values, Some([3, 7, 11]));
/// ```
///
/// And patterns don't have to be irrefutable.
///
/// ```
/// # use array_fu::collect_array;
/// let values = collect_array![(1, y) in [(1, 2), (3, 4), (1, 6)] => y; 2];
///
/// assert_eq!(values, Some([2, 6]));
/// ```
#[macro_export]
macro_rules! collect_array {
    ($it:expr; $n:expr) => {
        $crate::collect_array!(e in $it => e ; $n)
    };

    ($e:expr; $ph:pat in $ih:expr $( , $pt:pat in $it:expr )* $(; where $($cond:expr),+ )? ; $n:expr) => {{
        #[allow(unused_mut)]
        let mut array = $crate::PartiallyInitArray::<_, $n>::uninit();

        let iter = $crate::IntoIterator::into_iter($ih);
        $( let iter = iter.zip($it); )*
        let mut iter = iter;

        loop {
            if array.is_init() {
                break;
            }

            match iter.next() {
                None => break,
                Some($crate::pattern_list!($ph, $( $pt, )*)) => {
                    #[allow(unreachable_code)]
                    {
                        $($(
                            #[allow(unused_variables)]
                            #[warn(unreachable_code)]
                            let cond = $cond;

                            if <bool as $crate::Not>::not(cond) { continue; }
                        )+)?

                        #[allow(unused_variables)]
                        let elem;

                        #[allow(unused_variables)]
                        let dont_continue_in_element_expression_without_label;

                        loop {
                            #[allow(unused)]
                            {
                                dont_continue_in_element_expression_without_label = ();
                            }

                            #[allow(unused_variables)]
                            #[warn(unreachable_code)]
                            let value = $e;

                            elem = value;

                            break $crate::DontBreakFromElementExpressionWithoutLabel;
                        };

                        unsafe {
                            array.write(elem);
                        }
                    }
                }
                #[allow(unreachable_patterns)]
                _ => continue,
            }
        }

        array.try_init()
    }};

    ($( $p:pat in $i:expr ),+ => $e:expr $(; where $($cond:expr),+ )? ; $n:expr) => {
        $crate::collect_array!($e; $($p in $i),+ $( ; where $($cond),+ )? ; $n)
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
        collect_array!(x * 2; x in 1..3; 3),
        None,
        "There's not enough elements in iterator"
    );
    assert_eq!(
        collect_array!(x * 2; x in 1..; 3),
        Some([2, 4, 6]),
        "1*2, 2*2, 3*2"
    );
    assert_eq!(
        collect_array!(x * y; x in 1.., y in (1..3).cycle(); where x > 3, y == 1; 3),
        Some([5, 7, 9]),
        "x = 1,2,3,4,5,6,7,8,9
         y = 1,2,1,2,1,2,1,2,1
         r = x,x,x,x,5,x,7,x,9"
    );

    assert_eq!(
        collect_array!(x in 0.. => x * 2; where x & 1 == 1; 3),
        Some(array!(x => x * 2; where x & 1 == 1; 3)),
    );

    assert_eq!(
        collect_array!(x in 0.., _y in 1.., _z in 2.., _w in 3..5 => x; where x & 1 == 1; 3),
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
    collect_array!(_ in 1.. => 0; where return; 1);
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

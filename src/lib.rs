// -*- coding: utf-8 -*-
//
// Copyright 2022 Michael Büsch <m@bues.ch>
//
// Licensed under the Apache License version 2.0
// or the MIT license, at your option.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//

//! # SHR3: 3-shift register random number generator
//!
//! The SHR3 generator can be used to generate non-crypto random bits with only very few computations.
//!
//! It is suitable for running on very small and restricted hardware (e.g. small 8 bit microcontrollers).
//! The SHR3 function is evaluated once per extracted random bit. The LSB of the SHR3 state is extracted as output.
//!
//! The generator has a cycle of approximately `4_000_000_000` bits.
//! Do not use it to extract large amounts of random bits (more than a hundred MiB or so),
//! unless you can tolerate looping back to the beginning of the random stream.
//! It will loop back to the beginning after `2**32 - 1` iterations.
//!
//! This generator is *not* cryptographically secure! Do not use it for cryptographic applications.
//!
//! # Example:
//!
//! ```
//!     use shr3::prelude::*;
//!
//!     let mut shr3 = Shr3::new();                 // SHR3 with default seed (1).
//!     let x: u8 = shr3.get();                     // Extract 8 bits from shr3.
//!     let y: u16 = shr3.get_bits(10);             // Extract 10 bits from shr3 and store in lower bits of y.
//!     assert_eq!(x, 0xF8);                        // Extracted random value.
//!     assert_eq!(y, 0x2CC);                       // Extracted random value.
//!
//!     let mut shr3 = Shr3::new_state(42);         // SHR3 with custom seed (42).
//!
//!     let mut shr3: Shr3 = Default::default();    // Alternative to Shr::new().
//! ```
//!
//! # no_std
//!
//! This crate does not require the Rust std library. It does not link to std.
//!
//! # Optimized implementation
//!
//! This crate includes an optimized implementation for AVR 8-bit.
//!
//! All other architectures use the generic implementation.
//! On most architectures, this generic implementation will be compiled to rather efficient code.

#![no_std]
#![cfg_attr(target_arch="avr", feature(asm_experimental_arch))]

pub mod prelude {
    pub use crate::Shr3;
    pub use crate::Shr3Ops as _;
}

#[cfg(not(feature="__devmode__"))]
mod arch;
#[cfg(feature="__devmode__")]
pub mod arch;

use core::ops::{
    Add,
    BitOrAssign,
    Bound,
    RangeBounds,
    ShlAssign,
    Sub,
};
use core::num::Wrapping;

/// One round of the SHR3 shuffle function.
///
/// *Hint*: You probably want to use `Shr3Ops::get()`, `Shr3Ops::get_bits()`, 
///       `Shr3Ops::get_max()`, `Shr3Ops::get_minmax()` or `Shr3Ops::get_range()`
///       of struct `Shr3` instead.
///
/// SHR3 algorithm from sci.math post by George Marsaglia (Feb 25 2003, 10:25 am)
/// as part of the KISS generator:
///
/// `http://groups.google.com/group/sci.math/msg/9959175f66dd138f`
///
/// `http://groups.google.com/group/sci.math/msg/7e499231fb1e58d3`
///
/// The fixed variant with a full `2**32 - 1` cycle is implemented.
pub fn shr3(state: u32) -> u32 {
    #[cfg(target_arch="avr")]
    let state = arch::avr::shr3(state);

    #[cfg(not(target_arch="avr"))]
    let state = arch::generic::shr3(state);

    state
}

/// SHR3 generator register state.
pub struct Shr3 {
    state: u32,
}

impl Shr3 {
    /// Create a new SHR3 instance with default initial `state = 1`.
    #[inline]
    pub const fn new() -> Shr3 {
        Self::new_state(1)
    }

    /// Create a new SHR3 instance with user specified initial state.
    ///
    /// Special state 0: The SHR3 state must not be 0. If 0 is passed to this function,
    ///                  then the state 0x7FFFFFFF is picked instead.
    #[inline]
    pub const fn new_state(state: u32) -> Shr3 {
        Shr3 {
            state: if state == 0 { 0x7FFFFFFF } else { state },
        }
    }
}

impl Default for Shr3 {
    /// Create a new SHR3 instance with default initial `state = 1`.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Internal trait for basic operations on the output type.
pub trait BaseOps: Copy
{
    /// Unsigned companion of Self.
    type U;

    /// Number of bits in type `Self`.
    const NUMBITS: u8;
    /// Smallest possible value of type `Self`.
    const MINVAL: Self;
    /// Biggest possible value of type `Self`.
    const MAXVAL: Self;

    /// Convert an `u8` to `Self`.
    fn from_u8(v: u8) -> Self;

    /// Convert a wrapping unsigned to a bit identical Self.
    fn from_unsigned(v: Wrapping<Self::U>) -> Self;
    /// Convert self to a bit identical wrapping unsigned.
    fn to_unsigned(&self) -> Wrapping<Self::U>;

    /// Find last bit set in `self`.
    ///
    /// Bit 0 -> returns 1; Bit 1 -> returns 2; ...
    ///
    /// If no bit set -> returns 0.
    fn fls(&self) -> u8;
}

macro_rules! impl_base_ops {
    // $u = unsigned type.
    // $s = companion signed type.
    ($( ($u:ty, $s:ty) ),*) => {
        $(
            impl BaseOps for $u {
                type U = $u;
                const NUMBITS: u8 = <$u>::BITS as u8;
                const MINVAL: $u = <$u>::MIN;
                const MAXVAL: $u = <$u>::MAX;
                #[inline]
                fn from_u8(v: u8) -> $u {
                    v as $u
                }
                #[inline]
                fn from_unsigned(v: Wrapping<Self::U>) -> Self {
                    v.0
                }
                #[inline]
                fn to_unsigned(&self) -> Wrapping<Self::U> {
                    Wrapping(*self)
                }
                #[inline]
                fn fls(&self) -> u8 {
                    (<$u>::BITS - self.leading_zeros()) as u8
                }
            }
            impl BaseOps for $s {
                type U = $u;
                const NUMBITS: u8 = <$s>::BITS as u8;
                const MINVAL: $s = <$s>::MIN;
                const MAXVAL: $s = <$s>::MAX;
                #[inline]
                fn from_u8(v: u8) -> $s {
                    v as $s
                }
                #[inline]
                fn from_unsigned(v: Wrapping<Self::U>) -> Self {
                    v.0 as Self
                }
                #[inline]
                fn to_unsigned(&self) -> Wrapping<Self::U> {
                    Wrapping(*self as Self::U)
                }
                #[inline]
                fn fls(&self) -> u8 {
                    (<$s>::BITS - self.leading_zeros()) as u8
                }
            }
        )*
    }
}

impl_base_ops!((u8, i8), (u16, i16), (u32, i32), (u64, i64), (usize, isize));
#[cfg(has_u128)]
impl_base_ops!((u128, i128));

/// Main operations for extracting bits from SHR3 generator.
///
/// The type `T` can be either of
/// `u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`, `i64`, `u128`, `i128`, `usize` or `isize`.
pub trait Shr3Ops<T>:
    where T: BaseOps + Sub<Output=T> + PartialOrd,
          T::U: BaseOps,
          Wrapping<T::U>: Sub<Output=Wrapping<T::U>> + Add<Output=Wrapping<T::U>> + PartialOrd,
{
    /// Get a number of `bitcount` bits from SHR3 and store them in the lower
    /// bits of the returned type `T`.
    ///
    /// `bitcount` must be lower or equal to the number of bits in `T`.
    fn get_bits(&mut self, bitcount: u8) -> T;

    /// Get as many bits from SHR3 as fit into the return type `T`.
    ///
    /// *Note*: Consider using `get_bits()` instead, if you don't need all returned bits.
    #[inline]
    fn get(&mut self) -> T {
        self.get_bits(T::NUMBITS)
    }

    /// Get enough bits to construct a random value in the range between `min_value` and `max_value`.
    ///
    /// *Note*: If the extracted range is of non-power-of-two size,
    ///        then the number of bits extracted from the SHR3 generator will
    ///        be bigger to ensure an even distribution of the returned values.
    fn get_minmax(&mut self, min_value: T, max_value: T) -> T {
        debug_assert!(max_value >= min_value);
        let range = max_value.to_unsigned() - min_value.to_unsigned();
        let num_bits = range.0.fls();
        let value = loop {
            let value = self.get_bits(num_bits).to_unsigned();
            if value <= range {
                break value;
            }
        };
        T::from_unsigned(value + min_value.to_unsigned())
    }

    /// Get enough bits to construct a random value in the range between `0` and `max_value`.
    ///
    /// *Note*: If the extracted range is of non-power-of-two size,
    ///        then the number of bits extracted from the SHR3 generator will
    ///        be bigger to ensure an even distribution of the returned values.
    #[inline]
    fn get_max(&mut self, max_value: T) -> T {
        self.get_minmax(T::MINVAL, max_value)
    }

    /// Get enough bits to construct a random value in the given `range`.
    ///
    /// *Note*: If the extracted range is of non-power-of-two size,
    ///        then the number of bits extracted from the SHR3 generator will
    ///        be bigger to ensure an even distribution of the returned values.
    fn get_range(&mut self, range: impl RangeBounds<T>) -> T {
        let min = match range.start_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(_) | Bound::Unbounded => T::MINVAL,
        };
        let max = match range.end_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => {
                debug_assert!(*x > T::MINVAL);
                *x - T::from_u8(1) // to included
            },
            Bound::Unbounded => T::MAXVAL,
        };
        self.get_minmax(min, max)
    }
}

/// Shr3Ops for struct Shr3.
impl<T> Shr3Ops<T> for Shr3
    where T: BaseOps + Sub<Output=T> + PartialOrd,
          T::U: BaseOps,
          Wrapping<T::U>: Sub<Output=Wrapping<T::U>> + Add<Output=Wrapping<T::U>> + PartialOrd + ShlAssign<usize> + BitOrAssign,
{
    fn get_bits(&mut self, bitcount: u8) -> T {
        debug_assert!(bitcount <= T::NUMBITS);
        let mut ret = T::from_u8(0).to_unsigned();
        for _ in 0..bitcount {
            self.state = shr3(self.state);
            ret <<= 1;
            ret |= T::from_u8(self.state as u8 & 1).to_unsigned();
        }
        T::from_unsigned(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alg() {
        assert_eq!(shr3(0), 0);
        assert_eq!(shr3(0xFFFF_FFFF), 0x0003E01F);
        assert_eq!(shr3(0x5555_5555), 0x000EDFEA);
        assert_eq!(shr3(0xAAAA_AAAA), 0x000D3FF5);
        assert_eq!(shr3(0x4242_4242), 0x4B4AEFA7);
        assert_eq!(shr3(0x3C95_A60C), 0x82D826E6);
    }

    #[test]
    fn test_base_ops() {
        // unsigned
        assert_eq!(u8::NUMBITS, 8);
        assert_eq!(u16::NUMBITS, 16);
        assert_eq!(u32::NUMBITS, 32);
        assert_eq!(u64::NUMBITS, 64);
        assert_eq!(usize::NUMBITS, usize::BITS as u8);
        #[cfg(has_u128)]
        assert_eq!(u128::NUMBITS, 128);

        // signed
        assert_eq!(i8::NUMBITS, 8);
        assert_eq!(i16::NUMBITS, 16);
        assert_eq!(i32::NUMBITS, 32);
        assert_eq!(i64::NUMBITS, 64);
        assert_eq!(isize::NUMBITS, usize::BITS as u8);
        #[cfg(has_u128)]
        assert_eq!(i128::NUMBITS, 128);

        // unsigned
        assert_eq!(u8::MINVAL, 0);
        assert_eq!(u16::MINVAL, 0);
        assert_eq!(u32::MINVAL, 0);
        assert_eq!(u64::MINVAL, 0);
        assert_eq!(usize::MINVAL, 0);
        #[cfg(has_u128)]
        assert_eq!(u128::MINVAL, 0);

        // signed
        assert_eq!(i8::MINVAL, i8::MIN);
        assert_eq!(i16::MINVAL, i16::MIN);
        assert_eq!(i32::MINVAL, i32::MIN);
        assert_eq!(i64::MINVAL, i64::MIN);
        assert_eq!(isize::MINVAL, isize::MIN);
        #[cfg(has_u128)]
        assert_eq!(i128::MINVAL, i128::MIN);

        // unsigned
        assert_eq!(u8::MAXVAL, u8::MAX);
        assert_eq!(u16::MAXVAL, u16::MAX);
        assert_eq!(u32::MAXVAL, u32::MAX);
        assert_eq!(u64::MAXVAL, u64::MAX);
        assert_eq!(usize::MAXVAL, usize::MAX);
        #[cfg(has_u128)]
        assert_eq!(u128::MAXVAL, u128::MAX);

        // signed
        assert_eq!(i8::MAXVAL, i8::MAX);
        assert_eq!(i16::MAXVAL, i16::MAX);
        assert_eq!(i32::MAXVAL, i32::MAX);
        assert_eq!(i64::MAXVAL, i64::MAX);
        assert_eq!(isize::MAXVAL, isize::MAX);
        #[cfg(has_u128)]
        assert_eq!(i128::MAXVAL, i128::MAX);

        // unsigned
        assert_eq!(u8::from_u8(42), 42);
        assert_eq!(u16::from_u8(42), 42);
        assert_eq!(u32::from_u8(42), 42);
        assert_eq!(u64::from_u8(42), 42);
        assert_eq!(usize::from_u8(42), 42);
        #[cfg(has_u128)]
        assert_eq!(u128::from_u8(42), 42);

        // signed
        assert_eq!(i8::from_u8(42), 42);
        assert_eq!(i16::from_u8(42), 42);
        assert_eq!(i32::from_u8(42), 42);
        assert_eq!(i64::from_u8(42), 42);
        assert_eq!(isize::from_u8(42), 42);
        #[cfg(has_u128)]
        assert_eq!(i128::from_u8(42), 42);

        // signed
        assert_eq!(i8::from_u8(0xFF), -1);
        assert_eq!(i16::from_u8(0xFF), 0xFF);
        assert_eq!(i32::from_u8(0xFF), 0xFF);
        assert_eq!(i64::from_u8(0xFF), 0xFF);
        assert_eq!(isize::from_u8(0xFF), 0xFF);
        #[cfg(has_u128)]
        assert_eq!(i128::from_u8(0xFF), 0xFF);

        // from_unsigned (unsigned)
        assert_eq!(u8::from_unsigned(Wrapping(0xF0_u8)), 0xF0);
        assert_eq!(u16::from_unsigned(Wrapping(0xF0_u16)), 0xF0);
        assert_eq!(u32::from_unsigned(Wrapping(0xF0_u32)), 0xF0);
        assert_eq!(u64::from_unsigned(Wrapping(0xF0_u64)), 0xF0);
        assert_eq!(usize::from_unsigned(Wrapping(0xF0_usize)), 0xF0);
        #[cfg(has_u128)]
        assert_eq!(u128::from_unsigned(Wrapping(0xF0_u128)), 0xF0);

        // from_unsigned (signed)
        assert_eq!(i8::from_unsigned(Wrapping(0xF0_u8)), -16);
        assert_eq!(i16::from_unsigned(Wrapping(0xFFF0_u16)), -16);
        assert_eq!(i32::from_unsigned(Wrapping(0xFFFF_FFF0_u32)), -16);
        assert_eq!(i64::from_unsigned(Wrapping(0xFFFF_FFFF_FFFF_FFF0_u64)), -16);
        assert_eq!(isize::from_unsigned(Wrapping((-16_isize) as usize)), -16);
        #[cfg(has_u128)]
        assert_eq!(i128::from_unsigned(Wrapping((-16_i128) as u128)), -16);

        // to_unsigned (unsigned)
        assert_eq!(0xF0_u8.to_unsigned(), Wrapping(0xF0_u8));
        assert_eq!(0xF0_u16.to_unsigned(), Wrapping(0xF0_u16));
        assert_eq!(0xF0_u32.to_unsigned(), Wrapping(0xF0_u32));
        assert_eq!(0xF0_u64.to_unsigned(), Wrapping(0xF0_u64));
        assert_eq!(0xF0_usize.to_unsigned(), Wrapping(0xF0_usize));
        #[cfg(has_u128)]
        assert_eq!(0xF0_u128.to_unsigned(), Wrapping(0xF0_u128));

        // to_unsigned (signed)
        assert_eq!((-16_i8).to_unsigned(), Wrapping(0xF0_u8));
        assert_eq!((-16_i16).to_unsigned(), Wrapping(0xFFF0_u16));
        assert_eq!((-16_i32).to_unsigned(), Wrapping(0xFFFF_FFF0_u32));
        assert_eq!((-16_i64).to_unsigned(), Wrapping(0xFFFF_FFFF_FFFF_FFF0_u64));
        assert_eq!((-16_isize).to_unsigned(), Wrapping((-16_isize) as usize));
        #[cfg(has_u128)]
        assert_eq!((-16_i128).to_unsigned(), Wrapping((-16_i128) as u128));

        // unsigned fls
        assert_eq!(0x00_u8.fls(), 0);
        assert_eq!(0x80_u8.fls(), 8);
        assert_eq!(0x4F_u8.fls(), 7);
        assert_eq!(0x02_u8.fls(), 2);
        assert_eq!(0x01_u8.fls(), 1);
        assert_eq!(0x4000_u16.fls(), 15);
        assert_eq!(0x4000_0000_u32.fls(), 31);
        assert_eq!(0x4000_0000_0000_0000_u64.fls(), 63);
        #[cfg(has_u128)]
        assert_eq!(0x4000_0000_0000_0000_0000_0000_0000_0000_u128.fls(), 127);

        // signed fls
        assert_eq!(0x00_i8.fls(), 0);
        assert_eq!((-127_i8).fls(), 8);
        assert_eq!(0x4F_i8.fls(), 7);
        assert_eq!(0x02_i8.fls(), 2);
        assert_eq!(0x01_i8.fls(), 1);
        assert_eq!(0x4000_i16.fls(), 15);
        assert_eq!(0x4000_0000_i32.fls(), 31);
        assert_eq!(0x4000_0000_0000_0000_i64.fls(), 63);
        #[cfg(has_u128)]
        assert_eq!(0x4000_0000_0000_0000_0000_0000_0000_0000_i128.fls(), 127);
    }

    #[test]
    fn test_new() {
        let a: Shr3 = Default::default();
        assert_eq!(a.state, 1);
        assert_eq!(Shr3::new().state, 1);
        assert_eq!(Shr3::new_state(0).state, 0x7FFF_FFFF);
        assert_eq!(Shr3::new_state(1).state, 1);
        assert_eq!(Shr3::new_state(42).state, 42);
        assert_eq!(Shr3::new_state(0x7FFF_FFFF).state, 0x7FFF_FFFF);
        assert_eq!(Shr3::new_state(0xFFFF_FFFF).state, 0xFFFF_FFFF);
    }

    #[test]
    fn test_types() {
        // unsigned
        {
            let mut a = Shr3::new_state(42);
            let b: u8 = a.get_bits(5);
            assert_eq!(b, 4);
        }
        {
            let mut a = Shr3::new_state(42);
            let b: u16 = a.get_bits(5);
            assert_eq!(b, 4);
        }
        {
            let mut a = Shr3::new_state(42);
            let b: u32 = a.get_bits(5);
            assert_eq!(b, 4);
        }
        {
            let mut a = Shr3::new_state(42);
            let b: u64 = a.get_bits(5);
            assert_eq!(b, 4);
        }
        {
            let mut a = Shr3::new_state(42);
            let b: usize = a.get_bits(5);
            assert_eq!(b, 4);
        }
        #[cfg(has_u128)]
        {
            let mut a = Shr3::new_state(42);
            let b: u128 = a.get_bits(5);
            assert_eq!(b, 4);
        }

        // signed
        {
            let mut a = Shr3::new_state(42);
            let b: i8 = a.get_bits(5);
            assert_eq!(b, 4);
        }
        {
            let mut a = Shr3::new_state(42);
            let b: i16 = a.get_bits(5);
            assert_eq!(b, 4);
        }
        {
            let mut a = Shr3::new_state(42);
            let b: i32 = a.get_bits(5);
            assert_eq!(b, 4);
        }
        {
            let mut a = Shr3::new_state(42);
            let b: i64 = a.get_bits(5);
            assert_eq!(b, 4);
        }
        {
            let mut a = Shr3::new_state(42);
            let b: isize = a.get_bits(5);
            assert_eq!(b, 4);
        }
        #[cfg(has_u128)]
        {
            let mut a = Shr3::new_state(42);
            let b: i128 = a.get_bits(5);
            assert_eq!(b, 4);
        }
    }

    #[test]
    fn test_get_bits() {
        // unsigned
        let mut a = Shr3::new_state(42);
        for exp in [0x0001, 0x0000, 0x0001, 0x0005, 0x0001, 0x0004] {
            let b: u16 = a.get_bits(3);
            assert_eq!(b, exp);
        }
        let b: u16 = a.get_bits(0);
        assert_eq!(b, 0);

        // signed
        let mut a = Shr3::new_state(42);
        for exp in [0x0001, 0x0000, 0x0001, 0x0005, 0x0001, 0x0004] {
            let b: i16 = a.get_bits(3);
            assert_eq!(b, exp);
        }
        let b: i16 = a.get_bits(0);
        assert_eq!(b, 0);
    }

    #[test]
    fn test_get() {
        // unsigned
        let mut a = Shr3::new_state(42);
        for exp in [0x20D3, 0x2C5C, 0x2A17, 0xD3C5, 0xAF08, 0x9E5B] {
            let b: u16 = a.get();
            assert_eq!(b, exp);
        }

        // signed
        let mut a = Shr3::new_state(42);
        for exp in [0x20D3, 0x2C5C, 0x2A17, 0xD3C5, 0xAF08, 0x9E5B] {
            let b: i16 = a.get();
            assert_eq!(b as u16, exp as u16);
        }
    }

    #[test]
    fn test_max() {
        // unsigned
        let mut a = Shr3::new_state(42);
        for _ in 0..1000 {
            let b: u32 = a.get_max(100);
            assert!(b <= 100);
        }
        let b: u32 = a.get_max(0);
        assert_eq!(b, 0);

        // signed
        let mut a = Shr3::new_state(42);
        for _ in 0..1000 {
            let b: i32 = a.get_max(100);
            assert!(b <= 100);
        }
        let b: i32 = a.get_max(0);
        assert!(b <= 0);
    }

    #[test]
    fn test_minmax() {
        // unsigned
        let mut a = Shr3::new_state(42);
        for _ in 0..1000 {
            let b: u32 = a.get_minmax(60, 170);
            assert!((60..=170).contains(&b));
        }
        let b: u32 = a.get_minmax(111, 111);
        assert_eq!(b, 111);

        // signed
        let mut a = Shr3::new_state(42);
        for _ in 0..1000 {
            let b: i32 = a.get_minmax(-170, 60);
            assert!((-170..=60).contains(&b));
        }
        let b: i32 = a.get_minmax(-111, -111);
        assert_eq!(b, -111);
    }

    #[test]
    fn test_range() {
        let mut a = Shr3::new_state(42);

        // unsigned
        for _ in 0..1000 {
            let b: u32 = a.get_range(60..170);
            assert!((60..170).contains(&b));
        }
        for _ in 0..1000 {
            let b: u32 = a.get_range(60..=170);
            assert!((60..=170).contains(&b));
        }
        for _ in 0..1000 {
            let b: u32 = a.get_range(..170);
            assert!(b < 170);
        }
        for _ in 0..1000 {
            let b: u32 = a.get_range(..=170);
            assert!(b <= 170);
        }
        for _ in 0..1000 {
            let b: u32 = a.get_range(0xFFFF_FFF0..);
            assert!(b >= 0xFFFF_FFF0);
        }
        let b: u32 = a.get_range(111..112);
        assert_eq!(b, 111);
        let b: u32 = a.get_range(111..=111);
        assert_eq!(b, 111);

        // signed
        for _ in 0..1000 {
            let b: i32 = a.get_range(-60..170);
            assert!((-60..170).contains(&b));
        }
        for _ in 0..1000 {
            let b: i32 = a.get_range(-60..=170);
            assert!((-60..=170).contains(&b));
        }
        for _ in 0..1000 {
            let b: i32 = a.get_range(..170);
            assert!(b < 170);
        }
        for _ in 0..1000 {
            let b: i32 = a.get_range(..=170);
            assert!(b <= 170);
        }
        for _ in 0..1000 {
            let b: i32 = a.get_range(0x7FFF_FFF0..);
            assert!(b >= 0x7FFF_FFF0);
        }
        let b: i32 = a.get_range(-111..-110);
        assert_eq!(b, -111);
        let b: i32 = a.get_range(-111..=-111);
        assert_eq!(b, -111);
    }

/*
    #[test]
    fn test_cycle_unsigned() {
        let seed = 42;
        let mut a = Shr3::new_state(seed);
        let mut first = 0;
        let mut second = 0;
        for i in 0..=u32::MAX-2 {
            let _: u32 = a.get_bits(1);
            assert_ne!(a.state, seed);
            match i {
                0 => first = a.state,
                1 => second = a.state,
                _ => (),
            }
        }
        let _: u32 = a.get_bits(1);
        assert_eq!(a.state, seed);
        let _: u32 = a.get_bits(1);
        assert_eq!(a.state, first);
        let _: u32 = a.get_bits(1);
        assert_eq!(a.state, second);
    }
*/

/*
    #[test]
    fn test_cycle_signed() {
        let seed = 42;
        let mut a = Shr3::new_state(seed);
        let mut first = 0;
        let mut second = 0;
        for i in 0..=u32::MAX-2 {
            let _: i32 = a.get_bits(1);
            assert_ne!(a.state, seed);
            match i {
                0 => first = a.state,
                1 => second = a.state,
                _ => (),
            }
        }
        let _: i32 = a.get_bits(1);
        assert_eq!(a.state, seed);
        let _: i32 = a.get_bits(1);
        assert_eq!(a.state, first);
        let _: i32 = a.get_bits(1);
        assert_eq!(a.state, second);
    }
*/
}

// vim: ts=4 sw=4 expandtab

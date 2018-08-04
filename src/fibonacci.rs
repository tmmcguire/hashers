//! From https://probablydance.com/2018/06/16/fibonacci-hashing-the-optimization-that-the-world-forgot-or-a-better-alternative-to-integer-modulo/
//!
//! Possible modifier to separate hashes that are related.

use std::hash::Hasher;

/// This is a wrapper around another hasher that multiplies the
/// hash (with wrapping) by 2^64 / Φ (= 1.6180339...). It is intended
/// to spread closely-located but different hashes to different parts of
/// the table.
pub struct FibonacciWrapper<H: Hasher> {
    inner: H,
}

impl<H: Hasher> FibonacciWrapper<H> {
    /// Wrap an existing Hasher inner with the Fibonacci multiplier.
    pub fn wrap(inner: H) -> FibonacciWrapper<H> {
        FibonacciWrapper { inner: inner }
    }
}

impl<H: Hasher + Default> Default for FibonacciWrapper<H> {
    fn default() -> FibonacciWrapper<H> {
        FibonacciWrapper {
            inner: H::default(),
        }
    }
}

impl<H: Hasher> Hasher for FibonacciWrapper<H> {
    #[inline]
    fn finish(&self) -> u64 {
        self.inner.finish().wrapping_mul(11400714819323198485u64)
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.inner.write(bytes);
    }
}

macro_rules! fibonacci_mod {
    ($name:ident, $hash:path) => {
        /// Multiply the result of the unwrapped function by 2^64 / Φ.
        pub fn $name(slice: &[u8]) -> u64 {
            $hash(slice).wrapping_mul(11400714819323198485u64)
        }
    };
}

fibonacci_mod!(fibo_default, super::builtin::default);
fibonacci_mod!(fibo_null, super::null::null);
fibonacci_mod!(fibo_passthru, super::null::passthrough);
fibonacci_mod!(fibo_loselose, super::oz::loselose);
fibonacci_mod!(fibo_sdbm, super::oz::sdbm);
fibonacci_mod!(fibo_djb2, super::oz::djb2);
fibonacci_mod!(fibo_oaat, super::jenkins::oaat);
fibonacci_mod!(fibo_lookup3, super::jenkins::lookup3);

// ------------------------------------

#[cfg(test)]
mod fibonacci_tests {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(fibo_passthru(b""), 0);
        assert_eq!(fibo_passthru(b"a"), 17511437125486707701);
        assert_eq!(fibo_passthru(b"b"), 10465407871100354570);
        assert_eq!(fibo_passthru(b"ab"), 10834502084276483338);
        assert_eq!(fibo_passthru(b"abcd"), 2593013692314326836);
        assert_eq!(fibo_passthru(b"abcdefg"), 18445962681932401267);
    }
}

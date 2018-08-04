//! This collection of hash functions is based on:
//! - http://www.cse.yorku.ca/~oz/hash.html Oz's Hash functions. (oz)
//! - http://www.burtleburtle.net/bob/hash/doobs.html Bob Jenkins'
//!   (updated) 1997 Dr. Dobbs article. (jenkins)
//!
//! Each sub-module implements one or more Hashers plus a minimal testing module. As well, the
//! module has a benchmarking module for comparing the Hashers and some example programs using
//! statistical tests to prod the various Hashers.

#![feature(test)]

extern crate test;
extern crate fxhash;

// ====================================
// Utilities

/// Load an integer of the desired type from a byte stream, in LE order. Uses
/// `copy_nonoverlapping` to let the compiler generate the most efficient way
/// to load it from a possibly unaligned address.
///
/// Unsafe because: unchecked indexing at i..i+size_of(int_ty)
macro_rules! load_int_le {
    ($buf:expr, $i:expr, $int_ty:ident) => {{
        unsafe {
            debug_assert!($i + mem::size_of::<$int_ty>() <= $buf.len());
            let mut data = 0 as $int_ty;
            ptr::copy_nonoverlapping(
                $buf.get_unchecked($i),
                &mut data as *mut _ as *mut u8,
                mem::size_of::<$int_ty>(),
            );
            data.to_le()
        }
    }};
}

// This is how I might have done it.
// macro_rules! bytes_to {
//     ($slice:ident, $offset:expr, $dst_ty:ident) => {
//         unsafe {
//             *mem::transmute::<*const u8, &$dst_ty>(
//                 $slice
//                     .get_unchecked($offset..($offset + mem::size_of::<$dst_ty>()))
//                     .as_ptr(),
//             )
//         }
//     };
// }

// Create an implementation of Default for a simple type initialized
// with a constant value.
macro_rules! default_for_constant {
    ($(#[$attr:meta])* $name:ident, $default:expr) => {
        $(#[$attr])*
        impl Default for $name {
            #[inline]
            fn default() -> $name {
                $name($default)
            }
        }
    };
}

// Given a Hasher, create a single-use hash function.
macro_rules! hasher_to_fcn {
    ($(#[$attr:meta])* $name:ident, $hasher:ident) => {
        $(#[$attr])*
        #[inline]
        pub fn $name(bytes: &[u8]) -> u64 {
            let mut hasher = $hasher::default();
            hasher.write(bytes);
            hasher.finish()
        }
    };
}

// ====================================
// Hashing modules

pub mod oz;
pub mod jenkins;
pub mod fibonacci;

/// For easy access, reexport the built-in hash map's DefaultHasher,
/// including a matching one-stop function.
pub mod builtin {
    use std::hash::Hasher;

    pub use std::collections::hash_map::DefaultHasher;

    hasher_to_fcn!(
        /// Provide access to the DefaultHasher in a single function.
        default,
        DefaultHasher
    );
}

pub mod fx_hash {
    use std::hash::Hasher;
    pub use fxhash::{FxHasher,FxHasher32,FxHasher64};

    hasher_to_fcn!(fxhash, FxHasher);
    hasher_to_fcn!(fxhash32, FxHasher32);
    hasher_to_fcn!(fxhash64, FxHasher64);
}

/// Poor Hashers used for testing purposes.
///
/// These are not expected to be used. Really.
pub mod null {
    use std::hash::Hasher;

    /// Always returns 0.
    pub struct NullHasher;

    impl Hasher for NullHasher {
        #[inline]
        fn finish(&self) -> u64 {
            0u64
        }

        #[inline]
        fn write(&mut self, _bytes: &[u8]) {
            // data, you say?
        }
    }

    impl Default for NullHasher {
        fn default() -> NullHasher {
            NullHasher
        }
    }

    hasher_to_fcn!(
        /// Provide access to NullHasher in a single call.
        null,
        NullHasher
    );

    // --------------------------------

    /// Returns the last 4 bytes of the data, as a u64.
    pub struct PassThroughHasher(u64);

    impl Hasher for PassThroughHasher {
        #[inline]
        fn finish(&self) -> u64 {
            self.0
        }

        #[inline]
        fn write(&mut self, bytes: &[u8]) {
            for byte in bytes.iter() {
                self.0 = self.0.wrapping_shl(8) + (*byte as u64);
            }
        }
    }

    /// Provide a default PassThroughHasher initialized to 0.
    default_for_constant!(PassThroughHasher, 0);

    hasher_to_fcn!(
        /// Provide access to PassThroughHasher in a single call.
        passthrough,
        PassThroughHasher
    );
}

// ====================================
// FNV-1a (64-bit)

/// The [Fowler–Noll–Vo hash function](https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function).
pub mod fnv {
    use std::hash::Hasher;

    macro_rules! fnv1a {
        ($name:ident, $size:ty, $fnv_prime:expr, $offset_basis:expr) => {
            pub struct $name($size);
            impl Hasher for $name {
                #[inline]
                fn finish(&self) -> u64 {
                    self.0 as u64
                }
                #[inline]
                fn write(&mut self, bytes: &[u8]) {
                    for byte in bytes.iter() {
                        self.0 = self.0 ^ (*byte as $size);
                        self.0 = self.0.wrapping_mul($fnv_prime);
                    }
                }
            }
            default_for_constant!($name, $offset_basis);
        };
    }

    fnv1a!(FNV1aHasher32, u32, 16777619, 0x811c9dc5);
    fnv1a!(FNV1aHasher64, u64, 1099511628211, 0xcbf29ce484222325);

    hasher_to_fcn!(
        /// Provide access to FNV1aHasher32 in a single call.
        fnv1a32,
        FNV1aHasher32
    );

    hasher_to_fcn!(
        /// Provide access to FNV1aHasher64 in a single call.
        fnv1a64,
        FNV1aHasher64
    );

    #[cfg(test)]
    mod fnv1a_tests {
        use super::*;

        #[test]
        fn basic() {
            assert_eq!(fnv1a64(b""), 14695981039346656037);
            assert_eq!(fnv1a64(b"a"), 12638187200555641996);
            assert_eq!(fnv1a64(b"b"), 12638190499090526629);
            assert_eq!(fnv1a64(b"ab"), 620445648566982762);
            assert_eq!(fnv1a64(b"abcd"), 18165163011005162717);
            assert_eq!(fnv1a64(b"abcdefg"), 4642726675185563447);
        }
    }
}

// ====================================
// Benchmarks

#[cfg(test)]
mod benchmarks {
    use super::fnv::*;
    use super::jenkins::*;
    use super::null::*;
    use super::oz::*;
    use super::fx_hash::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;
    use test::{black_box, Bencher};

    macro_rules! tiny_bench {
        ($name:ident, $fcn:ident, $hasher:ident) => {
            hasher_to_fcn!($fcn, $hasher);
            #[bench]
            fn $name(b: &mut Bencher) {
                b.iter(|| black_box($fcn(b"abcd")))
            }
        };
    }

    tiny_bench!(tiny_default, defaulthasher, DefaultHasher);
    tiny_bench!(tiny_djb2, djb2, DJB2Hasher);
    tiny_bench!(tiny_sdbm, sdbm, SDBMHasher);
    tiny_bench!(tiny_loselose, loselose, LoseLoseHasher);
    tiny_bench!(tiny_oaat, oaat, OAATHasher);
    tiny_bench!(tiny_lookup3, lookup3, Lookup3Hasher);
    tiny_bench!(tiny_passthrough, passthrough, PassThroughHasher);
    tiny_bench!(tiny_fnv1a64, fnv1a64, FNV1aHasher64);
    tiny_bench!(tiny_fxhash, fxhash, FxHasher);
    tiny_bench!(tiny_fxhash32, fxhash32, FxHasher32);
    tiny_bench!(tiny_fxhash64, fxhash64, FxHasher64);

    macro_rules! w32_bench {
        ($name:ident, $hasher:ident, $count:expr) => {
            #[bench]
            fn $name(b: &mut Bencher) {
                b.iter(|| {
                    let mut h = $hasher::default();
                    for i in 0..$count {
                        h.write_i32(i);
                    }
                    black_box(h.finish())
                })
            }
        };
    }

    w32_bench!(w32_10_default, DefaultHasher, 10);
    w32_bench!(w32_10_djb2, DJB2Hasher, 10);
    w32_bench!(w32_10_sdbm, SDBMHasher, 10);
    w32_bench!(w32_10_loselose, LoseLoseHasher, 10);
    w32_bench!(w32_10_oaat, OAATHasher, 10);
    w32_bench!(w32_10_lookup3, Lookup3Hasher, 10);
    w32_bench!(w32_10_passthrough, PassThroughHasher, 10);
    w32_bench!(w32_10_fnv1a64, FNV1aHasher64, 10);
    w32_bench!(w32_10_fxhash, FxHasher, 10);

    w32_bench!(w32_100_default, DefaultHasher, 100);
    w32_bench!(w32_100_djb2, DJB2Hasher, 100);
    w32_bench!(w32_100_sdbm, SDBMHasher, 100);
    w32_bench!(w32_100_loselose, LoseLoseHasher, 100);
    w32_bench!(w32_100_oaat, OAATHasher, 100);
    w32_bench!(w32_100_lookup3, Lookup3Hasher, 100);
    w32_bench!(w32_100_passthrough, PassThroughHasher, 100);
    w32_bench!(w32_100_fnv1a64, FNV1aHasher64, 100);
    w32_bench!(w32_100_fxhash, FxHasher, 100);

    w32_bench!(w32_1000_default, DefaultHasher, 1000);
    w32_bench!(w32_1000_djb2, DJB2Hasher, 1000);
    w32_bench!(w32_1000_sdbm, SDBMHasher, 1000);
    w32_bench!(w32_1000_loselose, LoseLoseHasher, 1000);
    w32_bench!(w32_1000_oaat, OAATHasher, 1000);
    w32_bench!(w32_1000_lookup3, Lookup3Hasher, 1000);
    w32_bench!(w32_1000_passthrough, PassThroughHasher, 1000);
    w32_bench!(w32_1000_fnv1a64, FNV1aHasher64, 1000);
    w32_bench!(w32_1000_fxhash, FxHasher, 1000);

    macro_rules! w64_bench {
        ($name:ident, $hasher:ident, $count:expr) => {
            #[bench]
            fn $name(b: &mut Bencher) {
                b.iter(|| {
                    let mut h = $hasher::default();
                    for i in 0..$count {
                        h.write_i64(i);
                    }
                    black_box(h.finish())
                })
            }
        };
    }

    w64_bench!(w64_10_default, DefaultHasher, 10);
    w64_bench!(w64_10_djb2, DJB2Hasher, 10);
    w64_bench!(w64_10_sdbm, SDBMHasher, 10);
    w64_bench!(w64_10_loselose, LoseLoseHasher, 10);
    w64_bench!(w64_10_oaat, OAATHasher, 10);
    w64_bench!(w64_10_lookup3, Lookup3Hasher, 10);
    w64_bench!(w64_10_passthrough, PassThroughHasher, 10);
    w64_bench!(w64_10_fnv1a64, FNV1aHasher64, 10);
    w64_bench!(w64_10_fxhash, FxHasher, 10);

    w64_bench!(w64_100_default, DefaultHasher, 100);
    w64_bench!(w64_100_djb2, DJB2Hasher, 100);
    w64_bench!(w64_100_sdbm, SDBMHasher, 100);
    w64_bench!(w64_100_loselose, LoseLoseHasher, 100);
    w64_bench!(w64_100_oaat, OAATHasher, 100);
    w64_bench!(w64_100_lookup3, Lookup3Hasher, 100);
    w64_bench!(w64_100_passthrough, PassThroughHasher, 100);
    w64_bench!(w64_100_fnv1a64, FNV1aHasher64, 100);
    w64_bench!(w64_100_fxhash, FxHasher, 100);

    w64_bench!(w64_1000_default, DefaultHasher, 1000);
    w64_bench!(w64_1000_djb2, DJB2Hasher, 1000);
    w64_bench!(w64_1000_sdbm, SDBMHasher, 1000);
    w64_bench!(w64_1000_loselose, LoseLoseHasher, 1000);
    w64_bench!(w64_1000_oaat, OAATHasher, 1000);
    w64_bench!(w64_1000_lookup3, Lookup3Hasher, 1000);
    w64_bench!(w64_1000_passthrough, PassThroughHasher, 1000);
    w64_bench!(w64_1000_fnv1a64, FNV1aHasher64, 1000);
    w64_bench!(w64_1000_fxhash, FxHasher, 1000);

    fn read_words() -> Vec<String> {
        use std::fs::File;
        use std::io::prelude::*;
        use std::io::BufReader;

        let file = File::open("./data/words.txt").expect("cannot open words.txt");
        return BufReader::new(file)
            .lines()
            .map(|l| l.expect("bad read"))
            .collect();
    }

    macro_rules! words_bench {
        ($name:ident, $hasher:ident, $count:expr) => {
            #[bench]
            fn $name(b: &mut Bencher) {
                let words = read_words();
                b.iter(|| {
                    let mut h = $hasher::default();
                    for i in words.iter().take($count) {
                        h.write(i.as_bytes());
                    }
                    black_box(h.finish())
                })
            }
        };
    }

    words_bench!(words1000_default, DefaultHasher, 1000);
    words_bench!(words1000_djb2, DJB2Hasher, 1000);
    words_bench!(words1000_sdbm, SDBMHasher, 1000);
    words_bench!(words1000_loselose, LoseLoseHasher, 1000);
    words_bench!(words1000_oaat, OAATHasher, 1000);
    words_bench!(words1000_lookup3, Lookup3Hasher, 1000);
    words_bench!(words1000_passthrough, PassThroughHasher, 1000);
    words_bench!(words1000_fnv1a64, FNV1aHasher64, 1000);
    words_bench!(words1000_fxhash, FxHasher, 1000);

    macro_rules! file_bench {
        ($name:ident, $hasher:ident, $fcn:ident) => {
            hasher_to_fcn!($fcn, $hasher);
            #[bench]
            fn $name(b: &mut Bencher) {
                use std::fs::read;
                let file: Vec<u8> = read("./data/words.txt").expect("cannot read words.txt");
                b.iter(|| black_box($fcn(&file)))
            }
        };
    }

    file_bench!(file_default, DefaultHasher, defaultx);
    file_bench!(file_djb2, DJB2Hasher, djb2x);
    file_bench!(file_sdbm, SDBMHasher, sdbmx);
    file_bench!(file_loselose, LoseLoseHasher, loselosex);
    file_bench!(file_oaat, OAATHasher, oaatx);
    file_bench!(file_lookup3, Lookup3Hasher, lookup3x);
    file_bench!(file_passthrough, PassThroughHasher, passthroughx);
    file_bench!(file_fnv1a64, FNV1aHasher64, fnv1a64x);
    file_bench!(file_fnv1a32, FNV1aHasher32, fnv1a32x);
    file_bench!(file_fxhash, FxHasher, fxhashx);
}

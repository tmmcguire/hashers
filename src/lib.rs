//! This collection of hash functions is based on:
//! - http://www.cse.yorku.ca/~oz/hash.html Oz's Hash functions. (oz)
//! - http://www.burtleburtle.net/bob/hash/doobs.html Bob Jenkins'
//!   (updated) 1997 Dr. Dobbs article. (jenkins)
//!
//! Each sub-module implements one or more Hashers plus a minimal testing module. As well, the
//! module has a benchmarking module for comparing the Hashers and some example programs using
//! statistical tests to prod the various Hashers.

#![feature(test)]

extern crate byteorder;
extern crate test;

// ====================================
// Utilities

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

/// From http://www.cse.yorku.ca/~oz/hash.html.
///
/// > A comprehensive collection of hash functions, a hash visualiser
/// > and some test results [see Mckenzie et al. Selecting a Hashing
/// > Algorithm, SP&E 20(2):209-224, Feb 1990] will be available
/// > someday. If you just want to have a good hash function, and cannot
/// > wait, djb2 is one of the best string hash functions i know. it has
/// > excellent distribution and speed on many different sets of keys
/// > and table sizes. you are not likely to do better with one of the
/// > "well known" functions such as PJW, K&R[1], etc. Also see tpop
/// > pp. 126 for graphing hash functions.
///
/// "tpop" is *The Practice of Programming*. This page shows three
/// classic hashing algorithms.
pub mod oz {

    use std::hash::Hasher;
    use std::num::Wrapping;

    // ====================================
    // DJB2

    /// Dan Bernstein's famous hashing function.
    ///
    /// This Hasher is allegedly good for small tables with lowercase
    /// ASCII keys. It is also dirt-simple, although other hash
    /// functions are better and almost as simple.
    ///
    /// From http://www.cse.yorku.ca/~oz/hash.html:
    ///
    /// > this algorithm (k=33) was first reported by dan bernstein many
    /// > years ago in comp.lang.c. another version of this algorithm (now
    /// > favored by bernstein) uses xor: hash(i) = hash(i - 1) * 33 ^
    /// > str[i]; the magic of number 33 (why it works better than many
    /// > other constants, prime or not)
    /// > has never been adequately explained.
    ///
    /// From http://www.burtleburtle.net/bob/hash/doobs.html:
    ///
    /// > If your keys are lowercase English words, this will fit 6
    /// > characters into a 32-bit hash with no collisions (you'd
    /// > have to compare all 32 bits). If your keys are mixed case
    /// > English words, 65 * hash+key[i] fits 5 characters into a 32-bit
    /// > hash with no collisions. That means this type of hash can
    /// > produce (for the right type of keys) fewer collisions than
    /// > a hash that gives a more truly random distribution. If your
    /// > platform doesn't have fast multiplies, no sweat, 33 * hash =
    /// > hash+(hash<<5) and most compilers will figure that out for
    /// > you.
    /// >
    /// > On the down side, if you don't have short text keys, this hash
    /// > has a easily detectable flaws. For example, there's a 3-into-2
    /// > funnel that 0x0021 and 0x0100 both have the same hash (hex
    /// > 0x21, decimal 33) (you saw that one coming, yes?).
    pub struct DJB2Hasher(Wrapping<u64>);

    impl Hasher for DJB2Hasher {
        #[inline]
        fn finish(&self) -> u64 {
            (self.0).0
        }

        #[inline]
        fn write(&mut self, bytes: &[u8]) {
            for byte in bytes.iter() {
                self.0 = self.0 + (self.0 << 5) ^ Wrapping(*byte as u64);
            }
        }
    }

    default_for_constant!(DJB2Hasher, Wrapping(5381));
    hasher_to_fcn!(
        /// Provide access to DJB2Hasher in a single call.
        djb2,
        DJB2Hasher
    );

    // ------------------------------------

    #[cfg(test)]
    mod djb2_tests {
        use super::*;

        #[test]
        fn basic() {
            assert_eq!(djb2(b""), 5381);
            assert_eq!(djb2(b"a"), 177604);
            assert_eq!(djb2(b"b"), 177607);
            assert_eq!(djb2(b"ab"), 5860902);
        }
    }

    // ====================================
    // sdbm

    /// The hash function from SDBM (and gawk?). It might be good for
    /// something.
    ///
    /// From http://www.cse.yorku.ca/~oz/hash.html:
    ///
    /// > this algorithm was created for sdbm (a public-domain
    /// > reimplementation of ndbm) database library. it was found
    /// > to do well in scrambling bits, causing better distribution
    /// > of the keys and fewer splits. it also happens to be a good
    /// > general hashing function with good distribution. the actual
    /// > function is hash(i) = hash(i - 1) * 65599 + str[i]; what is
    /// > included below is the faster version used in gawk. [there is
    /// > even a faster, duff-device version] the magic constant 65599
    /// > was picked out of thin air while experimenting with different
    /// > constants, and turns out to be a prime. this is one of the
    /// > algorithms used in berkeley db (see sleepycat) and elsewhere.
    pub struct SDBMHasher(Wrapping<u64>);

    impl Hasher for SDBMHasher {
        #[inline]
        fn finish(&self) -> u64 {
            (self.0).0
        }

        #[inline]
        fn write(&mut self, bytes: &[u8]) {
            for byte in bytes.iter() {
                self.0 = Wrapping(*byte as u64) + (self.0 << 6) + (self.0 << 16) - self.0;
            }
        }
    }

    default_for_constant!(SDBMHasher, Wrapping(0));
    hasher_to_fcn!(
        /// Provide access to SDBMHasher in a single call.
        sdbm,
        SDBMHasher
    );

    // ------------------------------------

    #[cfg(test)]
    mod sdbm_tests {
        use super::*;

        #[test]
        fn basic() {
            assert_eq!(sdbm(b""), 0);
            assert_eq!(sdbm(b"a"), 97);
            assert_eq!(sdbm(b"b"), 98);
            assert_eq!(sdbm(b"ab"), 6363201);
        }
    }

    // ====================================
    // lose_lose

    /// A radically bad hash function from the 1st edition of the K&R C
    /// book. Do not use; it's horrible.
    ///
    /// From http://www.cse.yorku.ca/~oz/hash.html
    ///
    /// > This hash function appeared in K&R (1st ed) but at least the
    /// > reader was warned: "This is not the best possible algorithm,
    /// > but it has the merit of extreme simplicity." This is an
    /// > understatement; It is a terrible hashing algorithm, and it
    /// > could have been much better without sacrificing its "extreme
    /// > simplicity." [see the second edition!] Many C programmers
    /// > use this function without actually testing it, or checking
    /// > something like Knuth's Sorting and Searching, so it stuck. It
    /// > is now found mixed with otherwise respectable code, eg. cnews.
    /// > sigh. [see also: tpop]
    pub struct LoseLoseHasher(Wrapping<u64>);

    impl Hasher for LoseLoseHasher {
        #[inline]
        fn finish(&self) -> u64 {
            (self.0).0
        }

        #[inline]
        fn write(&mut self, bytes: &[u8]) {
            for byte in bytes.iter() {
                self.0 += Wrapping(*byte as u64);
            }
        }
    }

    default_for_constant!(LoseLoseHasher, Wrapping(0));
    hasher_to_fcn!(
        /// Provide access to LoseLoseHasher in a single call.
        loselose,
        LoseLoseHasher
    );

    // ------------------------------------

    #[cfg(test)]
    mod loselose_tests {
        use super::*;

        #[test]
        fn basic() {
            assert_eq!(loselose(b""), 0);
            assert_eq!(loselose(b"a"), 97);
            assert_eq!(loselose(b"b"), 98);
            assert_eq!(loselose(b"ab"), 195);
        }
    }
}

/// From http://www.burtleburtle.net/bob/hash/doobs.html.
///
/// This module mostly comes from his survey of hash functions. See also
/// https://en.wikipedia.org/wiki/Jenkins_hash_function.
pub mod jenkins {
    use std::hash::Hasher;
    use std::num::Wrapping;

    use byteorder::{ByteOrder, LittleEndian};

    // ================================
    // one_at_a_time

    /// The one-at-a-time Hasher. Relatively simple, but superseded by
    /// later algorithms.
    ///
    /// From http://www.burtleburtle.net/bob/hash/doobs.html:
    ///
    /// > This is similar to the rotating hash, but it actually mixes
    /// > the internal state. It takes 9n+9 instructions and produces a
    /// > full 4-byte result. Preliminary analysis suggests there are no
    /// > funnels.
    /// >
    /// > This hash was not in the original Dr. Dobb's article. I
    /// > implemented it to fill a set of requirements posed by Colin
    /// > Plumb. Colin ended up using an even simpler (and weaker) hash
    /// > that was sufficient for his purpose.
    pub struct OAATHasher(Wrapping<u64>);

    impl Hasher for OAATHasher {
        #[inline]
        fn finish(&self) -> u64 {
            let mut hash = self.0;
            hash += hash << 3;
            hash ^= hash >> 11;
            hash += hash << 15;
            hash.0
        }

        #[inline]
        fn write(&mut self, bytes: &[u8]) {
            for byte in bytes.iter() {
                self.0 += Wrapping(*byte as u64);
                self.0 += self.0 << 10;
                self.0 ^= self.0 >> 6;
            }
        }
    }

    default_for_constant!(OAATHasher, Wrapping(0));
    hasher_to_fcn!(
        /// Provide access to OAATHasher in a single call.
        oaat,
        OAATHasher
    );

    // ------------------------------------

    #[cfg(test)]
    mod oaat_tests {
        use super::*;

        #[test]
        fn basic() {
            assert_eq!(oaat(b""), 0);
            assert_eq!(oaat(b"a"), 29161854018);
            assert_eq!(oaat(b"b"), 30079156635);
            assert_eq!(oaat(b"ab"), 30087418617432);
            assert_eq!(oaat(b"abcdefg"), 3103867595652801641);
        }
    }

    // ================================
    // lookup3

    /// Another Hasher from the inventor of SpookyHash (which should be
    /// here, soon). Fancy bit-mixing. *Very fancy.*
    ///
    /// From http://www.burtleburtle.net/bob/hash/doobs.html:
    ///
    /// > ...http://burtleburtle.net/bob/c/lookup3.c (2006) is about 2
    /// > cycles/byte, works well on 32-bit platforms, and can produce a
    /// > 32 or 64 bit hash.
    ///
    /// > A hash I wrote nine years later designed along the same lines
    /// > as "My Hash", see http://burtleburtle.net/bob/c/lookup3.c.
    /// > It takes 2n instructions per byte for mixing instead of 3n.
    /// > When fitting bytes into registers (the other 3n instructions),
    /// > it takes advantage of alignment when it can (a trick learned
    /// > from Paul Hsieh's hash). It doesn't bother to reserve a byte
    /// > for the length. That allows zero-length strings to require no
    /// > mixing. More generally, the length that requires additional
    /// > mixes is now 13-25-37 instead of 12-24-36.
    /// >
    /// > One theoretical insight was that the last mix doesn't need to
    /// > do well in reverse (though it has to affect all output bits).
    /// > And the middle mixing steps don't have to affect all output
    /// > bits (affecting some 32 bits is enough), though it does have
    /// > to do well in reverse. So it uses different mixes for those
    /// > two cases. "My Hash" (lookup2.c) had a single mixing operation
    /// > that had to satisfy both sets of requirements, which is why it
    /// > was slower.
    /// >
    /// > On a Pentium 4 with gcc 3.4.?, Paul's hash was usually faster
    /// > than lookup3.c. On a Pentium 4 with gcc 3.2.?, they were about
    /// > the same speed. On a Pentium 4 with icc -O2, lookup3.c was a
    /// > little faster than Paul's hash. I don't know how it would play
    /// > out on other chips and other compilers. lookup3.c is slower
    /// > than the additive hash pretty much forever, but it's faster
    /// > than the rotating hash for keys longer than 5 bytes.
    /// >
    /// > lookup3.c does a much more thorough job of mixing than any of
    /// > my previous hashes (lookup2.c, lookup.c, One-at-a-time). All
    /// > my previous hashes did a more thorough job of mixing than Paul
    /// > Hsieh's hash. Paul's hash does a good enough job of mixing for
    /// > most practical purposes.
    /// >
    /// > The most evil set of keys I know of are sets of keys that are
    /// > all the same length, with all bytes zero, except with a few
    /// > bits set. This is tested by frog.c.. To be even more evil, I
    /// > had my hashes return b and c instead of just c, yielding a
    /// > 64-bit hash value. Both lookup.c and lookup2.c start seeing
    /// > collisions after 253 frog.c keypairs. Paul Hsieh's hash sees
    /// > collisions after 217 keypairs, even if we take two hashes
    /// > with different seeds. lookup3.c is the only one of the batch
    /// > that passes this test. It gets its first collision somewhere
    /// > beyond 263 keypairs, which is exactly what you'd expect from a
    /// > completely random mapping to 64-bit values.
    ///
    /// This structure implements hashlittle2:
    ///
    /// > You probably want to use hashlittle(). hashlittle() and
    /// > hashbig() hash byte arrays. hashlittle() is is faster than
    /// > hashbig() on little-endian machines. Intel and AMD are
    /// > little-endian machines. On second thought, you probably want
    /// > hashlittle2(), which is identical to hashlittle() except it
    /// > returns two 32-bit hashes for the price of one. You could
    /// > implement hashbig2() if you wanted but I haven't bothered
    /// > here.
    ///
    /// See http://www.burtleburtle.net/bob/c/lookup3.c.
    pub struct Lookup3Hasher {
        pc: Wrapping<u32>, // primary initval / primary hash
        pb: Wrapping<u32>, // secondary initval / secondary hash
    }

    impl Default for Lookup3Hasher {
        fn default() -> Lookup3Hasher {
            Lookup3Hasher {
                pc: Wrapping(0),
                pb: Wrapping(0),
            }
        }
    }

    #[inline]
    fn rot(x: Wrapping<u32>, k: usize) -> Wrapping<u32> {
        x << k | x >> (32 - k)
    }

    /// > mix -- mix 3 32-bit values reversibly.
    /// >
    /// > This is reversible, so any information in (a,b,c) before mix() is
    /// > still in (a,b,c) after mix().
    /// >
    /// > If four pairs of (a,b,c) inputs are run through mix(), or through
    /// > mix() in reverse, there are at least 32 bits of the output that
    /// > are sometimes the same for one pair and different for another pair.
    /// > This was tested for:
    /// > * pairs that differed by one bit, by two bits, in any combination
    /// >   of top bits of (a,b,c), or in any combination of bottom bits of
    /// >   (a,b,c).
    /// > * "differ" is defined as +, -, ^, or ~^.  For + and -, I transformed
    /// >   the output delta to a Gray code (a^(a>>1)) so a string of 1's (as
    /// >   is commonly produced by subtraction) look like a single 1-bit
    /// >   difference.
    /// > * the base values were pseudorandom, all zero but one bit set, or
    /// >   all zero plus a counter that starts at zero.
    /// >
    /// > Some k values for my "a-=c; a^=rot(c,k); c+=b;" arrangement that
    /// > satisfy this are
    /// >     4  6  8 16 19  4
    /// >     9 15  3 18 27 15
    /// >    14  9  3  7 17  3
    /// > Well, "9 15 3 18 27 15" didn't quite get 32 bits diffing
    /// > for "differ" defined as + with a one-bit base and a two-bit delta.  I
    /// > used http://burtleburtle.net/bob/hash/avalanche.html to choose
    /// > the operations, constants, and arrangements of the variables.
    /// >
    /// > This does not achieve avalanche.  There are input bits of (a,b,c)
    /// > that fail to affect some output bits of (a,b,c), especially of a.  The
    /// > most thoroughly mixed value is c, but it doesn't really even achieve
    /// > avalanche in c.
    /// >
    /// > This allows some parallelism.  Read-after-writes are good at doubling
    /// > the number of bits affected, so the goal of mixing pulls in the opposite
    /// > direction as the goal of parallelism.  I did what I could.  Rotates
    /// > seem to cost as much as shifts on every machine I could lay my hands
    /// > on, and rotates are much kinder to the top and bottom bits, so I used
    /// > rotates.
    #[inline]
    fn mix(a: &mut Wrapping<u32>, b: &mut Wrapping<u32>, c: &mut Wrapping<u32>) {
        *a -= *c;
        *a ^= rot(*c, 4);
        *c += *b;
        *b -= *a;
        *b ^= rot(*a, 6);
        *a += *c;
        *c -= *b;
        *c ^= rot(*b, 8);
        *b += *a;
        *a -= *c;
        *a ^= rot(*c, 16);
        *c += *b;
        *b -= *a;
        *b ^= rot(*a, 19);
        *a += *c;
        *c -= *b;
        *c ^= rot(*b, 4);
        *b += *a;
    }

    /// > final -- final mixing of 3 32-bit values (a,b,c) into c
    /// >
    /// > Pairs of (a,b,c) values differing in only a few bits will usually
    /// > produce values of c that look totally different.  This was tested for
    /// > - pairs that differed by one bit, by two bits, in any combination
    /// >   of top bits of (a,b,c), or in any combination of bottom bits of
    /// >   (a,b,c).
    /// > - "differ" is defined as +, -, ^, or ~^.  For + and -, I transformed
    /// >   the output delta to a Gray code (a^(a>>1)) so a string of 1's (as
    /// >   is commonly produced by subtraction) look like a single 1-bit
    /// >   difference.
    /// > - the base values were pseudorandom, all zero but one bit set, or
    /// >   all zero plus a counter that starts at zero.
    /// >
    /// > These constants passed:
    /// >
    /// >  14 11 25 16 4 14 24
    /// >  12 14 25 16 4 14 24
    /// >
    /// > and these came close:
    /// >
    /// >  4  8 15 26 3 22 24
    /// >  10  8 15 26 3 22 24
    /// >  11  8 15 26 3 22 24
    #[inline]
    fn final_mix(a: &mut Wrapping<u32>, b: &mut Wrapping<u32>, c: &mut Wrapping<u32>) {
        *c ^= *b;
        *c -= rot(*b, 14);
        *a ^= *c;
        *a -= rot(*c, 11);
        *b ^= *a;
        *b -= rot(*a, 25);
        *c ^= *b;
        *c -= rot(*b, 16);
        *a ^= *c;
        *a -= rot(*c, 4);
        *b ^= *a;
        *b -= rot(*a, 14);
        *c ^= *b;
        *c -= rot(*b, 24);
    }

    /// Turn 0-4 bytes into an unsigned 32-bit number.
    #[inline]
    fn shift_add(s: &[u8]) -> Wrapping<u32> {
        Wrapping(match s.len() {
            1 => s[0] as u32,
            2 => (s[0] as u32) + ((s[1] as u32) << 8),
            3 => (s[0] as u32) + ((s[1] as u32) << 8) + ((s[2] as u32) << 16),
            4 => {
                (s[0] as u32) + ((s[1] as u32) << 8) + ((s[2] as u32) << 16) + ((s[3] as u32) << 24)
            }
            _ => 0 as u32,
        })
    }

    #[inline]
    fn offset_to_align<T>(ptr: *const T, align: usize) -> usize {
        align - (ptr as usize & (align - 1))
    }

    /// Load an integer of the desired type from a byte stream, in LE order. Uses
    /// `copy_nonoverlapping` to let the compiler generate the most efficient way
    /// to load it from a possibly unaligned address.
    ///
    /// Unsafe because: unchecked indexing at i..i+size_of(int_ty)
    macro_rules! load_int_le {
        ($buf:expr, $i:expr, $int_ty:ident) => {{
            debug_assert!($i + mem::size_of::<$int_ty>() <= $buf.len());
            let mut data = 0 as $int_ty;
            ptr::copy_nonoverlapping(
                $buf.get_unchecked($i),
                &mut data as *mut _ as *mut u8,
                mem::size_of::<$int_ty>(),
            );
            data.to_le()
        }};
    }

    impl Hasher for Lookup3Hasher {
        #[inline]
        fn finish(&self) -> u64 {
            (self.pc.0 as u64) + ((self.pb.0 as u64) << 32)
        }

        #[inline]
        fn write(&mut self, bytes: &[u8]) {
            if bytes.len() == 0 {
                return;
            }
            let initial = Wrapping(0xdeadbeefu32) + Wrapping(bytes.len() as u32) + self.pc;
            let mut a: Wrapping<u32> = initial;
            let mut b: Wrapping<u32> = initial;
            let mut c: Wrapping<u32> = initial;
            c += self.pb;

            if cfg!(target_endian = "little") && offset_to_align(bytes.as_ptr(), 4) == 0 {
                // TODO: Use exact_chunks?
                for chunk in bytes.chunks(12) {
                    if chunk.len() == 12 {
                        let mut words: [u32; 3] = [0; 3]; // 3 * (4 bytes) = 12
                        LittleEndian::read_u32_into(chunk, &mut words);
                        a += Wrapping(words[0]);
                        b += Wrapping(words[1]);
                        c += Wrapping(words[2]);
                        mix(&mut a, &mut b, &mut c);
                    } else if chunk.len() >= 8 {
                        let (w, bs) = chunk.split_at(8);
                        let mut words: [u32; 2] = [0; 2]; // 2 * (4 bytes) = 12
                        LittleEndian::read_u32_into(w, &mut words);
                        a += Wrapping(words[0]);
                        b += Wrapping(words[1]);
                        c += shift_add(bs);
                    } else if chunk.len() >= 4 {
                        let (w, bs) = chunk.split_at(4);
                        let mut words: [u32; 1] = [0; 1]; // 1 * (4 bytes) = 12
                        LittleEndian::read_u32_into(w, &mut words);
                        a += Wrapping(words[0]);
                        b += shift_add(bs);
                    } else {
                        a += shift_add(chunk);
                    }
                }
            } else if cfg!(target_endian = "little") && offset_to_align(bytes.as_ptr(), 2) == 0 {
                for chunk in bytes.chunks(12) {
                    if chunk.len() == 12 {
                        let mut words: [u16; 6] = [0; 6]; // 6 * (2 bytes) = 12
                        LittleEndian::read_u16_into(chunk, &mut words);
                        a += Wrapping(words[0] as u32 + (words[1] as u32) << 16);
                        b += Wrapping(words[2] as u32 + (words[3] as u32) << 16);
                        c += Wrapping(words[4] as u32 + (words[3] as u32) << 16);
                        mix(&mut a, &mut b, &mut c);
                    } else if chunk.len() >= 8 {
                        let (w, bs) = chunk.split_at(8);
                        let mut words: [u16; 4] = [0; 4]; // 4 * (2 bytes) = 8
                        LittleEndian::read_u16_into(w, &mut words);
                        a += Wrapping(words[0] as u32 + (words[1] as u32) << 16);
                        b += Wrapping(words[2] as u32 + (words[3] as u32) << 16);
                        c += shift_add(bs);
                    } else if chunk.len() >= 4 {
                        let (w, bs) = chunk.split_at(4);
                        let mut words: [u16; 2] = [0; 2]; // 2 * (2 bytes) = 4
                        LittleEndian::read_u16_into(w, &mut words);
                        a += Wrapping(words[0] as u32 + (words[1] as u32) << 16);
                        b += shift_add(bs);
                    } else {
                        a += shift_add(chunk);
                    }
                }
            } else {
                // For big endian machines and unaligned slices: hash bytes.
                // "You could implement hashbig2() if you wanted but I
                // haven't bothered here."
                for chunk in bytes.chunks(12) {
                    if chunk.len() == 12 {
                        let (hunk, rest) = chunk.split_at(4);
                        a += shift_add(hunk);
                        let (hunk, rest) = rest.split_at(4);
                        b += shift_add(hunk);
                        let (hunk, _) = rest.split_at(4);
                        c += shift_add(hunk);
                        mix(&mut a, &mut b, &mut c);
                    } else if chunk.len() >= 8 {
                        let (hunk, rest) = chunk.split_at(4);
                        a += shift_add(hunk);
                        let (hunk, rest) = rest.split_at(4);
                        b += shift_add(hunk);
                        c += shift_add(rest);
                    } else if chunk.len() >= 4 {
                        let (hunk, rest) = chunk.split_at(4);
                        a += shift_add(hunk);
                        b += shift_add(rest);
                    } else {
                        a += shift_add(chunk);
                    }
                }
            }
            final_mix(&mut a, &mut b, &mut c);
            self.pb = b;
            self.pc = c;
        }
    }

    hasher_to_fcn!(
        /// Provide access to Lookup3Hasher in a single call.
        lookup3,
        Lookup3Hasher
    );

    // ------------------------------------

    #[cfg(test)]
    mod lookup3_tests {
        use super::*;

        #[test]
        fn basic() {
            assert_eq!(lookup3(b""), 0);
            assert_eq!(lookup3(b"a"), 6351843130003064584);
            assert_eq!(lookup3(b"b"), 5351957087540069269);
            assert_eq!(lookup3(b"ab"), 7744397999705663711);
            assert_eq!(lookup3(b"abcd"), 16288908501016938652);
            assert_eq!(lookup3(b"abcdefg"), 6461572128488215717);
        }
    }
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
// Fibonacci hash modifier

/// From https://probablydance.com/2018/06/16/fibonacci-hashing-the-optimization-that-the-world-forgot-or-a-better-alternative-to-integer-modulo/
///
/// Possible modifier to separate hashes that are related.
pub mod fibonacci {
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
}

// ====================================
// Benchmarks

#[cfg(test)]
mod benchmarks {
    use super::fnv::*;
    use super::jenkins::*;
    use super::null::*;
    use super::oz::*;
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

    tiny_bench!(tiny_default, defaulthasher, DefaultHasher);
    tiny_bench!(tiny_djb2, djb2, DJB2Hasher);
    tiny_bench!(tiny_sdbm, sdbm, SDBMHasher);
    tiny_bench!(tiny_loselose, loselose, LoseLoseHasher);
    tiny_bench!(tiny_oaat, oaat, OAATHasher);
    tiny_bench!(tiny_lookup3, lookup3, Lookup3Hasher);
    tiny_bench!(tiny_passthrough, passthrough, PassThroughHasher);
    tiny_bench!(tiny_fnv1a64, fnv1a64, FNV1aHasher64);

    w32_bench!(w32_10_default, DefaultHasher, 10);
    w32_bench!(w32_10_djb2, DJB2Hasher, 10);
    w32_bench!(w32_10_sdbm, SDBMHasher, 10);
    w32_bench!(w32_10_loselose, LoseLoseHasher, 10);
    w32_bench!(w32_10_oaat, OAATHasher, 10);
    w32_bench!(w32_10_lookup3, Lookup3Hasher, 10);
    w32_bench!(w32_10_passthrough, PassThroughHasher, 10);
    w32_bench!(w32_10_fnv1a64, FNV1aHasher64, 10);

    w32_bench!(w32_100_default, DefaultHasher, 100);
    w32_bench!(w32_100_djb2, DJB2Hasher, 100);
    w32_bench!(w32_100_sdbm, SDBMHasher, 100);
    w32_bench!(w32_100_loselose, LoseLoseHasher, 100);
    w32_bench!(w32_100_oaat, OAATHasher, 100);
    w32_bench!(w32_100_lookup3, Lookup3Hasher, 100);
    w32_bench!(w32_100_passthrough, PassThroughHasher, 100);
    w32_bench!(w32_100_fnv1a64, FNV1aHasher64, 100);

    w32_bench!(w32_1000_default, DefaultHasher, 1000);
    w32_bench!(w32_1000_djb2, DJB2Hasher, 1000);
    w32_bench!(w32_1000_sdbm, SDBMHasher, 1000);
    w32_bench!(w32_1000_loselose, LoseLoseHasher, 1000);
    w32_bench!(w32_1000_oaat, OAATHasher, 1000);
    w32_bench!(w32_1000_lookup3, Lookup3Hasher, 1000);
    w32_bench!(w32_1000_passthrough, PassThroughHasher, 1000);
    w32_bench!(w32_1000_fnv1a64, FNV1aHasher64, 1000);

    w64_bench!(w64_10_default, DefaultHasher, 10);
    w64_bench!(w64_10_djb2, DJB2Hasher, 10);
    w64_bench!(w64_10_sdbm, SDBMHasher, 10);
    w64_bench!(w64_10_loselose, LoseLoseHasher, 10);
    w64_bench!(w64_10_oaat, OAATHasher, 10);
    w64_bench!(w64_10_lookup3, Lookup3Hasher, 10);
    w64_bench!(w64_10_passthrough, PassThroughHasher, 10);
    w64_bench!(w64_10_fnv1a64, FNV1aHasher64, 10);

    w64_bench!(w64_100_default, DefaultHasher, 100);
    w64_bench!(w64_100_djb2, DJB2Hasher, 100);
    w64_bench!(w64_100_sdbm, SDBMHasher, 100);
    w64_bench!(w64_100_loselose, LoseLoseHasher, 100);
    w64_bench!(w64_100_oaat, OAATHasher, 100);
    w64_bench!(w64_100_lookup3, Lookup3Hasher, 100);
    w64_bench!(w64_100_passthrough, PassThroughHasher, 100);
    w64_bench!(w64_100_fnv1a64, FNV1aHasher64, 100);

    w64_bench!(w64_1000_default, DefaultHasher, 1000);
    w64_bench!(w64_1000_djb2, DJB2Hasher, 1000);
    w64_bench!(w64_1000_sdbm, SDBMHasher, 1000);
    w64_bench!(w64_1000_loselose, LoseLoseHasher, 1000);
    w64_bench!(w64_1000_oaat, OAATHasher, 1000);
    w64_bench!(w64_1000_lookup3, Lookup3Hasher, 1000);
    w64_bench!(w64_1000_passthrough, PassThroughHasher, 1000);
    w64_bench!(w64_1000_fnv1a64, FNV1aHasher64, 1000);

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
}

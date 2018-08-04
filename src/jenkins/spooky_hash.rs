//! From http://burtleburtle.net/bob/hash/spooky.html
//!
//! Quoted comments are from http://burtleburtle.net/bob/c/SpookyV2.h or
//! http://burtleburtle.net/bob/c/SpookyV2.cpp

use std::hash::Hasher;
use std::num::Wrapping;
use std::{mem, ptr};

/// number of uint64's in internal state
pub const SC_NUM_VARS: usize = 12;
/// size of the internal state in bytes
pub const SC_BLOCK_SIZE: usize = SC_NUM_VARS * 8;
/// size of buffer of unhashed data, in bytes
pub const SC_BUF_SIZE: usize = 2 * SC_BLOCK_SIZE;
/// > SC_CONST: a constant which:
/// >  * is not zero
/// >  * is odd
/// >  * is a not-very-regular mix of 1's and 0's
/// >  * does not need any other special mathematical properties
pub const SC_CONST: u64 = 0xdeadbeefdeadbeefu64;

#[inline]
pub fn rot64(x: Wrapping<u64>, k: usize) -> Wrapping<u64> {
    x << k | x >> (64 - k)
}

// > This is used if the input is 96 bytes long or longer.
// >
// > The internal state is fully overwritten every 96 bytes.
// > Every input bit appears to cause at least 128 bits of entropy
// > before 96 other bytes are combined, when run forward or backward
// >   For every input bit,
// >   Two inputs differing in just that input bit
// >   Where "differ" means xor or subtraction
// >   And the base value is random
// >   When run forward or backwards one Mix
// > I tried 3 pairs of each; they all differed by at least 212 bits.
#[inline]
pub fn mix(
    data: &[Wrapping<u64>],
    s0: &mut Wrapping<u64>,
    s1: &mut Wrapping<u64>,
    s2: &mut Wrapping<u64>,
    s3: &mut Wrapping<u64>,
    s4: &mut Wrapping<u64>,
    s5: &mut Wrapping<u64>,
    s6: &mut Wrapping<u64>,
    s7: &mut Wrapping<u64>,
    s8: &mut Wrapping<u64>,
    s9: &mut Wrapping<u64>,
    s10: &mut Wrapping<u64>,
    s11: &mut Wrapping<u64>,
) {
    *s0 += data[0];
    *s2 ^= *s10;
    *s11 ^= *s0;
    *s0 = rot64(*s0, 11);
    *s11 += *s1;
    *s1 += data[1];
    *s3 ^= *s11;
    *s0 ^= *s1;
    *s1 = rot64(*s1, 32);
    *s0 += *s2;
    *s2 += data[2];
    *s4 ^= *s0;
    *s1 ^= *s2;
    *s2 = rot64(*s2, 43);
    *s1 += *s3;
    *s3 += data[3];
    *s5 ^= *s1;
    *s2 ^= *s3;
    *s3 = rot64(*s3, 31);
    *s2 += *s4;
    *s4 += data[4];
    *s6 ^= *s2;
    *s3 ^= *s4;
    *s4 = rot64(*s4, 17);
    *s3 += *s5;
    *s5 += data[5];
    *s7 ^= *s3;
    *s4 ^= *s5;
    *s5 = rot64(*s5, 28);
    *s4 += *s6;
    *s6 += data[6];
    *s8 ^= *s4;
    *s5 ^= *s6;
    *s6 = rot64(*s6, 39);
    *s5 += *s7;
    *s7 += data[7];
    *s9 ^= *s5;
    *s6 ^= *s7;
    *s7 = rot64(*s7, 57);
    *s6 += *s8;
    *s8 += data[8];
    *s10 ^= *s6;
    *s7 ^= *s8;
    *s8 = rot64(*s8, 55);
    *s7 += *s9;
    *s9 += data[9];
    *s11 ^= *s7;
    *s8 ^= *s9;
    *s9 = rot64(*s9, 54);
    *s8 += *s10;
    *s10 += data[10];
    *s0 ^= *s8;
    *s9 ^= *s10;
    *s10 = rot64(*s10, 22);
    *s9 += *s11;
    *s11 += data[11];
    *s1 ^= *s9;
    *s10 ^= *s11;
    *s11 = rot64(*s11, 46);
    *s10 += *s0;
}

// > Mix all 12 inputs together so that h0, h1 are a hash of
// > them all.
// >
// > For two inputs differing in just the input bits Where
// > "differ" means xor or subtraction And the base value is
// > random, or a counting value starting at that bit The final
// > result will have each bit of h0, h1 flip For every input
// > bit, with probability 50 +- .3% For every pair of input
// > bits, with probability 50 +- 3%
// >
// > This does not rely on the last Mix() call having already
// > mixed some. Two iterations was almost good enough for a
// > 64-bit result, but a 128-bit result is reported, so End()
// > does three iterations.
//
//    static INLINE void EndPartial(
//        uint64 &h0, uint64 &h1, uint64 &h2, uint64 &h3,
//        uint64 &h4, uint64 &h5, uint64 &h6, uint64 &h7,
//        uint64 &h8, uint64 &h9, uint64 &h10,uint64 &h11)
//    {
//        h11+= h1;    h2 ^= h11;   h1 = Rot64(h1,44);
//        h0 += h2;    h3 ^= h0;    h2 = Rot64(h2,15);
//        h1 += h3;    h4 ^= h1;    h3 = Rot64(h3,34);
//        h2 += h4;    h5 ^= h2;    h4 = Rot64(h4,21);
//        h3 += h5;    h6 ^= h3;    h5 = Rot64(h5,38);
//        h4 += h6;    h7 ^= h4;    h6 = Rot64(h6,33);
//        h5 += h7;    h8 ^= h5;    h7 = Rot64(h7,10);
//        h6 += h8;    h9 ^= h6;    h8 = Rot64(h8,13);
//        h7 += h9;    h10^= h7;    h9 = Rot64(h9,38);
//        h8 += h10;   h11^= h8;    h10= Rot64(h10,53);
//        h9 += h11;   h0 ^= h9;    h11= Rot64(h11,42);
//        h10+= h0;    h1 ^= h10;   h0 = Rot64(h0,54);
//    }
#[inline]
pub fn end_partial(
    h0: &mut Wrapping<u64>,
    h1: &mut Wrapping<u64>,
    h2: &mut Wrapping<u64>,
    h3: &mut Wrapping<u64>,
    h4: &mut Wrapping<u64>,
    h5: &mut Wrapping<u64>,
    h6: &mut Wrapping<u64>,
    h7: &mut Wrapping<u64>,
    h8: &mut Wrapping<u64>,
    h9: &mut Wrapping<u64>,
    h10: &mut Wrapping<u64>,
    h11: &mut Wrapping<u64>,
) {
    *h11 += *h1;
    *h2 ^= *h11;
    *h1 = rot64(*h1, 44);
    *h0 += *h2;
    *h3 ^= *h0;
    *h2 = rot64(*h2, 15);
    *h1 += *h3;
    *h4 ^= *h1;
    *h3 = rot64(*h3, 34);
    *h2 += *h4;
    *h5 ^= *h2;
    *h4 = rot64(*h4, 21);
    *h3 += *h5;
    *h6 ^= *h3;
    *h5 = rot64(*h5, 38);
    *h4 += *h6;
    *h7 ^= *h4;
    *h6 = rot64(*h6, 33);
    *h5 += *h7;
    *h8 ^= *h5;
    *h7 = rot64(*h7, 10);
    *h6 += *h8;
    *h9 ^= *h6;
    *h8 = rot64(*h8, 13);
    *h7 += *h9;
    *h10 ^= *h7;
    *h9 = rot64(*h9, 38);
    *h8 += *h10;
    *h11 ^= *h8;
    *h10 = rot64(*h10, 53);
    *h9 += *h11;
    *h0 ^= *h9;
    *h11 = rot64(*h11, 42);
    *h10 += *h0;
    *h1 ^= *h10;
    *h0 = rot64(*h0, 54);
}

#[inline]
pub fn end(
    data: &[Wrapping<u64>],
    h0: &mut Wrapping<u64>,
    h1: &mut Wrapping<u64>,
    h2: &mut Wrapping<u64>,
    h3: &mut Wrapping<u64>,
    h4: &mut Wrapping<u64>,
    h5: &mut Wrapping<u64>,
    h6: &mut Wrapping<u64>,
    h7: &mut Wrapping<u64>,
    h8: &mut Wrapping<u64>,
    h9: &mut Wrapping<u64>,
    h10: &mut Wrapping<u64>,
    h11: &mut Wrapping<u64>,
) {
    *h0 += data[0];
    *h1 += data[1];
    *h2 += data[2];
    *h3 += data[3];
    *h4 += data[4];
    *h5 += data[5];
    *h6 += data[6];
    *h7 += data[7];
    *h8 += data[8];
    *h9 += data[9];
    *h10 += data[10];
    *h11 += data[11];
    end_partial(h0, h1, h2, h3, h4, h5, h6, h7, h8, h9, h10, h11);
    end_partial(h0, h1, h2, h3, h4, h5, h6, h7, h8, h9, h10, h11);
    end_partial(h0, h1, h2, h3, h4, h5, h6, h7, h8, h9, h10, h11);
}

/// > The goal is for each bit of the input to expand into 128
/// > bits of apparent entropy before it is fully overwritten. n
/// > trials both set and cleared at least m bits of h0 h1 h2 h3
/// >   n: 2   m: 29
/// >   n: 3   m: 46
/// >   n: 4   m: 57
/// >   n: 5   m: 107
/// >   n: 6   m: 146
/// >   n: 7   m: 152
/// > when run forwards or backwards for all 1-bit and 2-bit
/// > diffs with diffs defined by either xor or subtraction with
/// > a base of all zeros plus a counter, or plus another bit,
/// > or random
#[inline]
pub fn short_mix(
    h0: &mut Wrapping<u64>,
    h1: &mut Wrapping<u64>,
    h2: &mut Wrapping<u64>,
    h3: &mut Wrapping<u64>,
) {
    *h2 = rot64(*h2, 50);
    *h2 += *h3;
    *h0 ^= *h2;
    *h3 = rot64(*h3, 52);
    *h3 += *h0;
    *h1 ^= *h3;
    *h0 = rot64(*h0, 30);
    *h0 += *h1;
    *h2 ^= *h0;
    *h1 = rot64(*h1, 41);
    *h1 += *h2;
    *h3 ^= *h1;
    *h2 = rot64(*h2, 54);
    *h2 += *h3;
    *h0 ^= *h2;
    *h3 = rot64(*h3, 48);
    *h3 += *h0;
    *h1 ^= *h3;
    *h0 = rot64(*h0, 38);
    *h0 += *h1;
    *h2 ^= *h0;
    *h1 = rot64(*h1, 37);
    *h1 += *h2;
    *h3 ^= *h1;
    *h2 = rot64(*h2, 62);
    *h2 += *h3;
    *h0 ^= *h2;
    *h3 = rot64(*h3, 34);
    *h3 += *h0;
    *h1 ^= *h3;
    *h0 = rot64(*h0, 5);
    *h0 += *h1;
    *h2 ^= *h0;
    *h1 = rot64(*h1, 36);
    *h1 += *h2;
    *h3 ^= *h1;
}

/// > Mix all 4 inputs together so that h0, h1 are a hash of them all.
/// >
/// > For two inputs differing in just the input bits
/// > Where "differ" means xor or subtraction
/// > And the base value is random, or a counting value starting at that bit
/// > The final result will have each bit of h0, h1 flip
/// > For every input bit,
/// > with probability 50 +- .3% (it is probably better than that)
/// > For every pair of input bits,
/// > with probability 50 +- .75% (the worst case is approximately that)
#[inline]
pub fn short_end(
    h0: &mut Wrapping<u64>,
    h1: &mut Wrapping<u64>,
    h2: &mut Wrapping<u64>,
    h3: &mut Wrapping<u64>,
) {
    *h3 ^= *h2;
    *h2 = rot64(*h2, 15);
    *h3 += *h2;
    *h0 ^= *h3;
    *h3 = rot64(*h3, 52);
    *h0 += *h3;
    *h1 ^= *h0;
    *h0 = rot64(*h0, 26);
    *h1 += *h0;
    *h2 ^= *h1;
    *h1 = rot64(*h1, 51);
    *h2 += *h1;
    *h3 ^= *h2;
    *h2 = rot64(*h2, 28);
    *h3 += *h2;
    *h0 ^= *h3;
    *h3 = rot64(*h3, 9);
    *h0 += *h3;
    *h1 ^= *h0;
    *h0 = rot64(*h0, 47);
    *h1 += *h0;
    *h2 ^= *h1;
    *h1 = rot64(*h1, 54);
    *h2 += *h1;
    *h3 ^= *h2;
    *h2 = rot64(*h2, 32);
    *h3 += *h2;
    *h0 ^= *h3;
    *h3 = rot64(*h3, 25);
    *h0 += *h3;
    *h1 ^= *h0;
    *h0 = rot64(*h0, 63);
    *h1 += *h0;
}

// Short is used for messages under 192 bytes in length Short
// has a low startup cost, the normal mode is good for long
// keys, the cost crossover is at about 192 bytes. The two modes
// were held to the same quality bar.
pub fn short(message: &[u8], length: usize, hash1: &mut Wrapping<u64>, hash2: &mut Wrapping<u64>) {
    debug_assert!(length <= 192);
    // access the buffer as u64's
    let mut buffer: [Wrapping<u64>; 192 / 8] = [Wrapping(0); 192 / 8]; // 192 bytes, as u64 with wrapping ops.
    unsafe { ptr::copy_nonoverlapping(message.as_ptr(), &mut buffer as *mut _ as *mut u8, length) };
    let mut a = *hash1;
    let mut b = *hash2;
    let mut c = Wrapping(SC_CONST);
    let mut d = Wrapping(SC_CONST);
    let mut remainder = length % 32;
    let mut base = 0;
    if length > 15 {
        // handle complete sets of 32 bytes / 4 u64's
        let end = (length / 32) * 4;
        while base < end {
            c += buffer[base + 0];
            d += buffer[base + 1];
            short_mix(&mut a, &mut b, &mut c, &mut d);
            a += buffer[base + 2];
            b += buffer[base + 3];
            base += 4;
        }
        // handle the case of 16+ remaining bytes
        if remainder > 15 {
            c += buffer[base + 0];
            d += buffer[base + 1];
            short_mix(&mut a, &mut b, &mut c, &mut d);
            base += 2;
            remainder -= 16;
        }
    }
    // convert base to byte offsets
    base = base * 8;
    // handle bytes 0..16 and length
    d += Wrapping(length as u64) << 56;
    if remainder >= 12 {
        if remainder > 14 {
            d += Wrapping(message[base + 14] as u64) << 48;
        }
        if remainder > 13 {
            d += Wrapping(message[base + 13] as u64) << 40;
        }
        if remainder > 12 {
            d += Wrapping(message[base + 12] as u64) << 32;
        }
        c += Wrapping(load_int_le!(message, base, u64));
        d += Wrapping(load_int_le!(message, base + 8, u32) as u64);
    } else if remainder >= 8 {
        if remainder > 10 {
            d += Wrapping(message[base + 10] as u64) << 16;
        }
        if remainder > 9 {
            d += Wrapping(message[base + 9] as u64) << 8;
        }
        if remainder > 8 {
            d += Wrapping(message[base + 8] as u64);
        }
        c += Wrapping(load_int_le!(message, base, u64));
    } else if remainder >= 4 {
        if remainder > 6 {
            c += Wrapping(message[base + 6] as u64) << 48;
        }
        if remainder > 5 {
            c += Wrapping(message[base + 5] as u64) << 40;
        }
        if remainder > 4 {
            c += Wrapping(message[base + 4] as u64) << 32;
        }
        c += Wrapping(load_int_le!(message, base, u32) as u64);
    } else if remainder >= 1 {
        if remainder > 2 {
            c += Wrapping(message[base + 2] as u64) << 16;
        }
        if remainder > 1 {
            c += Wrapping(message[base + 1] as u64) << 8;
        }
        c += Wrapping(message[base] as u64);
    } else {
        c += Wrapping(SC_CONST);
        d += Wrapping(SC_CONST);
    }
    short_end(&mut a, &mut b, &mut c, &mut d);
    *hash1 = a;
    *hash2 = b;
}

pub struct SpookyHasher {
    // unhashed data, for partial messages; 2 * m_state, in bytes
    m_data: [u8; 2 * SC_NUM_VARS * 8],
    // internal state of the hash
    m_state: [Wrapping<u64>; SC_NUM_VARS],
    // total length of the input so far
    m_length: usize,
    // length of unhashed data stashed in m_data
    m_remainder: usize,
}

impl SpookyHasher {
    pub fn new(seed1: u64, seed2: u64) -> SpookyHasher {
        let mut sh = SpookyHasher {
            m_data: [0; 2 * SC_NUM_VARS * 8],
            m_state: [Wrapping(0u64); SC_NUM_VARS],
            m_length: 0,
            m_remainder: 0,
        };
        sh.m_state[0] = Wrapping(seed1);
        sh.m_state[1] = Wrapping(seed2);
        sh
    }

    pub fn finish128(&self) -> (u64, u64) {
        if self.m_length < SC_BUF_SIZE {
            let mut hash1 = self.m_state[0];
            let mut hash2 = self.m_state[1];
            short(&self.m_data, self.m_length, &mut hash1, &mut hash2);
            return (hash1.0, hash2.0);
        }
        // access m_data as u64's
        let mut data: [Wrapping<u64>; 2 * SC_NUM_VARS] = [Wrapping(0); 2 * SC_NUM_VARS];
        unsafe {
            ptr::copy_nonoverlapping(
                self.m_data.as_ptr(),
                &mut data as *mut _ as *mut u8,
                self.m_length,
            )
        };
        let mut remainder = self.m_remainder;
        let mut h0 = self.m_state[0];
        let mut h1 = self.m_state[1];
        let mut h2 = self.m_state[2];
        let mut h3 = self.m_state[3];
        let mut h4 = self.m_state[4];
        let mut h5 = self.m_state[5];
        let mut h6 = self.m_state[6];
        let mut h7 = self.m_state[7];
        let mut h8 = self.m_state[8];
        let mut h9 = self.m_state[9];
        let mut h10 = self.m_state[10];
        let mut h11 = self.m_state[11];
        let mut base = 0;
        if remainder > SC_BLOCK_SIZE {
            // handle the first, whole block
            mix(
                &data, &mut h0, &mut h1, &mut h2, &mut h3, &mut h4, &mut h5, &mut h6, &mut h7,
                &mut h8, &mut h9, &mut h10, &mut h11,
            );
            base = SC_BLOCK_SIZE;
            remainder -= SC_BLOCK_SIZE;
        }
        //
        unsafe {
            ptr::write_bytes(
                data.as_mut_ptr()
                    .offset(base as isize)
                    .offset(remainder as isize),
                0u8,
                SC_BLOCK_SIZE - remainder,
            );
            ptr::write_bytes(
                data.as_mut_ptr().offset((SC_BLOCK_SIZE as isize) - 1),
                remainder as u8,
                1,
            );
        }
        end(
            &mut data, &mut h0, &mut h1, &mut h2, &mut h3, &mut h4, &mut h5, &mut h6, &mut h7,
            &mut h8, &mut h9, &mut h10, &mut h11,
        );

        (h0.0, h1.0)
    }
}

impl Default for SpookyHasher {
    fn default() -> SpookyHasher {
        SpookyHasher::new(0, 0)
    }
}

impl Hasher for SpookyHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.finish128().0
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let new_length = self.m_remainder + bytes.len();
        if new_length < SC_BUF_SIZE {
            unsafe {
                ptr::copy_nonoverlapping(
                    bytes.as_ptr(),
                    (self.m_data.as_mut_ptr() as *mut u8).offset(self.m_remainder as isize),
                    bytes.len(),
                );
            }
            self.m_length += bytes.len();
            self.m_remainder = new_length;
            return;
        }
        // init the variables
        // let mut h0: u64;
        // let mut h1: u64; ,h2: u64; ,h3: u64; ,h4: u64; ,h5: u64; ,h6: u64; ,h7: u64; ,h8: u64; ,h9: u64; ,h10: u64; ,h11: u64;
        //    if (self.m_length < SC_BUF_SIZE)
        //    {
        //        h0=h3=h6=h9  = self.m_state[0];
        //        h1=h4=h7=h10 = self.m_state[1];
        //        h2=h5=h8=h11 = SC_CONST;
        //    }
        //    else
        //    {
        //        h0 = self.m_state[0];
        //        h1 = self.m_state[1];
        //        h2 = self.m_state[2];
        //        h3 = self.m_state[3];
        //        h4 = self.m_state[4];
        //        h5 = self.m_state[5];
        //        h6 = self.m_state[6];
        //        h7 = self.m_state[7];
        //        h8 = self.m_state[8];
        //        h9 = self.m_state[9];
        //        h10 = self.m_state[10];
        //        h11 = self.m_state[11];
        //    }
        //    self.m_length += length;
    }
}

#[cfg(test)]
mod spookyhash_test {
    use super::*;

    #[test]
    fn basic() {
        let mut sh = SpookyHasher::default();
        sh.write(b"");
        assert_eq!(sh.finish(), 2533000996631939353);
    }
}

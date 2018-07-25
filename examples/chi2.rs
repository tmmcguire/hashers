extern crate rand;

extern crate hashers;

use hashers::{builtin, jenkins, null, oz};

mod samples;

// This function is taken (loosely) from
//
// http://burtleburtle.net/bob/hash/hashfaq.html.
//
// Ok, so the chi2 (χ^2) test can be used to determine if a sample is
// drawn from a uniform distribution. This is useful because a hash
// table with n buckets and m items should have m/n items in each
// bucket (modulo open addressing). Jenkins' page above describes how
// to do this test, although I could not get his computation of chi2 to
// produce reasonable values, so I went with the algorithm presented at
//
// http://www.statisticshowto.com/probability-and-statistics/chi-square/.
//
// No idea if that's right either. And it's not remotely consistent.
//
// It's a giant piece of crap. I hate statistics.
fn chi2(sample: &[Vec<u8>], hfcn: fn(&[u8]) -> u64, mask_size: u32) -> f64 {
    let n_buckets = 2usize.pow(mask_size);
    let mask: u64 = n_buckets as u64 - 1;
    // count the hashes in each bucket
    let mut buckets: Vec<usize> = vec![0; n_buckets];
    sample
        .iter()
        .map(|s| (hfcn(s) & mask) as usize)
        .for_each(|s| buckets[s] += 1);
    // expected uniformly distributed samples
    let expected: f64 = (sample.len() as f64) / (n_buckets as f64);

    let chi2: f64 = buckets
        .iter()
        .map(|&c| (c as f64) - expected)
        .map(|r| (r * r) / expected)
        .sum();
    (chi2 - (n_buckets as f64)) / (n_buckets as f64).sqrt()
}

fn do_hashes(samples: &[Vec<u8>]) {
    println!("default:  {}", chi2(&samples, builtin::default, 7));
    println!("djb2:     {}", chi2(&samples, oz::djb2, 7));
    println!("lookup3:  {}", chi2(&samples, jenkins::lookup3, 7));
    println!("loselose: {}", chi2(&samples, oz::loselose, 7));
    println!("null:     {}", chi2(&samples, null::null, 7));
    println!("OAAT:     {}", chi2(&samples, jenkins::oaat, 7));
    println!("Pass:     {}", chi2(&samples, null::passthrough, 7));
    println!("sdbm:     {}", chi2(&samples, oz::sdbm, 7));
}

fn main() {
    println!("Uniform distribution");
    let s1 = samples::random_samples(&mut samples::uniform(), 1000, 6);
    do_hashes(&s1);

    println!("\nAlphanumeric distribution");
    let s2 = samples::alphanumeric_samples(1000, 6);
    do_hashes(&s2);

    println!("\nGenerated identifiers");
    let s3 = samples::generated_samples(1000, 6);
    do_hashes(&s3);

    println!("\nDictionary words");
    let s4 = samples::word_samples();
    do_hashes(&s4);
}

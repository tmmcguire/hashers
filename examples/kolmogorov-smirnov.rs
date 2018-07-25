extern crate hashers;

// See
// - https://www.itl.nist.gov/div898/handbook/eda/section3/eda35g.htm
// - https://onlinecourses.science.psu.edu/stat414/node/322/

fn word_samples() -> Vec<Vec<u8>> {
    use std::fs::File;
    use std::io::BufReader;
    use std::io::prelude::*;

    let file = File::open("./data/words.txt").expect("cannot open words.txt");
    BufReader::new(file)
        .lines()
        .map(|l| l.expect("bad read"))
        .map(|w| w.as_bytes().to_vec())
        .collect()
}

fn generated_samples(n: usize, s: usize) -> Vec<Vec<u8>> {
    (0..n)
        .map(|v| format!("a{:0width$}", v, width = s).as_bytes().to_vec())
        .collect()
}

fn do_hashes(fcn: fn(&[u8]) -> u64, data: &[Vec<u8>]) -> Vec<u64> {
    let mut res: Vec<u64> = data.iter().map(|elt| fcn(elt)).collect();
    res.sort();
    res
}

fn cdf_uniform(x: u64) -> f64 {
    (x as f64) / (std::u64::MAX as f64)
}

fn d(samples: &[u64]) -> f64 {
    let n = samples.len() as f64;
    let mut last_ecdf = 0.0f64;
    let mut max = std::f64::MIN;
    for (i, x) in samples.iter().enumerate() {
        let tcdf = (i as f64) / n;
        let next_ecdf = cdf_uniform(*x);
        let d1 = (last_ecdf - tcdf).abs();
        let d2 = (tcdf - next_ecdf).abs();
        max = if d1 > max { d1 } else { max };
        max = if d2 > max { d2 } else { max };
        last_ecdf = next_ecdf;
    }
    max
}

fn print_ds(sample: &str, hash: &str, d: f64) {
    println!("{}/{} {}", sample, hash, d);
}

fn main() {
    print_ds("word_samples", "null    ", d(&do_hashes(hashers::null::null, &word_samples())));
    print_ds("word_samples", "passthru", d(&do_hashes(hashers::null::passthrough, &word_samples())));
    print_ds("word_samples", "default ", d(&do_hashes(hashers::builtin::default, &word_samples())));
    print_ds("word_samples", "loselose", d(&do_hashes(hashers::oz::loselose, &word_samples())));
    print_ds("word_samples", "sdbm    ", d(&do_hashes(hashers::oz::sdbm, &word_samples())));
    print_ds("word_samples", "djb2    ", d(&do_hashes(hashers::oz::djb2, &word_samples())));
    print_ds("word_samples", "oaat    ", d(&do_hashes(hashers::jenkins::oaat, &word_samples())));
    print_ds("word_samples", "lookup3 ", d(&do_hashes(hashers::jenkins::lookup3, &word_samples())));

    print_ds("generated   ", "null    ", d(&do_hashes(hashers::null::null, &generated_samples(10000, 6))));
    print_ds("generated   ", "passthru", d(&do_hashes(hashers::null::passthrough, &generated_samples(10000, 6))));
    print_ds("generated   ", "default ", d(&do_hashes(hashers::builtin::default, &generated_samples(10000, 6))));
    print_ds("generated   ", "loselose", d(&do_hashes(hashers::oz::loselose, &generated_samples(10000, 6))));
    print_ds("generated   ", "sdbm    ", d(&do_hashes(hashers::oz::sdbm, &generated_samples(10000, 6))));
    print_ds("generated   ", "djb2    ", d(&do_hashes(hashers::oz::djb2, &generated_samples(10000, 6))));
    print_ds("generated   ", "oaat    ", d(&do_hashes(hashers::jenkins::oaat, &generated_samples(10000, 6))));
    print_ds("generated   ", "lookup3 ", d(&do_hashes(hashers::jenkins::lookup3, &generated_samples(10000, 6))));
}

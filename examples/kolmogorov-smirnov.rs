extern crate rand;
extern crate hashers;

mod samples;

// See
// - https://www.itl.nist.gov/div898/handbook/eda/section3/eda35g.htm
// - https://onlinecourses.science.psu.edu/stat414/node/322/

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
    println!("{} {} {}", sample, hash, d);
}

fn run_sample(name: &str, samples: &[Vec<u8>]) {
    print_ds(name, "null    ", d(&do_hashes(hashers::null::null, samples)));
    print_ds(name, "passthru", d(&do_hashes(hashers::null::passthrough, samples)));
    print_ds(name, "default ", d(&do_hashes(hashers::builtin::default, samples)));
    print_ds(name, "loselose", d(&do_hashes(hashers::oz::loselose, samples)));
    print_ds(name, "sdbm    ", d(&do_hashes(hashers::oz::sdbm, samples)));
    print_ds(name, "djb2    ", d(&do_hashes(hashers::oz::djb2, samples)));
    print_ds(name, "oaat    ", d(&do_hashes(hashers::jenkins::oaat, samples)));
    print_ds(name, "lookup3 ", d(&do_hashes(hashers::jenkins::lookup3, samples)));
    print_ds(name, "fnv1a 64", d(&do_hashes(hashers::fnv::fnv1a64, samples)));
}

fn main() {
    run_sample("random      ", &samples::random_samples(&mut samples::uniform(), 1000, 6));
    run_sample("alphanumeric", &samples::alphanumeric_samples(10000, 6));
    run_sample("generated   ", &samples::generated_samples(10000, 6));
    run_sample("word_samples", &samples::word_samples());
    println!("");
    print_ds("generated", "fibo djb2", d(&do_hashes(hashers::fibonacci::fibo_djb2, &samples::generated_samples(10000, 6))));
    print_ds("generated", "fibo default", d(&do_hashes(hashers::fibonacci::fibo_default, &samples::generated_samples(10000, 6))));
}

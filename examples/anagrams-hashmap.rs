#![feature(duration_as_u128)]

extern crate hashers;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::hash::{BuildHasher, BuildHasherDefault, Hasher};
use std::io::{BufRead, BufReader};
use std::time::Instant;

use hashers::{builtin, fibonacci, fnv, fx_hash, jenkins, oz};

pub mod combinations;

fn get_letters(s: &str) -> Vec<u8> {
    let mut t: Vec<char> = s.chars().collect();
    t.sort();
    return t.iter().map(|&ch| ch as u8).collect();
}

fn split_words(s: &str) -> Vec<String> {
    s.split(" ").map(|w| w.to_string()).collect()
}

type Dictionary<BH> = HashMap<Vec<u8>, Vec<String>, BH>;
type StringSet<BH> = HashSet<String, BH>;

fn load_dictionary<H: Default + Hasher>() -> Dictionary<BuildHasherDefault<H>> {
    let file = match File::open("./data/anadict.txt") {
        Ok(f) => f,
        Err(e) => panic!(e),
    };
    let buffered_file = BufReader::new(file);
    let mut map = HashMap::default();
    for line in buffered_file.lines() {
        let line = line.unwrap();
        let mut words = split_words(&line);
        let key: Vec<u8> = words.remove(0).chars().map(|ch| ch as u8).collect();
        map.insert(key, words);
    }
    return map;
}

fn search<H: Default + Hasher, BH: BuildHasher>(
    letters: &[u8],
    dictionary: &Dictionary<BH>,
) -> StringSet<BuildHasherDefault<H>> {
    let mut set = HashSet::default();
    for i in 0..letters.len() + 1 {
        let mut key: Vec<u8> = vec![0; i];
        combinations::each_combination(letters, i, |combo| {
            for j in 0..combo.len() {
                key[j] = combo[j];
            }
            match dictionary.get(&key) {
                Some(val) => {
                    for word in val.iter() {
                        set.insert(word.clone());
                    }
                }
                None => {}
            }
        });
    }
    return set;
}

fn do_search<H: Default + Hasher>() -> usize {
    let letters = get_letters("asdwtribnowplfglewhqagnbe");
    let dictionary = load_dictionary::<H>();
    let set = search::<H, BuildHasherDefault<H>>(&letters, &dictionary);
    set.len()
}

fn time<H: Default + Hasher>(title: &str, baseline: f64) -> f64 {
    let start = Instant::now();
    assert_eq!(do_search::<H>(), 7440);
    let duration = Instant::now().duration_since(start);
    if baseline > 0.0 {
        let percent = ((duration.as_micros() as f64 / baseline) * 1000.0).round() / 10.0;
        println!("{} {:?} ({}%)", title, duration, percent);
    } else {
        println!("{} {:?}", title, duration);
    }
    duration.as_micros() as f64
}

fn main() {
    let baseline = time::<builtin::DefaultHasher>("default", 0.0);
    time::<oz::DJB2Hasher>("djb2", baseline);
    time::<oz::SDBMHasher>("sdbm", baseline);
    time::<jenkins::OAATHasher>("oaat", baseline);
    time::<jenkins::Lookup3Hasher>("lookup3", baseline);
    time::<fnv::FNV1aHasher32>("fnv-1a 32", baseline);
    time::<fnv::FNV1aHasher64>("fnv-1a 64", baseline);
    time::<fx_hash::FxHasher>("fxhash", baseline);
    time::<fx_hash::FxHasher32>("fxhash32", baseline);
    time::<fx_hash::FxHasher64>("fxhash64", baseline);
    time::<fibonacci::FibonacciWrapper<oz::DJB2Hasher>>("fibo djb2", baseline);
    time::<fibonacci::FibonacciWrapper<oz::SDBMHasher>>("fibo sdbm", baseline);
    time::<fibonacci::FibonacciWrapper<jenkins::OAATHasher>>("fibo oaat", baseline);
    time::<fibonacci::FibonacciWrapper<jenkins::Lookup3Hasher>>("fibo lookup3", baseline);
    time::<fibonacci::FibonacciWrapper<fx_hash::FxHasher>>("fibo fxhash", baseline);
    time::<fibonacci::FibonacciWrapper<fx_hash::FxHasher32>>("fibo fxhash32", baseline);
    time::<fibonacci::FibonacciWrapper<fx_hash::FxHasher64>>("fibo fxhash64", baseline);
}

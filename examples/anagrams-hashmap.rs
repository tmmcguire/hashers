extern crate hashers;

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::hash::{BuildHasher, BuildHasherDefault, Hasher};
use std::io::{BufRead, BufReader};
use std::time::Instant;

use hashers::{builtin, fibonacci, fnv, jenkins, oz};

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

fn time<H: Default + Hasher>(title: &str) {
    let start = Instant::now();
    assert_eq!(do_search::<H>(), 7440);
    println!("{} {:?}", title, Instant::now().duration_since(start));
}

fn main() {
    time::<builtin::DefaultHasher>("default");
    time::<oz::DJB2Hasher>("djb2");
    time::<oz::SDBMHasher>("sdbm");
    time::<jenkins::OAATHasher>("oaat");
    time::<jenkins::Lookup3Hasher>("lookup3");
    time::<fnv::FNV1aHasher32>("fnv-1a 32");
    time::<fnv::FNV1aHasher64>("fnv-1a 64");
    time::<fibonacci::FibonacciWrapper<oz::DJB2Hasher>>("fibo djb2");
    time::<fibonacci::FibonacciWrapper<oz::SDBMHasher>>("fibo sdbm");
    time::<fibonacci::FibonacciWrapper<jenkins::OAATHasher>>("fibo oaat");
    time::<fibonacci::FibonacciWrapper<jenkins::Lookup3Hasher>>("fibo lookup3");
}

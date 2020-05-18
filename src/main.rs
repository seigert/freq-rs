use std::{
    fs::File,
    io::{BufWriter, Read, Write},
};

use clap::Clap;
use fnv::FnvHashMap;
use memmap::*;

const H: u32 = 2166136261;
const P: u32 = 0x1000193;

/// Counts number of unique `[a-zA-Z]+` words in input.
#[derive(Clap, Debug)]
#[clap(version = "0.1.0")]
struct Opts {
    /// Name of input file, or `-` if STDIN should be used.
    input: String,
    /// Name of output file, or `-` if STDOUT should be used.
    output: Option<String>,
}

struct FreqDict {
    hashes: FnvHashMap<u32, FreqEntry>,
    collisions: FnvHashMap<Box<[u8]>, u32>,
}

enum FreqEntry {
    Collision,
    Value(Box<[u8]>, u32),
}

fn main() {
    let opts: Opts = Opts::parse();
    let input = open_mmap(&opts);

    let mut word = Vec::with_capacity(256);
    let mut hash = H;
    let mut dict = FreqDict::new();

    for &byte in input.iter() {
        if b'a' <= byte && byte <= b'z' {
            word.push(byte);
            hash = (hash ^ byte as u32) * P;
            continue;
        } else if b'A' <= byte && byte <= b'Z' {
            let byte = byte ^ 0x20;
            word.push(byte);
            hash = (hash ^ byte as u32) * P;
            continue;
        } else if !word.is_empty() {
            dict.add_word(&word, hash);
            word.clear();
            hash = H;
        }
    }
    if !word.is_empty() {
        dict.add_word(&word, hash);
    }

    let mut output = create_output(&opts);
    for (count, word) in dict.get_freq() {
        writeln!(&mut output, "{} {}", count, word).unwrap_or_else(|e| {
            let output = opts.output.as_ref().map_or("-", |s| s.as_str());
            panic!("Unable to write results in '{}': {}", output, e)
        })
    }
}

impl FreqDict {
    fn new() -> Self {
        FreqDict {
            hashes: FnvHashMap::default(),
            collisions: FnvHashMap::default(),
        }
    }

    #[inline]
    fn add_word(&mut self, word: &[u8], hash: u32) {
        if let Some(entry) = self.hashes.get_mut(&hash) {
            match entry {
                FreqEntry::Collision => {
                    if let Some(counter) = self.collisions.get_mut(word) {
                        *counter += 1;
                    } else {
                        self.collisions.insert(word.into(), 1);
                    }
                }
                FreqEntry::Value(ref value, ref mut counter) =>
                    if word == value.as_ref() {
                        *counter += 1;
                    } else {
                        if let FreqEntry::Value(value, counter) = std::mem::replace(entry, FreqEntry::Collision) {
                            self.collisions.insert(value, counter);
                            self.collisions.insert(word.into(), 1);
                        }
                    }
            }
        } else {
            self.hashes.insert(hash, FreqEntry::Value(word.into(), 1));
        }
    }

    fn get_freq(&self) -> Vec<(u32, &str)> {
        let hashed = self.hashes.values()
            .filter_map(|e| match e {
                FreqEntry::Collision => None,
                FreqEntry::Value(value, counter) => {
                    let key = std::str::from_utf8(value).unwrap();
                    Some((*counter, key))
                }
            });
        let collided = self.collisions.iter().map(|(value, counter)| {
            let key = std::str::from_utf8(value).unwrap();
            (*counter, key)
        });

        let mut freq = Iterator::chain(hashed, collided).collect::<Vec<_>>();
        freq.sort_unstable_by(|(c1, w1), (c2, w2)| {
            Ord::cmp(c1, c2).reverse().then_with(|| Ord::cmp(w1, w2))
        });
        freq
    }
}

fn open_mmap(opts: &Opts) -> Mmap {
    match opts.input.as_str() {
        "-" => {
            let mut buffer = vec![];
            std::io::stdin()
                .read_to_end(&mut buffer)
                .unwrap_or_else(|e| panic!("Unable to read STDIN: {}", e));
            let mut mmap = MmapOptions::new()
                .len(buffer.len())
                .map_anon()
                .unwrap_or_else(|e| panic!("Unable to read STDIN: {}", e));
            mmap.copy_from_slice(&buffer);
            mmap.make_read_only()
                .unwrap_or_else(|e| panic!("Unable to read STDIN: {}", e))
        }
        fnm => {
            let file = File::open(fnm)
                .unwrap_or_else(|e| panic!("Unable to open '{}' for reading: {}", fnm, e));
            unsafe {
                MmapOptions::new()
                    .map(&file)
                    .unwrap_or_else(|e| panic!("Unable to read '{}' in memory: {}", fnm, e))
            }
        }
    }
}

fn create_output(opts: &Opts) -> Box<dyn Write> {
    match opts.output.as_deref() {
        Some("-") | None => Box::new(BufWriter::new(std::io::stdout())),
        Some(fnm) => {
            let file = File::create(fnm)
                .unwrap_or_else(|e| panic!("Unable to open '{}' for writing: {}", fnm, e));
            Box::new(BufWriter::new(file))
        }
    }
}

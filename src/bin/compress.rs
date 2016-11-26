extern crate lz77;
extern crate clap;

use std::fs::File;
use std::io::{copy, BufReader, BufWriter};
use lz77::{Lz77Encoder, LinearSearcher, Lz77Options};
use clap::{App, Arg};

fn main() {
    let matches = App::new("Lz77 compressor")
        .version("0.1")
        .author("Moritz Wanzenböck <moritz.wanzenboeck@gmail.com>")
        .about("Compresses files using pure LZ77")
        .arg(Arg::with_name("INPUT")
            .required(true)
            .help("Sets the input file to use")
            .index(1))
        .arg(Arg::with_name("OUTPUT")
            .required(true)
            .help("Sets the output file to use")
            .index(2))
        .get_matches();

    let infile = matches.value_of("INPUT").and_then(|file| File::open(file).ok()).expect("Input: No such file");
    let outfile = matches.value_of("OUTPUT").and_then(|file| File::create(file).ok()).expect("Output: Could not create file");

    let mut read = BufReader::new(infile);
    let write = BufWriter::new(outfile);

    let opts = Lz77Options {
        window_size: 12,
    };
    let mut encoder = Lz77Encoder::<_, LinearSearcher>::new(write, opts);
    copy(&mut read, &mut encoder).expect("Something went wrong while encoding");
}
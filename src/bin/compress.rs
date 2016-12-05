extern crate lz77;
extern crate clap;

mod util;

use std::fs::File;
use std::io::{copy, BufReader, BufWriter, Write};
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
        .arg(Arg::with_name("window_size")
            .default_value("12")
            .short("w")
            .long("window"))
        .arg(Arg::with_name("verbose")
            .short("v")
            .help("Sets verbose output"))
        .get_matches();

    let infile = matches.value_of("INPUT")
        .and_then(|file| File::open(file).ok())
        .expect("Input: No such file");
    let outfile = matches.value_of("OUTPUT")
        .and_then(|file| File::create(file).ok())
        .expect("Output: Could not create file");
    let window_size: u8 = matches.value_of("window_size")
        .and_then(|size| size.parse::<u8>().ok())
        .expect("Invalid value for window_size");

    let mut read = util::StatsReader::new(BufReader::new(infile));
    let mut write = util::StatsWriter::new(BufWriter::new(outfile));

    let opts = Lz77Options { window_size: window_size };
    {
        let mut encoder = Lz77Encoder::<_, LinearSearcher>::new(&mut write, opts);
        copy(&mut read, &mut encoder).expect("Something went wrong while encoding");
    }
    if matches.is_present("verbose") {
        write.flush().expect("Flush failed");
        let written = write.get_stats().processed;
        let read = read.get_stats().processed;

        println!("Read:    {} bytes", read);
        println!("Written: {} bytes", written);
    }
}

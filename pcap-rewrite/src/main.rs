#[macro_use]
extern crate log;

extern crate clap;
use clap::{crate_version, App, Arg};

// extern crate env_logger;
extern crate flate2;
extern crate pcap_parser;
extern crate std_logger;
extern crate xz2;

use std::fs::File;
use std::io;
use std::path::Path;

use flate2::read::GzDecoder;
use xz2::read::XzDecoder;

use libpcap_tools::{Config, PcapEngine};

mod rewriter;
use crate::rewriter::*;

fn load_config(config: &mut Config, filename: &str) -> Result<(), io::Error> {
    debug!("Loading configuration {}", filename);
    let path = Path::new(&filename);
    let file = File::open(path)?;
    config.load_config(file)
}

fn main() -> io::Result<()> {
    let matches = App::new("Pcap rewrite tool")
        .version(crate_version!())
        .author("Pierre Chifflier")
        .about("Tool for rewriting pcap files")
        .arg(
            Arg::with_name("plugins")
                .help("Plugins to load (default: all)")
                .short("p")
                .long("plugins")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("config")
                .help("Configuration file")
                .short("c")
                .long("config")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("INPUT")
                .help("Input file name")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("OUTPUT")
                .help("Output file name")
                .required(true)
                .index(2),
        )
        .get_matches();

    // env_logger::init();
    std_logger::init();

    debug!("Pcap rewrite tool {}", crate_version!());

    let mut config = Config::default();
    if let Some(filename) = matches.value_of("config") {
        load_config(&mut config, filename)?;
    }

    let input_filename = matches.value_of("INPUT").unwrap();
    let output_filename = matches.value_of("OUTPUT").unwrap();

    let mut input_reader = if input_filename == "-" {
        Box::new(io::stdin())
    } else {
        let path = Path::new(&input_filename);
        let file = File::open(path)?;
        if input_filename.ends_with(".gz") {
            Box::new(GzDecoder::new(file))
        } else if input_filename.ends_with(".xz") {
            Box::new(XzDecoder::new(file))
        } else {
            Box::new(file) as Box<dyn io::Read>
        }
    };
    let path = Path::new(&output_filename);
    let outfile = File::create(path)?;

    let rewriter = Rewriter::new(Box::new(outfile));
    let mut engine = PcapEngine::new(Box::new(rewriter), &config);

    let _ = engine.run(&mut input_reader).expect("run analyzer");

    Ok(())
}

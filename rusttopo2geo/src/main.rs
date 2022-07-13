extern crate clap;
extern crate rust_topojson_client;

use std::fs::File;
use std::io;
use std::io::prelude::*;

use clap::App;
use clap::Arg;
use geo::CoordFloat;
use geo::CoordNum;
use rust_topojson_client::feature::Builder as FeatureBuilder;
use serde::Serialize;
use serde_json::to_writer;
// use topojson::Geometry;
use topojson::Topology;

use geo::Geometry;

struct MissingObject();

fn main() -> io::Result<()> {
    let matches = App::new("topo2geo")
        .version("1.0")
        .author("Martin F. <martinfrances107@hotmail.com>")
        .about("Converts TopoJSON objects to GeoJSON features.")
        .arg(
            Arg::with_name("INPUT")
                .short('i')
                .long("in")
                .value_name("FILE")
                .help("input topology file name; defaults to “-” for stdin")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("LIST")
                .long("list")
                .short('l')
                .help("list the object names on the input topology")
                // .index(1)
                .required(false),
        )
        .arg(
            Arg::with_name("NEWLINE")
                .short('n')
                .long("newline-delimited")
                .multiple(true)
                .help("output newline-delimted JSON"),
        )
        .get_matches();

    let filename = matches.value_of("INPUT");

    match filename {
        Some(filename) => {
            println!("using input file: {}", filename);
        }
        None => {
            println!("using stdIn");
        }
    }

    // Convert
    let topo = read(filename)?;

    match matches.occurrences_of("LIST") {
        0 => {
            println!("writing file.");
            write(&topo)
        }
        _ => {
            println!("objects :-");
            write_list(&topo);
            Ok(())
        }
    }
}

fn read(filename: Option<&str>) -> io::Result<Topology> {
    let mut buffer = String::new();
    match filename {
        Some(filename) => {
            let mut f = File::open(filename)?;
            f.read_to_string(&mut buffer).expect("failed to read input");
        }
        None => {
            io::stdin().read_to_string(&mut buffer)?;
        }
    }

    let t = serde_json::from_str(&buffer).expect("Failed to parse");
    Ok(t)
}

fn write_list(topo: &Topology) {
    for ng in &topo.objects {
        println!("\t{}", ng.name);
    }
}

fn write(topo: &Topology) -> io::Result<()> {
    let name = "countries";

    // let not_found = topo.objects.iter().find(|ng| ng.name == name).is_none();
    let has_object = topo.objects.iter().any(|ng| ng.name == name);
    println!("has_object {}", has_object);
    if !has_object {
        panic!("error: object {} not found", name)
    }

    if let Some(feature) = FeatureBuilder::generate_from_name::<f64>(topo, name) {
        //TODO refactor of newlinedelited option
        println!("about to write");
        write_feature("out.txt", &feature)?;
    }
    // write_feature(file, feature);

    // Signal nothing to write.
    Ok(())
}

fn write_feature<T>(file: &str, feature: &Geometry<T>) -> io::Result<()>
where
    T: CoordNum + Serialize,
{
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    handle.write_all(b"hello world")?;

    // to_writer(handle, feature);
    println!("{:#?}", feature);
    Ok(())
}

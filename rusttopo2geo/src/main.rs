extern crate clap;
extern crate rust_topojson_client;

use std::fs::File;
use std::io;
use std::io::prelude::*;

use clap::Arg;
use clap::Command;
use geo::CoordNum;
use geo::Geometry;
use serde::Serialize;
use topojson::Topology;

use rust_topojson_client::feature::feature_from_name;

fn main() -> io::Result<()> {
    let matches = Command::new("topo2geo")
        .version("0.1")
        .author("Martin Frances <martinfrances107@hotmail.com>")
        .about("Converts TopoJSON objects to GeoJSON features.")
        .arg(
            Arg::new("INPUT")
                .short('i')
                .long("in")
                .value_name("FILE")
                .help("input topology file name; defaults to “-” for stdin")
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new("LIST")
                .long("list")
                .short('l')
                .help("list the object names on the input topology")
                .required(false),
        )
        .arg(
            Arg::new("NEWLINE")
                .short('n')
                .long("newline-delimited")
                .help("output newline-delimted JSON"),
        )
        .get_matches();

    let filename = matches.get_one::<String>("INPUT");

    let topo = read(filename.map(|x| &**x))?;

    if matches.contains_id("LIST") {
        write_list(&topo);
    } else if matches.contains_id("INPUT") {
        write(&topo)?
    }
    Ok(())
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

    let has_object = topo.objects.iter().any(|ng| ng.name == name);
    println!("has_object {}", has_object);
    if !has_object {
        panic!("error: object {} not found", name)
    }

    if let Some(feature) = feature_from_name::<f64>(topo, name) {
        //TODO refactor of newlinedelited option
        println!("about to write");
        write_feature("out.txt", &feature)?;
    }

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

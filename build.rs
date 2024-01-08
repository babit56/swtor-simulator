use std::io::{BufWriter, Write};
use std::{io::BufReader, fs::File};
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
struct Gom {
    associations: HashMap<u64, String>,
    classes: HashMap<u64, String>,
    enums: HashMap<u64, String>,
    fields: HashMap<u64, String>,
}

#[derive(Deserialize, Debug, Clone)]
struct ClientGom {
    enums: HashMap<u64, Vec<String>>,
    // fields:
    // classes:
}

fn create_enums(mut w: BufWriter<File>) {
    let client_gom_file = File::open("data/clientGom.json").expect("clientGom.json should exist");
    let client_gom: ClientGom = serde_json::from_reader(BufReader::new(client_gom_file)).expect("json should be good");
    let gom_file = File::open("data/GOM.json").expect("GOM.json should exist");
    let gom: Gom = serde_json::from_reader(BufReader::new(gom_file)).expect("json should be good");

    for (id, name) in gom.enums {
        let variants = &client_gom.enums[&id];
        writeln!(&mut w,
                 "#[derive(Debug, strum::Display, strum::EnumString)]\npub enum {name} {{\n{}}}",
                 variants.join(",\n")).unwrap()
    }
    // TODO: Add id <-> enum perhaps?
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=data/");
}

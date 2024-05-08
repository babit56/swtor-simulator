use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::{io::BufReader, fs::File};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
struct Gom {
    // associations: HashMap<u64, String>,
    // classes: HashMap<u64, String>,
    enums: HashMap<u64, String>,
    // fields: HashMap<u64, String>,
}

#[derive(Deserialize, Debug, Clone)]
struct ClientGom {
    enums: HashMap<u64, Vec<String>>,
    // fields: // Probably only needed to get enum/noderef/etc. ids from field id
    // classes: // Probably not necessary
}

fn create_enums(out_dir: &Path) {
    let enums_file = File::create(out_dir.join("enums.rs")).expect("should be able to create file");
    let mut enums = BufWriter::new(enums_file);
    let type_reg_file = File::create(out_dir.join("type_registration.rs")).expect("should be able to create file");
    let mut type_reg = BufWriter::new(type_reg_file);

    writeln!(&mut type_reg, "app").expect("should write");

    let client_gom_file = File::open("data/clientGom.json").expect("clientGom.json should exist");
    let client_gom: ClientGom = serde_json::from_reader(BufReader::new(client_gom_file)).expect("json should be good");
    let gom_file = File::open("data/GOM.json").expect("GOM.json should exist");
    let gom: Gom = serde_json::from_reader(BufReader::new(gom_file)).expect("json should be good");

    for (id, name) in gom.enums {
        let variants = match client_gom.enums.get(&id) {
            None => continue,
            Some(v) => v,
        };
        if variants.iter().find(|var| var == &"move").is_some() {
            // TODO: Solve this problem (can't have reserved words as enum variant)
            continue;
        }
        if variants.len() == 0 {
            continue;
        }
        writeln!(&mut enums,
                 "#[derive(Debug, strum::Display, strum::EnumString, Component, Reflect)]\npub enum {name} {{\n    {},\n}}",
                 variants.join(",\n    ")).expect("should write");
        writeln!(&mut type_reg, ".register_type::<types::{name}>()").expect("should write");
    }
    // TODO: Add id <-> enum perhaps?
}

#[derive(Deserialize, Debug)]
pub struct Node {
    pub id: String,
    pub fqn: String,
    pub path: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CodeClass(pub Vec<serde_json::Value>);

#[derive(Deserialize, Debug)]
pub struct NodeObjPair {
    pub node: Node,
    pub obj: CodeClass,
}

fn create_abilities(out_dir: &Path) {
    let abl_file = File::open("data/abl.json").expect("abl.json should exist");
    let abl: Vec<NodeObjPair> = serde_json::from_reader(BufReader::new(abl_file)).expect("json should be good");

    for pair in abl {
        let mut path_str = pair.node.fqn.replace(".", "/");
        path_str.push_str(".json");
        let path = out_dir.join(path_str);
        let parent = path.parent().expect("should have parent");
        fs::create_dir_all(parent).expect("can create directories");
        let mut file = File::create(path).expect("should be able to create file");
        let json_string = serde_json::to_string(&pair.obj).expect("should get json");
        file.write_all(json_string.as_bytes()).expect("should be able to write");
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=data/");
    let out_dir_str = std::env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir_str);
    create_enums(out_dir);
    create_abilities(out_dir);
}

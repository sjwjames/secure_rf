extern crate random_forest_rust;
extern crate serde;

use std::time::SystemTime;
use std::{env, thread};
use random_forest_rust::ti::ti::{TI, initialize_ti_context, run_ti_module};
use random_forest_rust::computing_party::computing_party::initialize_party_context;
use random_forest_rust::random_forest::random_forest;
use num::BigUint;
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::{Write, Read};
use std::str;
use serde::{Serialize, Deserialize, Serializer};
use num::integer::*;


fn main() {
    run();
}

fn run(){
        let prefix = "main:      ";

    println!("{} runtime count starting...", &prefix);
    let now = SystemTime::now();
    let args: Vec<String> = env::args().collect();
    let settings_file = args[1].clone();

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name(&settings_file.as_str())).unwrap()
        .merge(config::Environment::with_prefix("APP")).unwrap();

    match settings.get_bool("ti") {
        Ok(is_ti) => {
            if is_ti {
                let mut ti_context = initialize_ti_context(settings_file.clone());
                run_ti_module(&mut ti_context);
            } else {
                let mut party_context = initialize_party_context(settings_file.clone());
                party_context.thread_hierarchy.push("RF".to_string());
                random_forest::train(&mut party_context);
            }
        }
        Err(error) => {
            panic!(
                format!("{} encountered a problem while parsing settings: {:?}", &prefix, error))
        }
    };
    println!("{} total runtime = {:9} (ms)", &prefix, now.elapsed().unwrap().as_millis());
}


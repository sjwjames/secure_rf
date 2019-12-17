extern crate random_forest_rust;
extern crate serde;

use std::time::{SystemTime, Duration};
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
use amiquip::{Connection, Exchange, Publish, QueueDeclareOptions, ConsumerOptions, ConsumerMessage};
use random_forest_rust::utils::utils::{push_message_to_queue, receive_message_from_queue};

fn test_mq() {
    let args: Vec<String> = env::args().collect();
    let settings_file = args[1].clone();

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name(&settings_file.as_str())).unwrap()
        .merge(config::Environment::with_prefix("APP")).unwrap();

    match settings.get_int("party_id") {
        Ok(party_id) => {
            if party_id == 1 {
                let address = "amqp://guest:guest@localhost:5672".to_string();
                let routing_key = "hello1".to_string();
                let message = "hello party 0, from party 1".to_string();
                push_message_to_queue(&address,&routing_key,&message);

                //subscribe
                let address = "amqp://guest:guest@localhost:5673".to_string();
                let routing_key = "hello1".to_string();
                let message_count = 10;
                receive_message_from_queue(&address,&routing_key,message_count);
            } else {
                //publish
                let address = "amqp://guest:guest@localhost:5673".to_string();
                let routing_key = "hello1".to_string();
                let message = "hello party 1,from party 0".to_string();
                push_message_to_queue(&address,&routing_key,&message);

                //subscribe
                let address = "amqp://guest:guest@localhost:5672".to_string();
                let routing_key = "hello1".to_string();
                let message_count = 10;
                receive_message_from_queue(&address,&routing_key,message_count);
            }
        }
        Err(error) => {
            panic!("{}",error);
        }
    };
}

fn main() {
    //test_mq();
    run();
}


fn run() {
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


extern crate random_forest_rust;
extern crate serde;

use std::time::SystemTime;
use std::{env, thread};
use random_forest_rust::ti::ti::{TI, initialize_ti_context, run_ti_module};
use random_forest_rust::computing_party::computing_party::initialize_party_context;
use random_forest_rust::random_forest::random_forest;
use num::BigUint;
use std::net::{TcpListener, TcpStream, SocketAddr};
use random_forest_rust::thread_pool::thread_pool::ThreadPool;
use std::io::{Write, Read};
use std::str;
use serde::{Serialize, Deserialize, Serializer};


fn main() {
//    let prefix = "main:      ";
//
//    println!("{} runtime count starting...", &prefix);
//    let now = SystemTime::now();
//    let args: Vec<String> = env::args().collect();
//    let settings_file = args[1].clone();
//
//    let mut settings = config::Config::default();
//    settings
//        .merge(config::File::with_name(&settings_file.as_str())).unwrap()
//        .merge(config::Environment::with_prefix("APP")).unwrap();
//
//    match settings.get_bool("ti") {
//        Ok(is_ti) => {
//            if is_ti {
//                let mut ti_context = initialize_ti_context(settings_file.clone());
//                run_ti_module(&mut ti_context);
//            } else {
//                let mut party_context = initialize_party_context(settings_file.clone());
//                random_forest::train(party_context);
//            }
//        }
//        Err(error) => {
//            panic!(
//                format!("{} encountered a problem while parsing settings: {:?}", &prefix, error))
//        }
//    };
//    println!("{} total runtime = {:9} (ms)", &prefix, now.elapsed().unwrap().as_millis());

    let thread0 = thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:5000")
            .expect("unable to establish Tcp Listener");

        let mut o_stream: TcpStream = try_conn("127.0.0.1:6000".parse().unwrap());
        let a = BigUint::new(vec![1u32; 1]);
        let b = BigUint::new(vec![5u32; 1]);
        let c = BigUint::new(vec![5u32; 1]);
        let d = BigUint::new(vec![5u32; 1]);
        let e = BigUint::new(vec![5u32; 1]);
        let f = BigUint::new(vec![5u32; 1]);

        let test_vec = vec![a, b,c,d,e,f];

        let mut max = 0;
        let mut total_length:u64 = 0;
        for x in test_vec.iter(){
            let len:u64 = x.to_bytes_le().len() as u64;
            if max < len {
                max = len;
            }
            total_length += len;
        }

        let mut serialized_vec=Vec::new();
        for x in test_vec.iter(){
            serialized_vec.push(serde_json::to_string(&(x.to_bytes_le())).unwrap());
        }
        o_stream.write(serialized_vec.join(";").as_bytes());

//        for item in test_vec {
//            let bytes_vec = item.to_bytes_le();
//            o_stream.write(&bytes_vec);
//        }
    });
    let thread1 = thread::spawn(move || {
        let listener = TcpListener::bind("127.0.0.1:6000")
            .expect("unable to establish Tcp Listener");
        let mut in_stream = match listener.accept() {
            Ok((stream, _addr)) => stream,
            Err(_) => panic!("server 0: failed to accept connection"),
        };



        let mut buf = String::new();
        in_stream.read_to_string(&mut buf);
        let mut buf:Vec<&str> = buf.split(";").collect();
        for item in buf.iter(){
            let deserialized:Vec<u8> = serde_json::from_str(&item).unwrap();
            let deserialized = BigUint::from_bytes_le(&deserialized);
            println!("{}",&deserialized);
        }

//        let len: usize = format!("{}", String::from_utf8_lossy(&buf[..])).parse().expect("parse failed");
//        println!("{}", len);
//
//
//        let mut buf = Vec::new();
//        for _ in 0..len {
//            in_stream.read(&mut buf).unwrap();
//            println!("{}", String::from_utf8_lossy(&buf[..]));
//        }
    });
    match thread0.join() {
        Ok(_) => println!("thread 0 s"),
        Err(_) => panic!("thread 0 f"),
    };//.expect("main: failed to rejoin server 0");

    match thread1.join() {
        Ok(_) => println!("thread 1 s"),
        Err(_) => panic!("thread 1 f"),
    };//.expect("main: failed to rejoin server 0");
}

fn try_conn(socket: SocketAddr) -> TcpStream {
    loop {
        match TcpStream::connect(socket) {
            Ok(stream) => return stream,
            Err(_) => println!("{} connection refused by {}", 1, 1),
        };
    }
}

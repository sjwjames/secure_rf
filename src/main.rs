extern crate random_forest_rust;
extern crate serde;

use std::time::{SystemTime, Duration};
use std::{env, thread};
use random_forest_rust::ti::ti::{TI, initialize_ti_context, run_ti_module};
use random_forest_rust::computing_party::computing_party::{initialize_party_context, ti_receive};
use random_forest_rust::random_forest::random_forest;
use num::{BigUint, Zero, FromPrimitive};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::io::{Write, Read, BufReader, BufRead};
use std::str;
use serde::{Serialize, Deserialize, Serializer};
use num::integer::*;
use amiquip::{Connection, Exchange, Publish, QueueDeclareOptions, ConsumerOptions, ConsumerMessage};
use random_forest_rust::utils::utils::{push_message_to_queue, receive_message_from_queue, Xbuffer, send_u64_messages, send_biguint_messages};
use threadpool::ThreadPool;
use random_forest_rust::multiplication::multiplication::multiplication_byte;
use random_forest_rust::protocol_test::protocol_test::{test_multi_byte, test_batch_multiplication_byte, test_batch_multiplication_integer, test_multiplication_bigint, test_multi_thread_batch_mul_byte, test_parallel_multiplication, test_batch_multiply_bigint, test_parallel_multiplication_big_integer, test_equality_big_integer, test_comparison, test_comparison_bigint, test_bit_decomposition, test_bit_decomposition_bigint, test_dot_product_bigint, test_or_xor, test_change_binary_to_decimal_field, test_argmax, test_or_xor_bigint, test_change_binary_to_bigint_field, test_dot_product_integer, test_batch_integer_equality};
use rand::ThreadRng;
use num::bigint::RandBigInt;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::str::FromStr;
use random_forest_rust::constants::constants::{U64S_PER_TX, U8S_PER_TX};
use std::num::Wrapping;

fn test_mq() {
    let args: Vec<String> = env::args().collect();
    let settings_file = args[1].clone();

    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name(&settings_file.as_str())).unwrap()
        .merge(config::Environment::with_prefix("APP")).unwrap();
    let thread_pool = ThreadPool::new(10);
    match settings.get_int("party_id") {
        Ok(party_id) => {
            if party_id == 1 {
                for i in 0..40 {
                    thread_pool.execute(move || {
                        let address = "amqp://guest:guest@localhost:5672".to_string();
                        let routing_key = format!("hello{}", i);
                        let message = format!("hello party 0, greetings from party 1, message {}", i);
                        push_message_to_queue(&address, &routing_key, &message);

                        //subscribe
                        let address = "amqp://guest:guest@localhost:5673".to_string();
                        let routing_key = format!("hello{}", i);
                        let message_count = 1;
                        receive_message_from_queue(&address, &routing_key, message_count);
                    });

//                    let address = "amqp://guest:guest@localhost:5672".to_string();
//                    let routing_key = format!("hello{}",i);
//                    let message = format!("hello party 0, greetings from party 1, message {}", i);
//                    push_message_to_queue(&address, &routing_key, &message);
//
//                    //subscribe
//                    let address = "amqp://guest:guest@localhost:5673".to_string();
//                    let routing_key = format!("hello{}",i);
//                    let message_count = 1;
//                    receive_message_from_queue(&address, &routing_key, message_count);
                }
            } else {
                //publish
                for i in 0..40 {
                    thread_pool.execute(move || {
                        let address = "amqp://guest:guest@localhost:5673".to_string();
                        let routing_key = format!("hello{}", i);
                        let message = format!("hello party 1, greetings from party 0, message {}", i);
                        push_message_to_queue(&address, &routing_key, &message);

                        //subscribe
                        let address = "amqp://guest:guest@localhost:5672".to_string();
                        let routing_key = format!("hello{}", i);
                        let message_count = 1;
                        receive_message_from_queue(&address, &routing_key, message_count);
                    });

//                    let address = "amqp://guest:guest@localhost:5673".to_string();
//                    let routing_key = format!("hello{}",i);
//                    let message = format!("hello party 1, greetings from party 0, message {}", i);
//                    push_message_to_queue(&address, &routing_key, &message);
//
//                    //subscribe
//                    let address = "amqp://guest:guest@localhost:5672".to_string();
//                    let routing_key = format!("hello{}",i);
//                    let message_count = 1;
//                    receive_message_from_queue(&address, &routing_key, message_count);
                }
            }
        }
        Err(error) => {
            panic!("{}", error);
        }
    };

    thread_pool.join();
}

fn test_protocols() {
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
                let dt_shares = ti_receive(
                    party_context.ti_stream.try_clone().expect("failed to clone ti recvr"), &mut party_context);
                party_context.dt_shares = dt_shares;
                party_context.raw_tcp_communication = true;

//                println!("current binary share index:{}",party_context.dt_shares.sequential_binary_index);
//                println!("current additive share index:{}",party_context.dt_shares.sequential_additive_index);
//                test_multi_byte(&mut party_context);
//                test_batch_multiplication_byte(&mut party_context);
//                test_batch_multiplication_integer(&mut party_context);
//                test_multiplication_bigint(&mut party_context);
//                test_multi_thread_batch_mul_byte(&mut party_context);
//                test_parallel_multiplication(&mut party_context);
//                test_batch_multiply_bigint(&mut party_context);
//                test_parallel_multiplication_big_integer(&mut party_context);
//                test_equality_big_integer(&mut party_context);
//                test_comparison(&mut party_context);
//                test_bit_decomposition(&mut party_context);
//                test_bit_decomposition_bigint(&mut party_context);
//                test_comparison_bigint(&mut party_context);
//                test_dot_product_bigint(&mut party_context);
//                test_or_xor(&mut party_context);
//                test_or_xor_bigint(&mut party_context);
//                test_change_binary_to_decimal_field(&mut party_context);
//                test_change_binary_to_bigint_field(&mut party_context);
//                test_argmax(&mut party_context);
//                test_dot_product_integer(&mut party_context);
//                test_batch_integer_equality(&mut party_context);
                let mut b = Vec::new();
                for i in 1..1000 {
                    b.push(BigUint::from_i32(i).unwrap());
                }
//                let result = send_u64_messages(&mut party_context,&b);
                let result = send_biguint_messages(&mut party_context,&b);
                println!("{:?}",result);
                let a = BigUint::from_str("10000").unwrap();
                println!("{:?}",a);
            }
        }
        Err(error) => {
            panic!(
                format!("{} encountered a problem while parsing settings: {:?}", &prefix, error))
        }
    };
}

fn main() {
//    test_mq();
    run();
//    test_protocols();
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


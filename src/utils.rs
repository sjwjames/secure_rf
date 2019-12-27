pub mod utils {
    use num::bigint::{BigUint, ToBigUint, ToBigInt, RandBigInt};
    use num::integer::*;
    use std::ops::Sub;
    use std::num::Wrapping;
    use crate::computing_party::computing_party::ComputingParty;
    use std::sync::{Mutex, Arc};
    use amiquip::{Connection, Exchange, Publish, Channel, QueueDeclareOptions, ConsumerOptions, ConsumerMessage, ExchangeDeclareOptions, ExchangeType, FieldTable};
    use std::collections::HashMap;
    use std::process::exit;
    use std::{thread, time};
    use std::time::Duration;


    pub fn big_uint_subtract(x: &BigUint, y: &BigUint, big_int_prime: &BigUint) -> BigUint {
        let result = x.to_bigint().unwrap().sub(y.to_bigint().unwrap()).mod_floor(&(big_int_prime.to_bigint().unwrap())).to_biguint().unwrap();
        result
    }

    pub fn big_uint_clone(x: &BigUint) -> BigUint {
        let mut result = BigUint::from_bytes_le(&(x.to_bytes_le().clone()));
        result
    }

    pub fn big_uint_vec_clone(list: &Vec<BigUint>) -> Vec<BigUint> {
        let mut result = Vec::new();
        for item in list.iter() {
            result.push(big_uint_clone(item));
        }
        result
    }

    pub fn truncate_local(x: Wrapping<u64>,
                          decimal_precision: u32,
                          asymmetric_bit: u8) -> Wrapping<u64> {
        if asymmetric_bit == 0 {
            return -Wrapping((-x).0 >> decimal_precision);
        }

        Wrapping(x.0 >> decimal_precision)
    }

    pub enum ShareType {
        AdditiveShare,
        AdditiveBigIntShare,
        BinaryShare,
        EqualityShare,
    }

    pub fn serialize_biguint_vec(biguint_vec: &Vec<BigUint>) -> String {
        let mut str_vec = Vec::new();
        for item in biguint_vec.iter() {
            str_vec.push(serialize_biguint(item));
        }
        str_vec.join(";")
    }

    pub fn serialize_biguint_triple_vec(biguint_triple_vec: Vec<(BigUint, BigUint, BigUint)>) -> String {
        let mut str_vec: Vec<String> = Vec::new();
        for item in biguint_triple_vec.iter() {
            let mut tuple_vec = Vec::new();
            tuple_vec.push(serialize_biguint(&item.0));
            tuple_vec.push(serialize_biguint(&item.1));
            tuple_vec.push(serialize_biguint(&item.2));
            str_vec.push(tuple_vec.join("&"));
        }
        str_vec.join(";")
    }


    pub fn serialize_biguint(num: &BigUint) -> String {
        serde_json::to_string(&(num.to_bytes_le())).unwrap()
    }

    pub fn deserialize_biguint(message: &str) -> BigUint {
        BigUint::from_bytes_le(message.as_bytes())
    }

    pub fn deserialize_biguint_vec(message: String) -> Vec<BigUint> {
        let mut result = Vec::new();
        let str_vec: Vec<&str> = message.split(";").collect();
        for item in str_vec {
            result.push(deserialize_biguint(item));
        }
        result
    }


//    pub fn increment_current_share_index(ctx: &mut ComputingParty, share_type: ShareType) {
//        match share_type {
//            ShareType::AdditiveShare => {
//                let mut count = ctx.dt_shares.current_additive_index.lock().unwrap();
//                *count += 1;
//            }
//            ShareType::AdditiveBigIntShare => {
//                let mut count = ctx.dt_shares.current_additive_bigint_index.lock().unwrap();
//                *count += 1;
//            }
//            ShareType::BinaryShare => {
//                let mut count = ctx.dt_shares.current_binary_index.lock().unwrap();
//                *count += 1;
//            }
//            ShareType::EqualityShare => {
//                let mut count = ctx.dt_shares.current_equality_index.lock().unwrap();
//                *count += 1;
//            }
//        }
//    }

    pub fn increment_current_share_index(index: Arc<Mutex<usize>>) {
        let mut count = index.lock().unwrap();
        *count += 1;
    }


    pub fn get_current_bigint_share(ctx: &ComputingParty) -> &(BigUint, BigUint, BigUint) {
        let bigint_shares = &ctx.dt_shares.additive_bigint_triples;
        let current_index = *(ctx.dt_shares.current_additive_bigint_index.lock().unwrap());
        let result = &bigint_shares[current_index];
        increment_current_share_index(Arc::clone(&ctx.dt_shares.current_additive_bigint_index));
        result
    }

    pub fn get_current_equality_share(ctx: &ComputingParty) -> &BigUint {
        let shares = &ctx.dt_shares.equality_shares;
        let current_index = *(ctx.dt_shares.current_equality_index.lock().unwrap());
        let result = &shares[current_index];
        increment_current_share_index(Arc::clone(&ctx.dt_shares.current_equality_index));
        result
    }

    pub fn get_current_additive_share(ctx: &ComputingParty) -> &(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>) {
        let shares = &ctx.dt_shares.additive_triples;
        let current_index = *(ctx.dt_shares.current_additive_index.lock().unwrap());
        let result = &shares[current_index];
        increment_current_share_index(Arc::clone(&ctx.dt_shares.current_additive_index));
        result
    }

    pub fn get_current_binary_share(ctx: &ComputingParty) -> &(u8, u8, u8) {
        let shares = &ctx.dt_shares.binary_triples;
        let current_index = *(ctx.dt_shares.current_binary_index.lock().unwrap());
        let result = &shares[current_index];
//        increment_current_share_index(Arc::clone(&ctx.dt_shares.current_binary_index));
        result
    }

    pub fn receive_message_from_queue(address: &String, routing_key: &String, message_count: usize) -> Vec<String> {
//        thread::sleep(Duration::from_millis(10));
        let mut connection = Connection::insecure_open(address).unwrap();
        let channel = connection.open_channel(None).unwrap();
//        let exchange = channel.exchange_declare(ExchangeType::Direct,"direct",ExchangeDeclareOptions::default()).unwrap();
//        let queue = channel.queue_declare(routing_key, QueueDeclareOptions{
//            durable: false,
//            exclusive: false,
//            auto_delete: true,
//            arguments: FieldTable::default()
//        }).unwrap();
//        queue.bind(&exchange,routing_key,FieldTable::default());
        let queue = channel.queue_declare(routing_key, QueueDeclareOptions::default()).unwrap();

        let consumer = queue.consume(ConsumerOptions::default()).unwrap();
        let mut count = 0;
        let mut result = Vec::new();
        for (i, message) in consumer.receiver().iter().enumerate() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let body = String::from_utf8_lossy(&delivery.body).to_string();
                    print!("queue:{} ",routing_key);
                    println!("({:>3}) Received [{}]", i, body);
                    result.push(body.clone());
                    consumer.ack(delivery).unwrap();
                }
                other => {
                    println!("Consumer ended: {:?}", other);
                    break;
                }
            }
            count += 1;
            if message_count == count {
                break;
            }
        }
        connection.close();
        result
    }

    pub fn push_message_to_queue(address: &String, routing_key: &String, message: &String) {
        let mut connection = Connection::insecure_open(address).unwrap();
        let channel = connection.open_channel(None).unwrap();
        let exchange = channel.exchange_declare(ExchangeType::Direct,"direct",ExchangeDeclareOptions::default()).unwrap();
        let queue = channel.queue_declare(routing_key, QueueDeclareOptions::default()).unwrap();
        queue.bind(&exchange,routing_key,FieldTable::default());
        exchange.publish(Publish::new(message.as_bytes(), routing_key)).unwrap();
        connection.close();
    }
}
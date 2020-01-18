pub mod utils {
    use num::bigint::{BigUint, ToBigUint, ToBigInt, RandBigInt};
    use num::integer::*;
    use std::ops::{Sub, Add};
    use std::num::Wrapping;
    use crate::computing_party::computing_party::ComputingParty;
    use std::sync::{Mutex, Arc};
    use amiquip::{Connection, Exchange, Publish, Channel, QueueDeclareOptions, ConsumerOptions, ConsumerMessage, ExchangeDeclareOptions, ExchangeType, FieldTable};
    use std::collections::HashMap;
    use std::process::exit;
    use std::{thread, time};
    use std::time::Duration;
    use std::str::FromStr;
    use std::net::TcpStream;
    use std::io::Write;


    pub fn big_uint_subtract(x: &BigUint, y: &BigUint, big_int_prime: &BigUint) -> BigUint {
        let result = x.to_bigint().unwrap().sub(y.to_bigint().unwrap()).mod_floor(&(big_int_prime.to_bigint().unwrap())).to_biguint().unwrap();
        result
    }

    pub fn big_uint_clone(x: &BigUint) -> BigUint {
        let mut result = BigUint::from_bytes_le(&(x.to_bytes_le().clone()));
        result
    }

    pub fn big_uint_triple_clone(x:&(BigUint,BigUint,BigUint))->(BigUint,BigUint,BigUint){
        let mut result = (big_uint_clone(&x.0),big_uint_clone(&x.1),big_uint_clone(&x.2));
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

    pub fn serialize_biguint_double_vec(biguint_tuple: &Vec<(BigUint, BigUint)>) -> String {
        let mut str_vec: Vec<String> = Vec::new();
        for item in biguint_tuple.iter() {
            let mut tuple_vec = Vec::new();
            tuple_vec.push(serialize_biguint(&item.0));
            tuple_vec.push(serialize_biguint(&item.1));
            str_vec.push(tuple_vec.join("&"));
        }
        str_vec.join(";")
    }

    pub fn serialize_biguint_triple_vec(biguint_triple_vec: &Vec<(BigUint, BigUint, BigUint)>) -> String {
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
        num.to_string().clone()
    }

    pub fn deserialize_biguint(message: &str) -> BigUint {
        BigUint::from_str(&message).unwrap()
    }

    pub fn deserialize_biguint_vec(message: &str) -> Vec<BigUint> {
        let message = message.trim_end();
        let mut result = Vec::new();
        let str_vec: Vec<&str> = message.split(";").collect();
        for item in str_vec {
            result.push(deserialize_biguint(item));
        }
        result
    }

    pub fn deserialize_biguint_double_vec(message: &str) -> Vec<(BigUint,BigUint)> {
        let message = message.trim_end();
        let mut result = Vec::new();
        let str_vec: Vec<&str> = message.split(";").collect();
        for item in str_vec {
            let tuple_items: Vec<&str> = item.split("&").collect();
            result.push((deserialize_biguint(tuple_items[0]),deserialize_biguint(tuple_items[1])));
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


    pub fn get_current_bigint_share(ctx: &mut ComputingParty) -> (BigUint, BigUint, BigUint) {
        let bigint_shares = &ctx.dt_shares.additive_bigint_triples;
        if ctx.raw_tcp_communication{
            let current_index = ctx.dt_shares.sequential_additive_bigint_index;
            ctx.dt_shares.sequential_additive_bigint_index+=1;
            let current_bigint_share = &bigint_shares[current_index];
            return big_uint_triple_clone(current_bigint_share);
        }else{
            let current_index = *(ctx.dt_shares.current_additive_bigint_index.lock().unwrap());
            let result = big_uint_triple_clone(&bigint_shares[current_index]);
            increment_current_share_index(Arc::clone(&ctx.dt_shares.current_additive_bigint_index));
            return result;
        }
    }

    pub fn get_current_equality_share(ctx: &mut ComputingParty) -> BigUint {
        let shares = &ctx.dt_shares.equality_shares;
        let current_index = *(ctx.dt_shares.current_equality_index.lock().unwrap());
        let result = big_uint_clone(&shares[current_index]);
        increment_current_share_index(Arc::clone(&ctx.dt_shares.current_equality_index));
        result
    }

    pub fn get_current_additive_share(ctx: &mut ComputingParty) -> (Wrapping<u64>, Wrapping<u64>, Wrapping<u64>) {
        let shares = &ctx.dt_shares.additive_triples;
        if ctx.raw_tcp_communication{
            let current_index = ctx.dt_shares.sequential_additive_index;
            ctx.dt_shares.sequential_additive_index+=1;
            return shares[current_index];
        }else{
            let current_index = *(ctx.dt_shares.current_additive_index.lock().unwrap());
            let result = shares[current_index];
            increment_current_share_index(Arc::clone(&ctx.dt_shares.current_additive_index));
            return result;
        }
    }

    pub fn get_current_binary_share(ctx: &mut ComputingParty) -> (u8, u8, u8) {
        let shares = &ctx.dt_shares.binary_triples;
        if ctx.raw_tcp_communication{
            let current_index = ctx.dt_shares.sequential_binary_index;
            ctx.dt_shares.sequential_binary_index+=1;
            return shares[current_index];
        }else{
            let current_index = *(ctx.dt_shares.current_binary_index.lock().unwrap());
            let result = shares[current_index];
            increment_current_share_index(Arc::clone(&ctx.dt_shares.current_binary_index));
            return result;
        }
    }

    pub fn get_binary_shares(ctx: &mut ComputingParty,range:usize)->Vec<(u8, u8, u8)>{
        let current_index = ctx.dt_shares.sequential_binary_index;
        let shares = ctx.dt_shares.binary_triples[current_index..current_index+range].to_vec();

        ctx.dt_shares.sequential_binary_index+=range;
        return shares;
    }

    pub fn get_additive_shares(ctx: &mut ComputingParty,range:usize)->Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)>{
        let current_index = ctx.dt_shares.sequential_additive_index;
        let shares = ctx.dt_shares.additive_triples[current_index..current_index+range].to_vec();
        ctx.dt_shares.sequential_binary_index+=range;
        return shares;
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
        let queue = channel.queue_declare(routing_key, QueueDeclareOptions {
            durable: false,
            exclusive: false,
            auto_delete: false,
            arguments: Default::default(),
        }).unwrap();
//        let queue = channel.queue_declare(routing_key,QueueDeclareOptions::default()).unwrap();
        let consumer = queue.consume(ConsumerOptions::default()).unwrap();
        let mut count = 0;
        let mut result = Vec::new();
        for (i, message) in consumer.receiver().iter().enumerate() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let body = String::from_utf8_lossy(&delivery.body).to_string();
//                    print!("queue:{} ", routing_key);
//                    println!("({:>3}) Received [{}]", i, body);
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
        let exchange = channel.exchange_declare(ExchangeType::Direct, "direct", ExchangeDeclareOptions::default()).unwrap();
        let queue = channel.queue_declare(routing_key, QueueDeclareOptions {
            durable: false,
            exclusive: false,
            auto_delete: false,
            arguments: Default::default(),
        }).unwrap();
//        let queue = channel.queue_declare(routing_key,QueueDeclareOptions::default()).unwrap();
        queue.bind(&exchange, routing_key, FieldTable::default());
        exchange.publish(Publish::new(message.as_bytes(), routing_key)).unwrap();
        connection.close();
    }


    pub fn reveal_byte_result(x: u8, ctx: &mut ComputingParty) -> u8 {
        let message_id = "reveal".to_string();
        let message_content = serde_json::to_string(&x).unwrap();
//        println!("party{} sending {}", ctx.party_id, x);
        push_message_to_queue(&ctx.remote_mq_address, &message_id, &message_content);
        let message_received = receive_message_from_queue(&ctx.local_mq_address, &message_id, 1);
        let mut message_rec: u8 = message_received[0].parse().unwrap();
        x ^ message_rec
    }

    pub fn reveal_byte_vec_result(x: &Vec<u8>, ctx: &mut ComputingParty) -> Vec<u8> {
        let message_id = "reveal".to_string();
        let message_content = serde_json::to_string(x).unwrap();
        push_message_to_queue(&ctx.remote_mq_address, &message_id, &message_content);
        let message_received = receive_message_from_queue(&ctx.local_mq_address, &message_id, 1);
        let mut message_rec: Vec<u8> = serde_json::from_str(&message_received[0]).unwrap();
        let mut result = Vec::new();
        for i in 0..x.len() {
            result.push(x[i] ^ message_rec[i]);
        }
        result
    }

    pub fn reveal_int_vec_result(x: &Vec<Wrapping<u64>>, ctx: &mut ComputingParty) -> Vec<u64> {
        let message_id = "reveal".to_string();
        let message_content = serde_json::to_string(x).unwrap();
        push_message_to_queue(&ctx.remote_mq_address, &message_id, &message_content);
        let message_received = receive_message_from_queue(&ctx.local_mq_address, &message_id, 1);
        let mut message_rec: Vec<Wrapping<u64>> = serde_json::from_str(&message_received[0]).unwrap();
        let mut result = Vec::new();
        for i in 0..x.len() {
            result.push((x[i].0 + message_rec[i].0).mod_floor(&ctx.dt_training.dataset_size_prime));
        }
        result
    }

    pub fn reveal_bigint_result(x: &BigUint, ctx: &mut ComputingParty) -> BigUint {
        let message_id = "reveal".to_string();
        let message_content = serialize_biguint(x);
        push_message_to_queue(&ctx.remote_mq_address, &message_id, &message_content);
        let message_received = receive_message_from_queue(&ctx.local_mq_address, &message_id, 1);
        let mut message_rec: BigUint = deserialize_biguint(&message_received[0]);
        x.add(&message_rec).mod_floor(&ctx.dt_training.big_int_prime)
    }

    pub fn reveal_bigint_vec_result(x: &Vec<BigUint>, ctx: &mut ComputingParty) -> Vec<BigUint> {
        let message_id = "reveal".to_string();
        let message_content = serialize_biguint_vec(x);
        push_message_to_queue(&ctx.remote_mq_address, &message_id, &message_content);
        let message_received = receive_message_from_queue(&ctx.local_mq_address, &message_id, 1);
        let mut message_rec: Vec<BigUint> = deserialize_biguint_vec(&message_received[0].as_str());
        let mut result = Vec::new();
        for i in 0..x.len() {
            result.push((&x[i]).add(&message_rec[i]).mod_floor(&ctx.dt_training.big_int_prime));
        }
        result
    }
}
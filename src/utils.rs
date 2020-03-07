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
    use std::io::{Write, Read, BufReader, BufRead};
    use std::convert::TryFrom;
    use crate::constants::constants::{U64S_PER_TX, U8S_PER_TX, TYPE_U8, TYPE_U64, TYPE_BIGINT, U64S_PER_MINI_TX, U8S_PER_MINI_TX};
    use num::ToPrimitive;

    pub union Xbuffer {
        pub u64_buf: [u64; U64S_PER_TX],
        pub u8_buf: [u8; U8S_PER_TX],
    }

    pub union XbufferMini {
        pub u64_buf: [u64; U64S_PER_MINI_TX],
        pub u8_buf: [u8; U8S_PER_MINI_TX],
    }

    pub fn big_uint_subtract(x: &BigUint, y: &BigUint, big_int_prime: &BigUint) -> BigUint {
        let result = x.to_bigint().unwrap().sub(y.to_bigint().unwrap()).mod_floor(&(big_int_prime.to_bigint().unwrap())).to_biguint().unwrap();
        result
    }

    pub fn big_uint_clone(x: &BigUint) -> BigUint {
        let mut result = BigUint::from_bytes_le(&(x.to_bytes_le().clone()));
        result
    }

    pub fn big_uint_triple_clone(x: &(BigUint, BigUint, BigUint)) -> (BigUint, BigUint, BigUint) {
        let mut result = (big_uint_clone(&x.0), big_uint_clone(&x.1), big_uint_clone(&x.2));
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
        let message = message.trim_end();
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

    pub fn deserialize_biguint_double_vec(message: &str) -> Vec<(BigUint, BigUint)> {
        let message = message.trim_end();
        let mut result = Vec::new();
        let str_vec: Vec<&str> = message.split(";").collect();
        for item in str_vec {
            let tuple_items: Vec<&str> = item.split("&").collect();
            result.push((deserialize_biguint(tuple_items[0]), deserialize_biguint(tuple_items[1])));
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
        if ctx.raw_tcp_communication {
            let current_index = ctx.dt_shares.sequential_additive_bigint_index;
//            let current_bigint_share = &bigint_shares[current_index];
            let current_bigint_share = &bigint_shares[0];
            ctx.dt_shares.sequential_additive_bigint_index += 1;
            return big_uint_triple_clone(current_bigint_share);
        } else {
            let current_index = *(ctx.dt_shares.current_additive_bigint_index.lock().unwrap());
            let result = big_uint_triple_clone(&bigint_shares[current_index]);
            increment_current_share_index(Arc::clone(&ctx.dt_shares.current_additive_bigint_index));
            return result;
        }
    }

    pub fn get_current_equality_share(ctx: &mut ComputingParty) -> BigUint {
        let shares = &ctx.dt_shares.equality_shares;
        if ctx.raw_tcp_communication {
            let current_index = ctx.dt_shares.sequential_equality_index;
//            let current_share = &shares[current_index];
            let current_share = &shares[0];
            ctx.dt_shares.sequential_additive_bigint_index += 1;

            return big_uint_clone(&current_share);
        } else {
            let current_index = *(ctx.dt_shares.current_equality_index.lock().unwrap());
            let result = big_uint_clone(&shares[current_index]);
            increment_current_share_index(Arc::clone(&ctx.dt_shares.current_equality_index));
            return result;
        }
    }

    pub fn get_current_equality_integer_shares(ctx: &mut ComputingParty, range: usize, prime: u64) -> Vec<Wrapping<u64>> {
        let current_index = ctx.dt_shares.sequential_equality_integer_index.get(&prime).unwrap();
        let shares = ctx.dt_shares.equality_integer_shares.get(&prime).unwrap()[*current_index..(current_index + range)].to_vec();
//        let share = ctx.dt_shares.equality_integer_shares.get(&prime).unwrap()[0];
//        let shares = vec![share; range];
        ctx.dt_shares.sequential_equality_integer_index.insert(prime, current_index + range);
        return shares;
    }

    pub fn get_current_additive_share(ctx: &mut ComputingParty, prime: u64) -> (Wrapping<u64>, Wrapping<u64>, Wrapping<u64>) {
        let shares = ctx.dt_shares.additive_triples.get(&prime).unwrap();
        if ctx.raw_tcp_communication {
            let current_index = ctx.dt_shares.sequential_additive_index.get(&prime).unwrap();
            ctx.dt_shares.sequential_additive_index.insert(prime, current_index + 1);
//            let share = shares[*current_index];
            let share = shares[0];
            return share;
        } else {
            let current_index = *(ctx.dt_shares.current_additive_index.lock().unwrap());
            let result = shares[current_index];
            increment_current_share_index(Arc::clone(&ctx.dt_shares.current_additive_index));
            return result;
        }
    }

    pub fn get_current_binary_share(ctx: &mut ComputingParty) -> (u8, u8, u8) {
        let shares = &ctx.dt_shares.binary_triples;
        if ctx.raw_tcp_communication {
            let current_index = ctx.dt_shares.sequential_binary_index;
            ctx.dt_shares.sequential_binary_index += 1;
//            let share = shares[current_index];
            let share = shares[0];
            return share;
        } else {
            let current_index = *(ctx.dt_shares.current_binary_index.lock().unwrap());
            let result = shares[current_index];
            increment_current_share_index(Arc::clone(&ctx.dt_shares.current_binary_index));
            return result;
        }
    }

    pub fn get_binary_shares(ctx: &mut ComputingParty, range: usize) -> Vec<(u8, u8, u8)> {
        let current_index = ctx.dt_shares.sequential_binary_index;
//        let shares = ctx.dt_shares.binary_triples[current_index..current_index + range].to_vec();
        let share = ctx.dt_shares.binary_triples[0];
        let shares = vec![share; range];
        ctx.dt_shares.sequential_binary_index += range;
        return shares;
    }

    pub fn get_additive_shares(ctx: &mut ComputingParty, range: usize, prime: u64) -> Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)> {
        let current_index = ctx.dt_shares.sequential_additive_index.get(&prime).unwrap();
//        let shares = ctx.dt_shares.additive_triples.get(&prime).unwrap()[*current_index..current_index + range].to_vec();
        let share = ctx.dt_shares.additive_triples.get(&prime).unwrap()[0];
        let shares = vec![share; range];
        ctx.dt_shares.sequential_additive_index.insert(prime, current_index + range);
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
                    print!("queue:{} ", routing_key);
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

    pub fn reveal_int_result(x: &Wrapping<u64>, ctx: &mut ComputingParty) -> u64 {
        let message_id = "reveal".to_string();
        let message_content = serde_json::to_string(x).unwrap();
        push_message_to_queue(&ctx.remote_mq_address, &message_id, &message_content);
        let message_received = receive_message_from_queue(&ctx.local_mq_address, &message_id, 1);
        let mut message_rec: Wrapping<u64> = serde_json::from_str(&message_received[0]).unwrap();
        let mut result = (x.0 + message_rec.0).mod_floor(&ctx.dt_training.dataset_size_prime);
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

    fn send_batch_message(ctx: &ComputingParty, data: &Vec<u8>) -> Xbuffer {
        let mut o_stream = ctx.o_stream.try_clone()
            .expect("failed cloning tcp o_stream");
        let mut in_stream = ctx.in_stream.try_clone().expect("failed cloning tcp o_stream");
        let mut recv_buf = Xbuffer { u8_buf: [0u8; U8S_PER_TX] };
        if ctx.asymmetric_bit == 1 {
            let mut bytes_written = 0;
            while bytes_written < U8S_PER_TX {
                let current_bytes = unsafe {
                    o_stream.write(&data[bytes_written..])
                };
                bytes_written += current_bytes.unwrap();
            }

            unsafe {
                let mut bytes_read = 0;
                while bytes_read < recv_buf.u8_buf.len() {
                    let current_bytes = in_stream.read(&mut recv_buf.u8_buf[bytes_read..]).unwrap();
                    bytes_read += current_bytes;
                }
            }
        } else {
            unsafe {
                let mut bytes_read = 0;
                while bytes_read < recv_buf.u8_buf.len() {
                    let current_bytes = in_stream.read(&mut recv_buf.u8_buf[bytes_read..]).unwrap();
                    bytes_read += current_bytes;
                }
            }

            let mut bytes_written = 0;
            while bytes_written < U8S_PER_TX {
                let current_bytes = unsafe {
                    o_stream.write(&data[bytes_written..])
                };
                bytes_written += current_bytes.unwrap();
            }
        }
        recv_buf
    }

    pub fn send_u8_messages(ctx: &ComputingParty, data: &Vec<u8>) -> Vec<u8> {
        let mut batches: usize = 0;
        let mut data_len = data.len();
        let mut result: Vec<u8> = Vec::new();
        let mut current_batch = 0;
        let mut push_buf = Xbuffer { u8_buf: [0u8; U8S_PER_TX] };
        batches = (data_len as f64 / U8S_PER_TX as f64).ceil() as usize;
        while current_batch < batches {
            for i in 0..U8S_PER_TX {
                unsafe {
                    if current_batch * U8S_PER_TX + i < data_len {
                        push_buf.u8_buf[i] = data[current_batch * U8S_PER_TX + i];
                    } else {
                        break;
                    }
                }
            }
            unsafe {
                let buf_vec = push_buf.u8_buf.to_vec();
                let mut part_result = send_batch_message(ctx, &buf_vec);
                result.append(&mut (part_result.u8_buf.to_vec()));
            }

            current_batch += 1;
        }
        result[0..data_len].to_vec()
    }

    pub fn send_u64_messages(ctx: &ComputingParty, data: &Vec<Wrapping<u64>>) -> Vec<Wrapping<u64>> {
        let mut batches: usize = 0;
        let mut data_len = data.len();
        let mut result: Vec<Wrapping<u64>> = Vec::new();
        let mut current_batch = 0;
        let mut push_buf = Xbuffer { u64_buf: [0u64; U64S_PER_TX] };
        batches = (data_len as f64 / U64S_PER_TX as f64).ceil() as usize;
        while current_batch < batches {
            for i in 0..U64S_PER_TX {
                unsafe {
                    if current_batch * U64S_PER_TX + i < data_len {
                        push_buf.u64_buf[i] = data[current_batch * U64S_PER_TX + i].0;
                    } else {
                        break;
                    }
                }
            }
            unsafe {
                let buf_vec = push_buf.u8_buf.to_vec();
                let mut part_result = send_batch_message(ctx, &buf_vec);
                for item in part_result.u64_buf.to_vec() {
                    result.push(Wrapping(item));
                }
            }

            current_batch += 1;
        }
        result[0..data_len].to_vec()
    }


    pub fn send_biguint_messages(ctx: &ComputingParty, data: &Vec<BigUint>) -> Vec<BigUint> {
//        let mut batches: usize = 0;
//        let mut data_len = data.len();
//        let mut result: Vec<BigUint> = Vec::new();
//        let mut current_batch = 0;
//        let mut push_buf = Xbuffer { u8_buf: [0u8; U8S_PER_TX] };
//        let mut data_transformed = Vec::new();
//        for i in 0..data_len {
//            let mut bytes = (data[i].to_string() + ";").as_bytes().to_vec();
//            data_transformed.append(&mut bytes);
//        }
//        let temp_result = send_u8_messages(ctx, &data_transformed);
//
//        let mut iter = temp_result.split(|num| *num == 59);
//        for item in iter {
//
//            let bigint_str = String::from_utf8(item.to_vec()).unwrap();
//            result.push(BigUint::from_str(&bigint_str).unwrap());
//            if result.len() == data.len() {
//                break;
//            }
//        }
        let mut o_stream = ctx.o_stream.try_clone()
            .expect("failed cloning tcp o_stream");
        let mut in_stream = ctx.in_stream.try_clone().expect("failed cloning tcp o_stream");
        let mut reader = BufReader::new(in_stream);
        let mut share_message = String::new();
        let mut result: Vec<BigUint> = Vec::new();
        share_message = String::new();
        if ctx.asymmetric_bit == 1 {
            o_stream.write(format!("{}\n", serialize_biguint_vec(data)).as_bytes());
            reader.read_line(&mut share_message).expect("fail to read share message str");
            result = deserialize_biguint_vec(&share_message);
        } else {
            reader.read_line(&mut share_message).expect("fail to read share message str");
            result = deserialize_biguint_vec(&share_message);
            o_stream.write(format!("{}\n", serialize_biguint_vec(data)).as_bytes());
        }

        result
    }

    pub fn send_receive_u64_matrix(matrix_sent: &Vec<Vec<Wrapping<u64>>>, ctx: &ComputingParty) -> Vec<Vec<Wrapping<u64>>> {
        let mut list_sent = Vec::new();
        let mut matrix_received = Vec::new();
        for row in matrix_sent {
            for item in row {
                list_sent.push(item.clone());
            }
        }
        let list_received = send_u64_messages(ctx, &list_sent);
        let row_len = matrix_sent[0].len();
        let matrix_len = matrix_sent.len();
        for i in 0..matrix_len {
            let mut row = Vec::new();
            for j in 0..row_len {
                row.push(list_received[i * row_len + j]);
            }
            matrix_received.push(row);
        }
        matrix_received
    }

    pub fn send_receive_u8_matrix(matrix_sent: &Vec<Vec<u8>>, ctx: &ComputingParty) -> Vec<Vec<u8>> {
        let mut list_sent = Vec::new();
        let mut matrix_received = Vec::new();
        for row in matrix_sent {
            for item in row {
                list_sent.push(item.clone());
            }
        }
        let list_received = send_u8_messages(ctx, &list_sent);
        let row_len = matrix_sent[0].len();
        let matrix_len = matrix_sent.len();
        for i in 0..matrix_len {
            let mut row = Vec::new();
            for j in 0..row_len {
                row.push(list_received[i * row_len + j]);
            }
            matrix_received.push(row);
        }
        matrix_received
    }


    pub fn receive_u64_shares(stream: &mut TcpStream, amount: u64) -> Vec<Wrapping<u64>> {
        let mut recv_buf = Xbuffer { u64_buf: [0u64; U64S_PER_TX] };
        let mut result = Vec::new();
        let mut batches: usize = (amount as f64 / U64S_PER_TX as f64).ceil() as usize;
        let mut current_batch = 0;
        while current_batch < batches {
            unsafe {
                let mut bytes_read = 0;
                while bytes_read < recv_buf.u8_buf.len() {
                    let current_bytes = stream.read(&mut recv_buf.u8_buf[bytes_read..]).unwrap();
                    bytes_read += current_bytes;
                }

                for i in 0..U64S_PER_TX {
                    result.push(Wrapping(recv_buf.u64_buf[i]));
                }
            }
            current_batch += 1;
        }
        result[0..amount as usize].to_vec()
    }

    pub fn receive_u64_triple_shares(stream: &mut TcpStream, amount: u64) -> Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)> {
        let mut recv_buf = Xbuffer { u64_buf: [0u64; U64S_PER_TX] };
        let mut result_temp = Vec::new();
        let mut batches: usize = ((amount * 3) as f64 / U64S_PER_TX as f64).ceil() as usize;
        let mut current_batch = 0;
        while current_batch < batches {
            unsafe {
                let mut bytes_read = 0;
                while bytes_read < recv_buf.u8_buf.len() {
                    let current_bytes = stream.read(&mut recv_buf.u8_buf[bytes_read..]).unwrap();
                    bytes_read += current_bytes;
                }

                for i in 0..U64S_PER_TX {
                    result_temp.push(Wrapping(recv_buf.u64_buf[i]));
                }
            }
            current_batch += 1;
        }
        let mut result = Vec::new();
        for i in 0..amount as usize {
            result.push((result_temp[i * 3], result_temp[i * 3 + 1], result_temp[i * 3 + 2]));
        }
        result
    }

    pub fn receive_u8_triple_shares(stream: &mut TcpStream, amount: u64) -> Vec<(u8, u8, u8)> {
        let mut recv_buf = Xbuffer { u8_buf: [0u8; U8S_PER_TX] };
        let mut result_temp = Vec::new();
        let mut batches: usize = ((amount * 3) as f64 / U8S_PER_TX as f64).ceil() as usize;
        let mut current_batch = 0;
        while current_batch < batches {
            unsafe {
                let mut bytes_read = 0;
                while bytes_read < recv_buf.u8_buf.len() {
                    let current_bytes = stream.read(&mut recv_buf.u8_buf[bytes_read..]).unwrap();
                    bytes_read += current_bytes;
                }
                for i in 0..recv_buf.u8_buf.len() {
                    result_temp.push(recv_buf.u8_buf[i]);
                }
            }
            current_batch += 1;
        }
        let mut result = Vec::new();
        for i in 0..amount as usize {
            result.push((result_temp[i * 3], result_temp[i * 3 + 1], result_temp[i * 3 + 2]));
        }
        result
    }

    pub fn mod_subtraction(x: Wrapping<u64>, y: Wrapping<u64>, prime: u64) -> Wrapping<u64> {
        Wrapping((x.0 as i64 - y.0 as i64).mod_floor(&(prime as i64)) as u64)
    }

    pub fn mod_subtraction_u8(x: u8, y: u8, prime: u8) -> u8 {
        (x as i8 - y as i8).mod_floor(&(prime as i8)) as u8
    }

    pub fn u64_to_byte_array(x: u64) -> [u8 ; 8] {

        [ ((x >> 56) & 0xff) as u8,
            ((x >> 48) & 0xff) as u8,
            ((x >> 40) & 0xff) as u8,
            ((x >> 32) & 0xff) as u8,
            ((x >> 24) & 0xff) as u8,
            ((x >> 16) & 0xff) as u8,
            ((x >>  8) & 0xff) as u8,
            (x & 0xff) as u8 ]
    }

    pub fn byte_array_to_u64(buf: [u8; 8]) -> u64 {


        ((buf[0] as u64) << 56) +
            ((buf[1] as u64) << 48 )+
            ((buf[2] as u64) << 40) +
            ((buf[3] as u64) << 32) +
            ((buf[4] as u64) << 24) +
            ((buf[5] as u64) << 16) +
            ((buf[6] as u64) <<  8) +
            (buf[7] as u64)
    }
}
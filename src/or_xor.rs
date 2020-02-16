pub mod or_xor{
    use std::num::Wrapping;
    use crate::computing_party::computing_party::ComputingParty;
    use crate::constants::constants::{BATCH_SIZE, U8S_PER_TX, BUF_SIZE, U64S_PER_TX, BINARY_PRIME};
    use std::io::{Read, Write, BufReader, BufRead};
    use threadpool::ThreadPool;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::cmp::{min, max};
    use num::integer::*;
    use num::bigint::{BigUint, ToBigUint, ToBigInt};
    use num::{Zero, One, FromPrimitive, abs, BigInt};
    use crate::utils::utils::*;
    use serde::{Serialize, Deserialize, Serializer};
    use std::net::TcpStream;
    use std::ops::{Add, Mul};
    use crate::multiplication::multiplication::{batch_multiply, batch_multiply_bigint, batch_multiplication_integer};

    pub fn or_xor(x_list: &Vec<Wrapping<u64>>,
                  y_list: &Vec<Wrapping<u64>>,
                  ctx: &mut ComputingParty, constant_multiplier: u64,prime:u64) -> Vec<Wrapping<u64>> {
        ctx.thread_hierarchy.push("or_xor".to_string());
        let bit_length = x_list.len();
        let mut i = 0;
        let mut output = Vec::new();

        if ctx.raw_tcp_communication{
            let mut global_index = 0;
            while i < bit_length {
                let to_index = min(i + ctx.batch_size, bit_length);
                let mut batch_mul_result = batch_multiplication_integer(&x_list[i..to_index].to_vec(), &y_list[i..to_index].to_vec(), ctx);
                for item in batch_mul_result.iter() {
                    let result = Wrapping(x_list[global_index].0) + Wrapping(y_list[global_index].0) - (Wrapping(constant_multiplier) * Wrapping(item.0));
                    output.push(Wrapping(mod_floor(result.0, prime)));
                    global_index += 1;
                }
                i = to_index;
            }
        }else{
            let thread_pool = ThreadPool::new((bit_length/ctx.batch_size)+1);
            let mut output_map = Arc::new(Mutex::new(HashMap::new()));
            let mut batch_count = 0;
            while i < bit_length {
                let mut output_map = Arc::clone(&output_map);
                let to_index = min(i + ctx.batch_size, bit_length);
                let mut ctx_copied = ctx.clone();
                ctx_copied.thread_hierarchy.push(format!("{}",batch_count));
                let mut x_list = x_list.clone();
                let mut y_list = y_list.clone();
                thread_pool.execute(move || {
                    let mut batch_mul_result = batch_multiplication_integer(&x_list[i..to_index].to_vec(), &y_list[i..to_index].to_vec(), &mut ctx_copied);
                    let mut output_map = output_map.lock().unwrap();
                    (*output_map).insert(batch_count, batch_mul_result);
                });

                i = to_index;
                batch_count += 1;
            }
            thread_pool.join();
            let output_map = &(*(output_map.lock().unwrap()));

            let mut global_index = 0;
            for i in 0..batch_count {
                let batch_result = output_map.get(&i).unwrap();
                for item in batch_result.iter() {
                    let result = Wrapping(x_list[global_index].0) + Wrapping(y_list[global_index].0) - (Wrapping(constant_multiplier) * Wrapping(item.0));
                    output.push(Wrapping(mod_floor(result.0, prime)));
                    global_index += 1;
                }
            }
        }

        ctx.thread_hierarchy.pop();
        println!("or_xor ends");
        output
    }

    pub fn or_xor_bigint(x_list: &Vec<BigUint>, y_list: &Vec<BigUint>, ctx: &mut ComputingParty, constant_multiplier: &BigUint) -> Vec<BigUint> {
        println!("or_xor_bigint starts");
        ctx.thread_hierarchy.push("or_xor_bigint".to_string());
        let bit_length = x_list.len();
        let mut output = Vec::new();
        let mut i = 0;
        if ctx.raw_tcp_communication{
            let mut global_index = 0;
            while i < bit_length {
                let to_index = min(i + ctx.batch_size, bit_length);
                let mut batch_mul_result = batch_multiply_bigint(&x_list[i..to_index].to_vec(), &y_list[i..to_index].to_vec(), ctx);
                for item in batch_mul_result.iter() {
                    let x_item = &x_list[global_index];
                    let y_item = &y_list[global_index];
                    output.push(mod_floor(big_uint_subtract(&(x_item.add(y_item)), &constant_multiplier.mul(item), &ctx.dt_training.big_int_prime), big_uint_clone(&ctx.dt_training.big_int_prime)));
                    global_index += 1;
                }
                i = to_index;
            }
        }else{
            let thread_pool = ThreadPool::new(ctx.thread_count);
            let mut output_map = Arc::new(Mutex::new(HashMap::new()));
            let mut batch_count = 0;
            while i < bit_length {
                let mut output_map = Arc::clone(&output_map);
                let to_index = min(i + ctx.batch_size, bit_length);
                let mut ctx_copied = ctx.clone();
                ctx_copied.thread_hierarchy.push(format!("{}",batch_count));
                let mut x_list_cp = big_uint_vec_clone(&x_list[i..to_index].to_vec());
                let mut y_list_cp = big_uint_vec_clone(&y_list[i..to_index].to_vec());
                thread_pool.execute(move || {
                    let mut batch_mul_result = batch_multiply_bigint(&x_list_cp, &y_list_cp, &mut ctx_copied);
                    let mut output_map = output_map.lock().unwrap();
                    (*output_map).insert(batch_count, batch_mul_result);
                });
                i = to_index;
                batch_count += 1;
            }
            thread_pool.join();
            let output_map = &(*(output_map.lock().unwrap()));
            let mut global_index = 0;
            for i in 0..batch_count {
                let batch_result = output_map.get(&i).unwrap();
                for item in batch_result.iter() {
                    let x_item = &x_list[global_index];
                    let y_item = &y_list[global_index];
                    output.push(mod_floor(big_uint_subtract(&(x_item.add(y_item)), &constant_multiplier.mul(item), &ctx.dt_training.big_int_prime), big_uint_clone(&ctx.dt_training.big_int_prime)));
                    global_index += 1;
                }
            }
        }


        ctx.thread_hierarchy.pop();
        println!("or_xor_bigint ends");
        output
    }
}
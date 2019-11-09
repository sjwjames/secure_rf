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
    use crate::multiplication::multiplication::{batch_multiply, batch_multiply_bigint};

    pub fn or_xor(x_list: &Vec<Wrapping<u64>>,
                  y_list: &Vec<Wrapping<u64>>,
                  ctx: &mut ComputingParty, constant_multiplier: u64) -> Vec<Wrapping<u64>> {
        let bit_length = x_list.len();
        let thread_pool = ThreadPool::new(ctx.thread_count);
        let mut i = 0;
        let mut output_map = Arc::new(Mutex::new(HashMap::new()));
        let mut batch_count = 0;
        while i < bit_length {
            let mut output_map = Arc::clone(&output_map);
            let to_index = min(i + ctx.batch_size, bit_length);
            let mut ctx_copied = ctx.clone();
            let mut x_list = x_list.clone();
            let mut y_list = y_list.clone();
            thread_pool.execute(move || {
                let mut batch_mul_result = batch_multiply(&x_list[i..to_index].to_vec(), &y_list[i..to_index].to_vec(), &mut ctx_copied);
                let mut output_map = output_map.lock().unwrap();
                (*output_map).insert(batch_count, batch_mul_result);
            });
            i += ctx.batch_size;
            batch_count += 1;
        }
        thread_pool.join();
        let output_map = &(*(output_map.lock().unwrap()));

        let mut global_index = 0;
        let mut output = Vec::new();
        for i in 0..batch_count {
            let batch_result = output_map.get(&i).unwrap();
            for item in batch_result.iter() {
                output.push(Wrapping(mod_floor(x_list[global_index].0 + y_list[global_index].0 - (constant_multiplier * item.0), ctx.dt_training.dataset_size_prime)));
                global_index += 1;
            }
        }
        output
    }

    pub fn or_xor_bigint(x_list: &Vec<BigUint>, y_list: &Vec<BigUint>, ctx: &mut ComputingParty, constant_multiplier: &BigUint) -> Vec<BigUint> {
        let bit_length = x_list.len();
        let mut output = vec![BigUint::zero(); bit_length];
        let thread_pool = ThreadPool::new(ctx.thread_count);
        let mut i = 0;
        let mut output_map = Arc::new(Mutex::new(HashMap::new()));
        let mut batch_count = 0;
        while i < bit_length {
            let mut output_map = Arc::clone(&output_map);
            let to_index = min(i + ctx.batch_size, bit_length);
            let mut ctx_copied = ctx.clone();
            let mut x_list = big_uint_vec_clone(x_list);
            let mut y_list = big_uint_vec_clone(y_list);
            thread_pool.execute(move || {
                let mut batch_mul_result = batch_multiply_bigint(&x_list[i..to_index].to_vec(), &y_list[i..to_index].to_vec(), &mut ctx_copied);
                let mut output_map = output_map.lock().unwrap();
                (*output_map).insert(batch_count, batch_mul_result);
            });
            i += ctx.batch_size;
            batch_count += 1;
        }
        thread_pool.join();
        let output_map = &(*(output_map.lock().unwrap()));
        let mut global_index = 0;
        let mut output = Vec::new();
        for i in 0..batch_count {
            let batch_result = output_map.get(&i).unwrap();
            for item in batch_result.iter() {
                let x_item = &x_list[global_index];
                let y_item = &y_list[global_index];
                output.push(mod_floor(big_uint_subtract(&(x_item.add(y_item)), &constant_multiplier.mul(item), &ctx.dt_training.big_int_prime), big_uint_clone(&ctx.dt_training.big_int_prime)));
                global_index += 1;
            }
        }
        output
    }
}
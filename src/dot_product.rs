pub mod dot_product{
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

    /* computed the dp modulo 2^64 of two vectors with pre/post truncation options */
    pub fn dot_product(x_list: &Vec<Wrapping<u64>>,
                       y_list: &Vec<Wrapping<u64>>,
                       ctx: &mut ComputingParty,
                       decimal_precision: u32,
                       truncate: bool,
                       pretruncate: bool) -> Wrapping<u64> {
        ctx.thread_hierarchy.push("dot_product".to_string());
        //println!("entering dot product");
        let z_list = batch_multiply(x_list, y_list, ctx);

        if !truncate {
            return z_list.iter().sum();
        }

        if !pretruncate {
            return truncate_local(
                z_list.iter().sum(), decimal_precision, (*ctx).asymmetric_bit,
            );
        }


        let mut z_trunc_list = vec![Wrapping(0); z_list.len()];
        for i in 0..z_list.len() {
            z_trunc_list[i] = truncate_local(
                z_list[i], decimal_precision, (*ctx).asymmetric_bit,
            );
        }
        ctx.thread_hierarchy.pop();
        z_trunc_list.iter().sum()
    }

    pub fn dot_product_bigint(x_list: &Vec<BigUint>, y_list: &Vec<BigUint>, ctx: &mut ComputingParty) -> BigUint {
        let mut dot_product = BigUint::zero();
        let vector_length = x_list.len();
        let thread_pool = ThreadPool::new(ctx.thread_count);
        let mut i = 0;
        let output_map = Arc::new(Mutex::new(HashMap::new()));
        let mut batch_count = 0;
        while i < vector_length {
            let to_index = min(i + ctx.batch_size, vector_length);
            let mut output_map = Arc::clone(&output_map);
            let mut ctx_copied = ctx.clone();
            let mut x_list_copied = big_uint_vec_clone(&x_list[i..to_index].to_vec());
            let mut y_list_copied = big_uint_vec_clone(&y_list[i..to_index].to_vec());
            thread_pool.execute(move || {
                let multi_result = batch_multiply_bigint(&x_list_copied, &y_list_copied, &mut ctx_copied);
                let mut output_map = output_map.lock().unwrap();
                (*output_map).insert(batch_count, multi_result);
            });
            batch_count += 1;
            i = to_index;
        }

        let output_map = &*(output_map.lock().unwrap());
        for i in 0..batch_count {
            let multi_result = output_map.get(&i).unwrap();
            for item in multi_result {
                dot_product = dot_product.add(item).mod_floor(&ctx.dt_training.big_int_prime);
            }
        }
        dot_product
    }

}
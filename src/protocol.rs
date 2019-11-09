pub mod protocol {
    /**
    ** @author Davis.R, James.S
    **/
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
    use crate::multiplication::multiplication::{batch_multiplication_byte, parallel_multiplication};

    union Xbuffer {
        u64_buf: [u64; U64S_PER_TX],
        u8_buf: [u8; U8S_PER_TX],
    }








    pub fn arg_max(bit_shares: Vec<Vec<u8>>, ctx: &mut ComputingParty) -> Vec<u8> {
        let number_count = bit_shares.len();

        let mut result = Vec::new();
        if number_count == 1 {
            result.push(1);
        } else {
            let mut bit_length = 0;
            bit_shares.iter().map(|x| bit_length = max(bit_length, x.len()));
            let mut w_intermediate = HashMap::new();

            for i in 0..number_count {
                let mut list = Vec::new();
                w_intermediate.insert(i,list);
            }
            //computeComparisons in JAVA Lynx
            let ti_count = 2 * bit_length + (bit_length * (bit_length - 1) / 2);
            let thread_pool = ThreadPool::new(ctx.thread_count);
            let mut output_map = Arc::new(Mutex::new((HashMap::new())));
            let mut key = 0;
            for i in 0..number_count {
                for j in 0..number_count {
                    if i != j {
                        let mut output_map = Arc::clone(&output_map);
                        let mut ctx_copied = ctx.clone();
                        let mut bit_shares = bit_shares.clone();
                        thread_pool.execute(move || {
                            key = i * number_count + j;
                            let comparison_result = comparison(&bit_shares[i], &bit_shares[j], &mut ctx_copied);
                            let mut output_map = output_map.lock().unwrap();
                            (*output_map).insert(key, comparison_result);
                        });
                    }
                }
            }

            let output_map = output_map.lock().unwrap();
            for i in 0..number_count * (number_count - 1) {
                let mut comparison = *output_map.get(&i).unwrap();
                let key = i / (number_count - 1);
                w_intermediate.get_mut(&key).unwrap().push(comparison);
            }

            let mut output_map = Arc::new(Mutex::new((HashMap::new())));
            //multi-threaded parallel multiplication
            for i in 0..number_count {
                let mut vec = Vec::new();
                for item in w_intermediate.get(&i).unwrap().iter() {
                    vec.push(*item);
                }
                let mut output_map = Arc::clone(&output_map);
                let mut ctx_copied = ctx.clone();
                thread_pool.execute(move || {
                    let multi_result = parallel_multiplication(&vec, &mut ctx_copied);
                    let mut output_map = output_map.lock().unwrap();
                    (*output_map).insert(i, multi_result);
                });
            }

            let output_map = &*(output_map.lock().unwrap());
            for i in 0..number_count {
                let multi_result = output_map.get(&i).unwrap();
                result[i] = *multi_result;
            }
        }
        result
    }

    pub fn comparison(x_list: &Vec<u8>, y_list: &Vec<u8>, ctx: &mut ComputingParty) -> u8 {
        let bit_length = max(x_list.len(), y_list.len());
        let prime = BINARY_PRIME;
        let ti_shares = &ctx.dt_shares.binary_triples;
        let ti_shares_start_index = *(ctx.dt_shares.current_binary_index.lock().unwrap());
        let mut e_shares = vec![0u8; bit_length];
        let mut d_shares = vec![0u8; bit_length];
        let mut c_shares = vec![0u8; bit_length];
        let mut multiplication_e = vec![0u8; bit_length];
        let mut w = -1;
        //computeEShares in Java Lynx
        for i in 0..bit_length {
            let e_share = x_list[i] + y_list[i] + ctx.asymmetric_bit;
            e_shares[i] = mod_floor(e_share, BINARY_PRIME as u8);
        }
        let thread_pool = ThreadPool::new(ctx.thread_count);
        //compute D shares
        let mut x_list_copied = x_list.clone();
        let mut y_list_copied = y_list.clone();
        let mut ctx_copied = ctx.clone();
        let mut d_shares_wrapper = Arc::new(Mutex::new(d_shares));
        let mut d_shares_copied = Arc::clone(&d_shares_wrapper);
        thread_pool.execute(move || {
            let (batch_count, output_map) = multi_thread_batch_mul_byte(&x_list_copied, &y_list_copied, &mut ctx_copied, bit_length);
            let mut global_index = 0;
            let mut d_shares_copied = d_shares_copied.lock().unwrap();
            for i in 0..batch_count {
                let product_result = output_map.get(&i).unwrap();
                for item in product_result {
                    let local_diff = y_list_copied[global_index] - *item;

                    (*d_shares_copied)[global_index] = mod_floor(local_diff, BINARY_PRIME as u8);
                    global_index += 1;
                }
            }
        });
        //compute multiplication E parallel
        let mut ctx_copied = ctx.clone();
        thread_pool.execute(move || {
            let mut main_index = bit_length - 1;
            main_index -= 1;
            multiplication_e[main_index] = e_shares[bit_length - 1];
            let mut temp_mul_e = vec![0u8; bit_length];
            while temp_mul_e.len() > 1 {
                let inner_pool = ThreadPool::new(ctx_copied.thread_count);
                let mut i = 0;
                let mut batch_count = 0;
                let mut output_map = Arc::new(Mutex::new(HashMap::new()));
                while i < temp_mul_e.len() - 1 {
                    let mut output_map = Arc::clone(&output_map);
                    let to_index = min(i + ctx_copied.batch_size, temp_mul_e.len());
                    let mut ctx_copied_inner = ctx_copied.clone();
                    let mut temp_mul_e = temp_mul_e.clone();
                    inner_pool.execute(move || {
                        let mut batch_mul_result = batch_multiplication_byte(&temp_mul_e[i..to_index - 1].to_vec(), &temp_mul_e[i + 1..to_index].to_vec(), &mut ctx_copied_inner);
                        let mut output_map = output_map.lock().unwrap();
                        (*output_map).insert(batch_count, batch_mul_result);
                    });
                    i += to_index - 1;
                    batch_count += 1;
                }
                inner_pool.join();
                let output_map = &*(output_map.lock().unwrap());
                let mut products = Vec::new();
                for i in 0..batch_count {
                    let product_result = output_map.get(&i).unwrap();
                    for item in product_result {
                        products.push(*item);
                    }
                }
                temp_mul_e.clear();
                temp_mul_e = products;
                main_index -= 1;
                multiplication_e[main_index] = *temp_mul_e.last().unwrap();
            }
            multiplication_e[0] = 0;
        });
        thread_pool.join();
        //compute c shares
        let mut i = 0;
        let mut batch_count = 0;
        let d_shares = &*(d_shares_wrapper.lock().unwrap());
        let mut output_map = Arc::new(Mutex::new(HashMap::new()));
        while i < bit_length - 1 {
            let mut output_map = Arc::clone(&output_map);
            let to_index = min(i + ctx.batch_size, bit_length - 1);
            let mut ctx_copied = ctx.clone();
            let mut d_shares_copied = d_shares[i..to_index].to_vec().to_vec().clone();
            let mut multiplication_e_copied = multiplication_e[i + 1..to_index + 1].to_vec().clone();
            thread_pool.execute(move || {
                let mut batch_mul_result = batch_multiplication_byte(&multiplication_e_copied, &d_shares_copied, &mut ctx_copied);
                let mut output_map = output_map.lock().unwrap();
                (*output_map).insert(batch_count, batch_mul_result);
            });
            i = to_index;
            batch_count += 1;
        }
        thread_pool.join();
        let output_map = &*(output_map.lock().unwrap());
        for i in 0..batch_count {
            let products = output_map.get(&i).unwrap();
            for j in 0..products.len() {
                let global_index = i * 10 + j;
                c_shares[global_index] = products[j];
            }
        }
        c_shares[bit_length - 1] = d_shares[bit_length - 1];
        //compute w shares
        w = ctx.asymmetric_bit as i8;
        for i in 0..bit_length {
            w += c_shares[i] as i8;
            w = mod_floor(w, BINARY_PRIME as i8);
        }
        w as u8
    }





    pub fn equality_big_integer(x: &BigUint, y: &BigUint, ctx: &mut ComputingParty) -> BigUint {
        let equality_share = get_current_equality_share(ctx);
        let bigint_share = get_current_bigint_share(ctx);
        let prime = &ctx.dt_training.big_int_prime;
        let diff = big_uint_subtract(x, y, prime);
        let mut diff_list = Vec::new();
        diff_list.push(big_uint_subtract(&diff, &bigint_share.0, prime));
        diff_list.push(big_uint_subtract(equality_share, &bigint_share.1, prime));
        let mut in_stream = ctx.in_stream.try_clone()
            .expect("failed cloning tcp o_stream");

        let mut o_stream = ctx.o_stream.try_clone()
            .expect("failed cloning tcp o_stream");

        let mut message = serialize_biguint_vec(diff_list);
        o_stream.write((message + "\n").as_bytes());

        let mut reader = BufReader::new(in_stream);
        let mut diff_list_message = String::new();
        reader.read_line(&mut diff_list_message).expect("fail to read diff list message");

        let mut diff_list = deserialize_biguint_vec(diff_list_message);

        let d = big_uint_subtract(&diff, &bigint_share.0, prime).add(&diff_list[0]).mod_floor(prime);
        let e = big_uint_subtract(&diff, &bigint_share.0, prime).add(&diff_list[0]).mod_floor(prime);
        let product = bigint_share.2.add(d.mul(&bigint_share.1)).add(&bigint_share.0.mul(e)).add(d.mul(&e).mul(BigUint::from(ctx.asymmetric_bit)));
        product
    }



}
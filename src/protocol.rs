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
    use crate::multiplication::multiplication::{batch_multiplication_byte, parallel_multiplication, multi_thread_batch_mul_byte};
    use crate::comparison::comparison::comparison;

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
                w_intermediate.insert(i, list);
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
        let product = big_uint_clone(&bigint_share.2).add(&big_uint_clone(&d).mul(&big_uint_clone(&bigint_share.1)))
                  .add(big_uint_clone(&bigint_share.0).mul(big_uint_clone(&e))).add(big_uint_clone(&d).mul(&big_uint_clone(&e)).mul(BigUint::from(ctx.asymmetric_bit)));
        product
    }
}
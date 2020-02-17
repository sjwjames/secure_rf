pub mod protocol {
    /**
    ** @author Davis.R, James.S
    **/
    use std::num::Wrapping;
    use crate::computing_party::computing_party::ComputingParty;
    use crate::constants::constants::{BATCH_SIZE, U8S_PER_TX, BUF_SIZE, U64S_PER_TX, BINARY_PRIME, LOCAL_ADDITION, LOCAL_SUBTRACTION};
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

    pub fn arg_max(bit_shares: &Vec<Vec<u8>>, ctx: &mut ComputingParty) -> Vec<u8> {
        println!("arg_max starts");
        ctx.thread_hierarchy.push("arg_max".to_string());
        let number_count = bit_shares.len();
        // denote result shares of pairwise comparisons
        let mut result = vec![0u8; number_count];
        if number_count == 1 {
            result[0] = 1;
        } else {
            // all the comparisons should happen in the same size of bit length
            let mut bit_length = 0;
            for item in bit_shares.iter() {
                bit_length = max(bit_length, item.len());
            }
            let mut w_intermediate = HashMap::new();

            for i in 0..number_count {
                let mut list = Vec::new();
                w_intermediate.insert(i, list);
            }
            //computeComparisons in JAVA Lynx
            if ctx.raw_tcp_communication {
                for i in 0..number_count {
                    for j in 0..number_count {
                        if i != j {
                            let comparison_result = comparison(&bit_shares[i], &bit_shares[j], ctx);
                            w_intermediate.get_mut(&i).unwrap().push(comparison_result);
                        }
                    }
                }

                for i in 0..number_count {
                    let mut vec = Vec::new();
                    for item in w_intermediate.get(&i).unwrap().iter() {
                        vec.push(*item);
                    }
                    let multi_result = parallel_multiplication(&vec, ctx);
                    result[i] = multi_result;
                }
            } else {
                let thread_pool = ThreadPool::new(ctx.thread_count);
                let mut output_map = Arc::new(Mutex::new((HashMap::new())));
                let mut key = 0;
                for i in 0..number_count {
                    for j in 0..number_count {
                        if i != j {
                            let mut output_map = Arc::clone(&output_map);
                            let mut ctx_copied = ctx.clone();
                            ctx_copied.thread_hierarchy.push(format!("{}", i * number_count + j));
                            let mut bit_shares = bit_shares.clone();
                            thread_pool.execute(move || {
                                let comparison_result = comparison(&bit_shares[i], &bit_shares[j], &mut ctx_copied);
                                let mut output_map = output_map.lock().unwrap();
                                (*output_map).insert(key, comparison_result);
                            });
                            key += 1;
                        }
                    }
                }
                thread_pool.join();
                let output_map = output_map.lock().unwrap();
                for i in 0..number_count * (number_count - 1) {
                    let mut comparison = *output_map.get(&i).unwrap();
                    let key = i / (number_count - 1);
                    w_intermediate.get_mut(&key).unwrap().push(comparison);
                }

                let mut output_map = Arc::new(Mutex::new((HashMap::new())));
                //multi-threaded parallel multiplication
                ctx.thread_hierarchy.push("parallel_multiplication".to_string());
                for i in 0..number_count {
                    let mut vec = Vec::new();
                    for item in w_intermediate.get(&i).unwrap().iter() {
                        vec.push(*item);
                    }
                    let mut output_map = Arc::clone(&output_map);
                    let mut ctx_copied = ctx.clone();
                    ctx_copied.thread_hierarchy.push(format!("{}", i));
                    thread_pool.execute(move || {
                        let multi_result = parallel_multiplication(&vec, &mut ctx_copied);
                        let mut output_map = output_map.lock().unwrap();
                        (*output_map).insert(i, multi_result);
                    });
                }
                thread_pool.join();
                ctx.thread_hierarchy.pop();
                let output_map = &*(output_map.lock().unwrap());
                for i in 0..number_count {
                    let multi_result = output_map.get(&i).unwrap();
                    result[i] = *multi_result;
                }
            }
        }

        ctx.thread_hierarchy.pop();
        println!("arg_max ends");
        result
    }

    pub fn equality_big_integer(x: &BigUint, y: &BigUint, ctx: &mut ComputingParty) -> BigUint {
        ctx.thread_hierarchy.push("equality_big_integer".to_string());
        let equality_share = get_current_equality_share(ctx);
        let bigint_share = get_current_bigint_share(ctx);
        let prime = &ctx.dt_training.big_int_prime;
        let diff = big_uint_subtract(x, y, prime);
        let mut diff_list = Vec::new();
        diff_list.push(big_uint_subtract(&diff, &bigint_share.0, prime));
        diff_list.push(big_uint_subtract(&equality_share, &bigint_share.1, prime));

        let mut diff_list_received = Vec::new();
        if ctx.raw_tcp_communication {
            let mut o_stream = ctx.o_stream.try_clone()
                .expect("failed cloning tcp o_stream");
            let mut in_stream = ctx.in_stream.try_clone().expect("failed cloning tcp o_stream");
            let mut reader = BufReader::new(in_stream);
            let mut share_message = String::new();
            if ctx.asymmetric_bit == 1 {
                o_stream.write((serialize_biguint_vec(&diff_list) + "\n").as_bytes());
                reader.read_line(&mut share_message).expect("fail to read share message str");
                diff_list_received = deserialize_biguint_vec(&share_message.as_str());
            } else {
                reader.read_line(&mut share_message).expect("fail to read share message str");
                diff_list_received = deserialize_biguint_vec(&share_message.as_str());
                o_stream.write((serialize_biguint_vec(&diff_list) + "\n").as_bytes());
            }
        } else {
            let mut diff_list_message = String::new();
            let message_id = ctx.thread_hierarchy.join(":");
            let message_content = serialize_biguint_vec(&diff_list);
            push_message_to_queue(&ctx.remote_mq_address, &message_id, &message_content);
            let message_received = receive_message_from_queue(&ctx.local_mq_address, &message_id, 1);
            diff_list_message = message_received[0].clone();
            let mut diff_list_received = deserialize_biguint_vec(&diff_list_message.as_str());
        }

        let d = big_uint_subtract(&diff, &bigint_share.0, prime).add(&diff_list_received[0]).mod_floor(prime);
        let e = big_uint_subtract(&equality_share, &bigint_share.1, prime).add(&diff_list_received[1]).mod_floor(prime);
        let product = (&bigint_share.2).add((&d).mul(&bigint_share.1)).add((&bigint_share.0).mul(&e)).add((&d).mul(&e).mul(BigUint::from_u8(ctx.asymmetric_bit).unwrap())).mod_floor(prime);
        ctx.thread_hierarchy.pop();
        product
    }


    //
    fn local_matrix_computation(x: &Vec<Vec<Wrapping<u64>>>, y: &Vec<Vec<Wrapping<u64>>>, prime: u64, operation: u8) -> Vec<Vec<Wrapping<u64>>> {
        let mut result = Vec::new();
        for i in 0..x.len() {
            let mut row = Vec::new();
            for j in 0..x[0].len() {
                match operation {
                    LOCAL_ADDITION => row.push(Wrapping((x[i][j] + y[i][j]).0.mod_floor(&prime))),
                    LOCAL_SUBTRACTION => row.push(Wrapping((x[i][j] - y[i][j]).0.mod_floor(&prime))),
                    _ => {}
                }
            }
            result.push(row);
        }
        result
    }

    fn local_matrix_multiplication(x: &Vec<Vec<Wrapping<u64>>>, y: &Vec<Vec<Wrapping<u64>>>, prime: u64) -> Vec<Vec<Wrapping<u64>>> {
        let i = x.len();
        let k = x[0].len();
        let j = y[0].len();
        let mut result = Vec::new();
        for m in 0..i {
            let mut row = Vec::new();
            for n in 0..j {
                let mut multi_result = Wrapping(0);
                for p in 0..k {
                    multi_result += (x[m][p] * y[p][n]);
                }
                multi_result = Wrapping(multi_result.0.mod_floor(&prime));
                row.push(multi_result);
            }
            result.push(row);
        }
        result
    }


    pub fn matrix_multiplication_integer(x: &Vec<Vec<Wrapping<u64>>>, y: &Vec<Vec<Wrapping<u64>>>, ctx: &ComputingParty,prime:u64,matrix_mul_shares:&(Vec<Vec<Wrapping<u64>>>,Vec<Vec<Wrapping<u64>>>,Vec<Vec<Wrapping<u64>>>)) -> Vec<Vec<Wrapping<u64>>> {
        let mut d_matrix = Vec::new();
        let mut e_matrix = Vec::new();
        let u_shares = matrix_mul_shares.0.clone();
        let v_shares = matrix_mul_shares.1.clone();
        let w_shares = matrix_mul_shares.2.clone();
        println!("{:?}",u_shares);
        println!("{:?}",v_shares);
        println!("{:?}",w_shares);

        let mut o_stream = ctx.o_stream.try_clone()
            .expect("failed cloning tcp o_stream");
        let mut in_stream = ctx.in_stream.try_clone().expect("failed cloning tcp o_stream");
        let mut reader = BufReader::new(in_stream);
        d_matrix = local_matrix_computation(x, &u_shares, prime, LOCAL_SUBTRACTION);

        e_matrix = local_matrix_computation(y, &v_shares, prime, LOCAL_SUBTRACTION);

        let mut d_matrix_received_str = String::new();
        let mut d_matrix_received = Vec::new();
        if ctx.asymmetric_bit == 1 {
            o_stream.write(format!("{}\n", serde_json::to_string(&d_matrix).unwrap()).as_bytes());
            reader.read_line(&mut d_matrix_received_str).expect("fail to read share message str");
            d_matrix_received = serde_json::from_str(&d_matrix_received_str).unwrap();
        } else {
            reader.read_line(&mut d_matrix_received_str).expect("fail to read share message str");
            d_matrix_received = serde_json::from_str(&d_matrix_received_str).unwrap();
            o_stream.write(format!("{}\n", serde_json::to_string(&d_matrix).unwrap()).as_bytes());
        }

        let mut e_matrix_received_str = String::new();
        let mut e_matrix_received = Vec::new();
        if ctx.asymmetric_bit == 1 {
            o_stream.write(format!("{}\n", serde_json::to_string(&e_matrix).unwrap()).as_bytes());
            reader.read_line(&mut e_matrix_received_str).expect("fail to read share message str");
            e_matrix_received = serde_json::from_str(&e_matrix_received_str).unwrap();
        } else {
            reader.read_line(&mut e_matrix_received_str).expect("fail to read share message str");
            e_matrix_received = serde_json::from_str(&e_matrix_received_str).unwrap();
            o_stream.write(format!("{}\n", serde_json::to_string(&e_matrix).unwrap()).as_bytes());
        }

        let mut d = local_matrix_computation(&d_matrix, &d_matrix_received, prime, LOCAL_ADDITION);
        let mut e = local_matrix_computation(&e_matrix, &e_matrix_received, prime, LOCAL_ADDITION);
        let de = local_matrix_multiplication(&d, &e, prime);
        let eu = local_matrix_multiplication(&u_shares, &e, prime);
        let dv = local_matrix_multiplication(&d, &v_shares, prime);
        let w_eu = local_matrix_computation(&w_shares, &eu, prime, LOCAL_ADDITION);
        let w_eu_dv = local_matrix_computation(&w_eu, &dv, prime, LOCAL_ADDITION);
        let result = if ctx.asymmetric_bit==1 { local_matrix_computation(&w_eu_dv, &de, prime, LOCAL_ADDITION)} else { w_eu_dv.clone() } ;
        println!("d:{:?}",d);
        println!("e:{:?}",e);
        println!("de:{:?}",de);
        println!("eu:{:?}",eu);
        println!("dv:{:?}",dv);
        println!("w_eu:{:?}",w_eu);
        println!("w_eu_dv:{:?}",w_eu_dv);
        println!("rfs result{:?}", result);
        result
    }

    pub fn batch_equality_integer(x: &Vec<Wrapping<u64>>, y: &Vec<Wrapping<u64>>, ctx: &mut ComputingParty,prime:u64) -> Vec<u64> {
        let range = x.len();
        let mut xy_diff = Vec::new();
        let mut diff_list = vec![vec![Wrapping(0 as u64); 2]; range];
        let mut result = Vec::new();
        let additive_shares = get_ohe_additive_shares(ctx, range);
        let equality_shares = get_current_equality_integer_shares(ctx, range);
        for i in 0..range {
            let diff = Wrapping((x[i] - y[i]).0.mod_floor(&prime));
            xy_diff.push(diff);
            diff_list[i][0] = Wrapping((diff - additive_shares[i].0).0.mod_floor(&prime));
            diff_list[i][1] = Wrapping((equality_shares[i] - additive_shares[i].1).0.mod_floor(&prime));
        }

        let mut o_stream = ctx.o_stream.try_clone()
            .expect("failed cloning tcp o_stream");
        let mut in_stream = ctx.in_stream.try_clone().expect("failed cloning tcp o_stream");
        let mut reader = BufReader::new(in_stream);
        let mut share_message = String::new();
        let mut received_list: Vec<Vec<Wrapping<u64>>> = Vec::new();
        if ctx.asymmetric_bit == 1 {
            o_stream.write((serde_json::to_string(&diff_list).unwrap() + "\n").as_bytes());
            reader.read_line(&mut share_message).expect("fail to read share message str");
            received_list = serde_json::from_str(&share_message).unwrap();
        } else {
            reader.read_line(&mut share_message).expect("fail to read share message str");
            received_list = serde_json::from_str(&share_message).unwrap();
            o_stream.write((serde_json::to_string(&diff_list).unwrap() + "\n").as_bytes());
        }

        for i in 0..range {
            let d = Wrapping((diff_list[i][0] + received_list[i][0]).0.mod_floor(&prime));
            let e = Wrapping((diff_list[i][1] + received_list[i][1]).0.mod_floor(&prime));
            let product = (additive_shares[i].2 + d * additive_shares[i].1 + additive_shares[i].0 * e + d * e * Wrapping(ctx.asymmetric_bit as u64)).0.mod_floor(&prime);
            result.push(product);
        }

        result
    }
}
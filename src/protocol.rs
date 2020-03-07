pub mod protocol {
    /**
    ** @author Davis.R, James.S
    **/
    use std::num::Wrapping;
    use crate::computing_party::computing_party::ComputingParty;
    use crate::constants::constants::{BATCH_SIZE, U8S_PER_TX, BUF_SIZE, U64S_PER_TX, BINARY_PRIME, LOCAL_ADDITION, LOCAL_SUBTRACTION, CR_BIN_1, CR_BIN_0};
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
    use crate::or_xor::or_xor::or_xor;
    use crate::bit_decomposition::bit_decomposition::{batch_log_decomp, bit_decomposition_opt, bit_decomposition};
    use crate::discretize::discretize::{xor_share_to_additive, reveal_submodule};
    use crate::constants::constants;

    const BATCH_SIZE_REVEAL: usize = constants::REVEAL_BATCH_SIZE;

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
            diff_list_received = send_biguint_messages(ctx, &diff_list);
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
                    LOCAL_SUBTRACTION => row.push(mod_subtraction(x[i][j], y[i][j], prime)),
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


    pub fn matrix_multiplication_integer(x: &Vec<Vec<Wrapping<u64>>>, y: &Vec<Vec<Wrapping<u64>>>, ctx: &ComputingParty, prime: u64, matrix_mul_shares: &(Vec<Vec<Wrapping<u64>>>, Vec<Vec<Wrapping<u64>>>, Vec<Vec<Wrapping<u64>>>)) -> Vec<Vec<Wrapping<u64>>> {
        let mut d_matrix = Vec::new();
        let mut e_matrix = Vec::new();
        let u_shares = matrix_mul_shares.0.clone();
        let v_shares = matrix_mul_shares.1.clone();
        let w_shares = matrix_mul_shares.2.clone();
//        println!("{:?}", u_shares);
//        println!("{:?}", v_shares);
//        println!("{:?}", w_shares);

//        let mut o_stream = ctx.o_stream.try_clone()
//            .expect("failed cloning tcp o_stream");
//        let mut in_stream = ctx.in_stream.try_clone().expect("failed cloning tcp o_stream");
//        let mut reader = BufReader::new(in_stream);
        d_matrix = local_matrix_computation(x, &u_shares, prime, LOCAL_SUBTRACTION);

        e_matrix = local_matrix_computation(y, &v_shares, prime, LOCAL_SUBTRACTION);

        let mut d_matrix_received = send_receive_u64_matrix(&d_matrix, ctx);

        let mut e_matrix_received = send_receive_u64_matrix(&e_matrix, ctx);

        let mut d = local_matrix_computation(&d_matrix, &d_matrix_received, prime, LOCAL_ADDITION);
        let mut e = local_matrix_computation(&e_matrix, &e_matrix_received, prime, LOCAL_ADDITION);
        let de = local_matrix_multiplication(&d, &e, prime);
        let eu = local_matrix_multiplication(&u_shares, &e, prime);
        let dv = local_matrix_multiplication(&d, &v_shares, prime);
        let w_eu = local_matrix_computation(&w_shares, &eu, prime, LOCAL_ADDITION);
        let w_eu_dv = local_matrix_computation(&w_eu, &dv, prime, LOCAL_ADDITION);
        let result = if ctx.asymmetric_bit == 1 { local_matrix_computation(&w_eu_dv, &de, prime, LOCAL_ADDITION) } else { w_eu_dv.clone() };
//        println!("d:{:?}", d);
//        println!("e:{:?}", e);
//        println!("de:{:?}", de);
//        println!("eu:{:?}", eu);
//        println!("dv:{:?}", dv);
//        println!("w_eu:{:?}", w_eu);
//        println!("w_eu_dv:{:?}", w_eu_dv);
//        println!("rfs result{:?}", result);
        result
    }

    pub fn batch_equality_integer(x: &Vec<Wrapping<u64>>, y: &Vec<Wrapping<u64>>, ctx: &mut ComputingParty, prime: u64) -> Vec<Wrapping<u64>> {
        let range = x.len();
        let mut xy_diff = Vec::new();
        let mut diff_list = vec![vec![Wrapping(0 as u64); 2]; range];
        let mut result = Vec::new();
        let additive_shares = get_additive_shares(ctx, range, prime);
        let equality_shares = get_current_equality_integer_shares(ctx, range, prime);
        let mut list_sent = Vec::new();
        for i in 0..range {
            let diff = mod_subtraction(x[i], y[i], prime);
            xy_diff.push(diff);
            let item0 = mod_subtraction(diff, additive_shares[i].0, prime);
            let item1 = mod_subtraction(equality_shares[i], additive_shares[i].1, prime);
            diff_list[i][0] = item0;
            diff_list[i][1] = item1;
            list_sent.push(item0);
            list_sent.push(item1);
        }

        let received_list = send_u64_messages(ctx, &list_sent);
//
//        let mut o_stream = ctx.o_stream.try_clone()
//            .expect("failed cloning tcp o_stream");
//        let mut in_stream = ctx.in_stream.try_clone().expect("failed cloning tcp o_stream");
//        let mut reader = BufReader::new(in_stream);
//        let mut share_message = String::new();
//        let mut received_list: Vec<Vec<Wrapping<u64>>> = Vec::new();
//        if ctx.asymmetric_bit == 1 {
//            o_stream.write((serde_json::to_string(&diff_list).unwrap() + "\n").as_bytes());
//            reader.read_line(&mut share_message).expect("fail to read share message str");
//            received_list = serde_json::from_str(&share_message).unwrap();
//        } else {
//            reader.read_line(&mut share_message).expect("fail to read share message str");
//            received_list = serde_json::from_str(&share_message).unwrap();
//            o_stream.write((serde_json::to_string(&diff_list).unwrap() + "\n").as_bytes());
//        }

        for i in 0..range {
            let d = Wrapping((diff_list[i][0] + received_list[i * 2]).0.mod_floor(&prime));
            let e = Wrapping((diff_list[i][1] + received_list[i * 2 + 1]).0.mod_floor(&prime));
            let product = (additive_shares[i].2 + d * additive_shares[i].1 + additive_shares[i].0 * e + d * e * Wrapping(ctx.asymmetric_bit as u64)).0.mod_floor(&prime);
            result.push(Wrapping(product));
        }

        result
    }

    pub fn convert_integer_to_bits(x: u64, bit_length: usize) -> Vec<u8> {
        let mut result = Vec::new();
        let binary_str = format!("{:b}", x);
        let reversed_binary_vec: Vec<char> = binary_str.chars().rev().collect();
        for item in &reversed_binary_vec {
            let item_u8: u8 = format!("{}", item).parse().unwrap();
            result.push(item_u8);
        }
        if bit_length > reversed_binary_vec.len() {
            let mut temp = vec![0u8; bit_length - reversed_binary_vec.len()];
            result.append(&mut temp);
        } else {
            result = result[0..bit_length].to_vec();
        }
        result
    }


    pub fn equality_integer_over_field(x: u64, y: u64, ctx: &mut ComputingParty, bit_length: usize) -> u8 {
        let x_bits = convert_integer_to_bits(x, bit_length);
        let y_bits = convert_integer_to_bits(y, bit_length);
        let mut c = 0;
        let mut c_list = Vec::new();
        for i in 0..bit_length {
            c_list.push(Wrapping((x_bits[i] + y_bits[i]).mod_floor(&(BINARY_PRIME as u8)) as u64));
//            c_list.push((x_bits[i] + y_bits[i]).mod_floor(&(BINARY_PRIME as u8)));
        }

        let mut current_length = bit_length;
        let mut temp_list = c_list.clone();
        while current_length > 1 {
            if current_length % 2 == 0 {
                temp_list = or_xor(&temp_list[0..current_length / 2].to_vec(), &temp_list[current_length / 2..current_length].to_vec(), ctx, 1, 2);
//                temp_list = batch_multiplication_byte(&temp_list[0..current_length / 2].to_vec(), &temp_list[current_length / 2..current_length].to_vec(), ctx);
            } else {
                let mut mid = temp_list[current_length / 2];
                temp_list = or_xor(&temp_list[0..current_length / 2].to_vec(), &temp_list[current_length / 2 + 1..current_length].to_vec(), ctx, 1, 2);
//                temp_list = batch_multiplication_byte(&temp_list[0..current_length / 2].to_vec(), &temp_list[current_length / 2 + 1..current_length].to_vec(), ctx);

                temp_list.push(mid);
            }
            current_length = temp_list.len();
        }

        (temp_list[0].0 as u8 + ctx.asymmetric_bit).mod_floor(&(BINARY_PRIME as u8))
//        for i in 0..c_list.len() {
//            c =
//        }
//        (c + ctx.asymmetric_bit).mod_floor(&(BINARY_PRIME as u8))
    }


    pub fn batch_bitwise_and(x_list: &Vec<u64>,
                             y_list: &Vec<u64>,
                             ctx: &mut ComputingParty,
                             invert_output: bool) -> Vec<u64> {
        let len = (*x_list).len();
        let mut z_list: Vec<u64> = vec![0u64; len];

        let mut remainder = len;
        let mut index = 0;
        while remainder > BATCH_SIZE {
            let mut x_sublist = [0u64; BATCH_SIZE];
            let mut y_sublist = [0u64; BATCH_SIZE];

            x_sublist.clone_from_slice(&(x_list[BATCH_SIZE * index..BATCH_SIZE * (index + 1)]));
            y_sublist.clone_from_slice(&(y_list[BATCH_SIZE * index..BATCH_SIZE * (index + 1)]));

            let z_sublist = batch_bitwise_and_submodule(x_sublist, y_sublist, BATCH_SIZE, ctx);

            z_list[BATCH_SIZE * index..BATCH_SIZE * (index + 1)].clone_from_slice(&z_sublist);

            remainder -= BATCH_SIZE;
            index += 1;
        }

        let mut x_sublist = [0u64; BATCH_SIZE];
        let mut y_sublist = [0u64; BATCH_SIZE];

        x_sublist[0..remainder].clone_from_slice(&(x_list[BATCH_SIZE * index..]));
        y_sublist[0..remainder].clone_from_slice(&(y_list[BATCH_SIZE * index..]));

        let z_sublist = batch_bitwise_and_submodule(x_sublist, y_sublist, remainder, ctx);

        z_list[BATCH_SIZE * index..].clone_from_slice(&(z_sublist[..remainder]));

        if invert_output {
            let inversion_mask = (-Wrapping((*ctx).asymmetric_bit as u64)).0;
            for i in 0..len {
                z_list[i] ^= inversion_mask;
            }
        }

        z_list
    }

    pub fn batch_bitwise_and_submodule(x_list: [u64; BATCH_SIZE],
                                       y_list: [u64; BATCH_SIZE],
                                       tx_len: usize,
                                       ctx: &mut ComputingParty) -> [u64; BATCH_SIZE] {
        let asymmetric_bit = ctx.asymmetric_bit as u64;
        let inversion_mask: u64 = (-Wrapping(asymmetric_bit)).0;

        let mut in_stream = ctx.in_stream.try_clone()
            .expect("failed cloning tcp in_stream");

        let mut o_stream = ctx.o_stream.try_clone()
            .expect("failed cloning tcp o_stream");

        let mut u_list = [0u64; BATCH_SIZE];
        let mut v_list = [0u64; BATCH_SIZE];
        let mut w_list = [0u64; BATCH_SIZE];
        let mut d_list = [0u64; BATCH_SIZE];
        let mut e_list = [0u64; BATCH_SIZE];
        let mut z_list = [0u64; BATCH_SIZE];

        {
            for i in 0..tx_len {

                //let (u, v, w) = corr_rand_xor.pop().unwrap();
                let (u, v, w) = if asymmetric_bit == 1 { CR_BIN_1 } else { CR_BIN_0 };

                u_list[i] = u;
                v_list[i] = v;
                w_list[i] = w;

                d_list[i] = x_list[i] ^ u;
                e_list[i] = y_list[i] ^ v;
            }
        }

        let mut tx_buf = Xbuffer { u8_buf: [0u8; BUF_SIZE] };
        let mut rx_buf = Xbuffer { u8_buf: [0u8; BUF_SIZE] };

        for i in (0..2 * tx_len).step_by(2) {
            let d = d_list[i / 2];
            let e = e_list[i / 2];

            unsafe {
                tx_buf.u64_buf[i] = d;
                tx_buf.u64_buf[i + 1] = e;
            }
        }

        if ctx.asymmetric_bit == 1 {
            let mut bytes_written = 0;
            while bytes_written < BUF_SIZE {
                let current_bytes = unsafe {
                    o_stream.write(&tx_buf.u8_buf[bytes_written..]).unwrap()
                };
                bytes_written += current_bytes;
            }

            let mut bytes_read = 0;
            while bytes_read < BUF_SIZE {
                let current_bytes = unsafe {
                    match in_stream.read(&mut rx_buf.u8_buf[bytes_read..]) {
                        Ok(size) => size,
                        Err(_) => panic!("couldn't read"),
                    }
                };
                bytes_read += current_bytes;
            }
        } else {
            let mut bytes_read = 0;
            while bytes_read < BUF_SIZE {
                let current_bytes = unsafe {
                    match in_stream.read(&mut rx_buf.u8_buf[bytes_read..]) {
                        Ok(size) => size,
                        Err(_) => panic!("couldn't read"),
                    }
                };
                bytes_read += current_bytes;
            }

            let mut bytes_written = 0;
            while bytes_written < BUF_SIZE {
                let current_bytes = unsafe {
                    o_stream.write(&tx_buf.u8_buf[bytes_written..]).unwrap()
                };
                bytes_written += current_bytes;
            }
        }

        for i in (0..2 * tx_len).step_by(2) {
            let d = d_list[i / 2] ^ unsafe { rx_buf.u64_buf[i] };
            let e = e_list[i / 2] ^ unsafe { rx_buf.u64_buf[i + 1] };

            let u = u_list[i / 2];
            let v = v_list[i / 2];
            let w = w_list[i / 2];

            z_list[i / 2] = w ^ (d & v) ^ (u & e) ^ (d & e & inversion_mask);
        }

        z_list
    }


    // returns x == y ? 1 : 0 for all values in x_list, y_list
    pub fn batch_equality(x_list : &Vec<Wrapping<u64>>,
                          y_list : &Vec<Wrapping<u64>>,
                          ctx    : &mut ComputingParty) -> Vec<u64> {


        let len = x_list.len();
        let mut op1: Vec<Wrapping<u64>> = x_list.iter().zip(y_list.iter()).map(|(&x, &y)| (x - y)).collect();
        op1.append( &mut x_list.iter().zip(y_list.iter()).map(|(&x, &y)| (y - x)).collect() );

        let diff_dc  = batch_bit_extract(
            &op1,
            (ctx.integer_precision + ctx.decimal_precision + 1) as usize,
            ctx
        );

        (0..len).map(|i| (diff_dc[i] ^ diff_dc[i+len]) ^ (ctx.asymmetric_bit as u64))
            .collect()

    }

    pub fn batch_bit_extract(x_additive_list : &Vec<Wrapping<u64>>,
                             bit_pos         : usize,
                             ctx             : &mut ComputingParty) -> Vec<u64> {

        assert!(0 <= bit_pos && bit_pos < 64);
        let len = x_additive_list.len();

        if bit_pos == 0 {
            return x_additive_list.iter().map(|x| 1 & x.0).collect();
        }

        let propogate: Vec<u64> = x_additive_list.iter().map(|x| x.0).collect();
        let mut p_layer = propogate.clone();
        let mut g_layer = if ctx.asymmetric_bit == 0 {
            batch_bitwise_and(&propogate, &vec![0u64; len], ctx, false)
        } else {
            batch_bitwise_and(&vec![0u64; len], &propogate, ctx, false)
        };

        let mut matrices = bit_pos;
        while matrices > 1 {

            let pairs = matrices / 2;
            let remainder = matrices % 2;

            let mut p      = vec![0u64 ; len];
            let mut p_next = vec![0u64 ; len];
            let mut g      = vec![0u64 ; len];
            let mut g_next = vec![0u64 ; len];

            for j in 0..len {
                for i in 0..pairs {
                    p[j] |= ((p_layer[j] >> (2 * i) as u64) & 1) << i as u64;
                    g[j] |= ((g_layer[j] >> (2 * i) as u64) & 1) << i as u64;
                    p_next[j] |= ((p_layer[j] >> (2 * i + 1) as u64) & 1) << i as u64;
                    g_next[j] |= ((g_layer[j] >> (2 * i + 1) as u64) & 1) << i as u64;
                }
            }

            let mut l_ops = p_next.clone();
            l_ops.append(&mut p_next);

            let mut r_ops = p.clone();
            r_ops.append(&mut g);

            let matmul = batch_bitwise_and(&l_ops, &r_ops, ctx, false);

            // let l_ops = utility::compactify_bit_vector( &l_ops, pairs as usize );
            // let r_ops = utility::compactify_bit_vector( &r_ops, pairs as usize );
            // let matmul = batch_bitwise_and(&l_ops, &r_ops, ctx, false);
            // let matmul = utility::decompactify_bit_vector(&matmul, pairs as usize, 2*len);

            let mut p_layer_next = vec![0u64 ; len];
            let mut g_layer_next = vec![0u64 ; len];

            for j in 0..len {
                for i in 0..pairs {
                    p_layer_next[j] |= ((matmul[j] >> i as u64) & 1) << i as u64;
                    g_layer_next[j] |= (((g_next[j] >> i as u64) ^ (matmul[j + len] >> i as u64)) & 1) << i as u64;
                }

                if remainder == 1 {
                    p_layer_next[j] |= ((p_layer[j] >> (matrices - 1) as u64) & 1) << pairs as u64;
                    g_layer_next[j] |= ((g_layer[j] >> (matrices - 1) as u64) & 1) << pairs as u64;
                }
            }

            p_layer = p_layer_next;
            g_layer = g_layer_next;
            matrices = pairs + remainder;
        }

        g_layer.iter().zip(&propogate).map(|(&g, &p)| 1 & (g ^ (p >> bit_pos)) ).collect()

    }

    pub fn reveal_xor(x_list : &Vec<u64>,
                      ctx    : &mut ComputingParty) -> Vec<Wrapping<u64>> {


        let x_additive = xor_share_to_additive( x_list, ctx, 64 );

        reveal_wrapping(&x_additive, ctx)
    }

    // recombine a list of secret shares with option to map from ring 2^64 to real.
    pub fn reveal_wrapping(x_list : &Vec<Wrapping<u64>>,
                           ctx    : &mut ComputingParty) -> Vec<Wrapping<u64>> {

        let len = (*x_list).len();
        let mut x_combined = vec![ Wrapping(0u64) ; len];
        let mut remainder = len.clone();
        let mut index = 0;

        while remainder > BATCH_SIZE_REVEAL {

            let mut x_sublist = [ Wrapping(0) ; BATCH_SIZE_REVEAL];
            x_sublist.clone_from_slice(&(x_list[BATCH_SIZE_REVEAL*index..BATCH_SIZE_REVEAL*(index+1)]));

            let x_combined_sublist = reveal_submodule(x_sublist, BATCH_SIZE_REVEAL, ctx,false);

            x_combined[BATCH_SIZE_REVEAL*index..BATCH_SIZE_REVEAL*(index+1)].clone_from_slice(&x_combined_sublist);

            remainder -= BATCH_SIZE_REVEAL;
            index += 1;
        }

        let mut x_sublist = [ Wrapping(0) ; BATCH_SIZE_REVEAL];
        x_sublist[0..remainder].clone_from_slice(&(x_list[BATCH_SIZE_REVEAL*index..]));

        let x_combined_sublist = reveal_submodule(x_sublist, BATCH_SIZE_REVEAL, ctx,false);

        x_combined[BATCH_SIZE_REVEAL*index..].clone_from_slice(&(x_combined_sublist[..remainder]));

        x_combined
    }
}
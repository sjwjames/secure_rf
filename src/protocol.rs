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
    use num::{Zero, One, FromPrimitive, abs};
    use crate::utils::utils::{big_uint_subtract, big_uint_vec_clone, big_uint_clone, truncate_local, increment_current_share_index, ShareType, serialize_biguint_vec, deserialize_biguint_vec};
    use serde::{Serialize, Deserialize, Serializer};
    use std::net::TcpStream;
    use std::ops::{Add, Mul};

    union Xbuffer {
        u64_buf: [u64; U64S_PER_TX],
        u8_buf: [u8; U8S_PER_TX],
    }

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

    pub fn change_binary_to_decimal_field(binary_numbers: &Vec<u8>, ctx: &mut ComputingParty) -> Vec<Wrapping<u64>> {
        let mut dummy_list = vec![Wrapping(0u64); binary_numbers.len()];
        let mut output = Vec::new();
        let mut binary_int_list = Vec::new();
        for item in binary_numbers {
            binary_int_list.push(Wrapping(*item as u64));
        }
        if ctx.asymmetric_bit == 1 {
            output = or_xor(&binary_int_list, &dummy_list, ctx, 2);
        } else {
            output = or_xor(&dummy_list, &binary_int_list, ctx, 2);
        }
        output
    }

    pub fn change_binary_to_bigint_field(binary_numbers: &Vec<u8>, ctx: &mut ComputingParty) -> Vec<BigUint> {
        let mut binary_num_bigint = Vec::new();
        for item in binary_numbers.iter() {
            binary_num_bigint.push(item.to_biguint().unwrap());
        }
        let mut dummy_list = vec![BigUint::zero(); binary_numbers.len()];

        let mut output = Vec::new();
        if ctx.asymmetric_bit == 1 {
            output = or_xor_bigint(&binary_num_bigint, &dummy_list, ctx, &BigUint::from_usize(2).unwrap());
        } else {
            output = or_xor_bigint(&dummy_list, &binary_num_bigint, ctx, &BigUint::from_usize(2).unwrap());
        }
        output
    }

    /* computed the dp modulo 2^64 of two vectors with pre/post truncation options */
    pub fn dot_product(x_list: &Vec<Wrapping<u64>>,
                       y_list: &Vec<Wrapping<u64>>,
                       ctx: &mut ComputingParty,
                       decimal_precision: u32,
                       truncate: bool,
                       pretruncate: bool) -> Wrapping<u64> {

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
        z_trunc_list.iter().sum()
    }

    pub fn batch_multiply_bigint(x_list: &Vec<BigUint>, y_list: &Vec<BigUint>, ctx: &mut ComputingParty) -> Vec<BigUint> {
        let mut result = vec![BigUint::zero(); x_list.len()];
        let mut diff_list = Vec::new();
        let prime = big_uint_clone(&ctx.dt_training.big_int_prime);
        for i in 0..x_list.len() {
            diff_list.push((big_uint_subtract(&x_list[i], &ctx.dt_shares.additive_bigint_triples[i].0, &prime),
                            big_uint_subtract(&y_list[i], &ctx.dt_shares.additive_bigint_triples[i].1, &prime)));
        }
        let mut in_stream = ctx.in_stream.try_clone()
            .expect("failed cloning tcp o_stream");

        let mut o_stream = ctx.o_stream.try_clone()
            .expect("failed cloning tcp o_stream");

        let mut diff_list_str_vec = Vec::new();
        for item in diff_list.iter() {
            let mut tuple = Vec::new();
            tuple.push(serde_json::to_string(&(item.0.to_bytes_le())).unwrap());
            tuple.push(serde_json::to_string(&(item.1.to_bytes_le())).unwrap());
            diff_list_str_vec.push(format!("({})", tuple.join(",")));
        }
        o_stream.write((diff_list_str_vec.join(";") + "\n").as_bytes());

        let mut reader = BufReader::new(in_stream);
        let mut diff_list_message = String::new();
        reader.read_line(&mut diff_list_message).expect("fail to read diff list message");

        let mut diff_list_str_vec: Vec<&str> = diff_list_message.split(";").collect();
        let mut diff_list = Vec::new();
        for item in diff_list_str_vec {
            let temp_str = &item[1..item.len()];
            let str_vec: Vec<&str> = temp_str.split(",").collect();
            diff_list.push(
                (
                    BigUint::from_bytes_le(str_vec[0].as_bytes()),
                    BigUint::from_bytes_le(str_vec[1].as_bytes())
                )
            );
        }
        let batch_size = x_list.len();
        let mut d_list = Vec::new();
        let mut e_list = Vec::new();

        for i in 0..batch_size {
            d_list.push(diff_list[i].0.mod_floor(&prime));
            e_list.push(diff_list[i].1.mod_floor(&prime));
        }

        let big_int_shares = &ctx.dt_shares.additive_bigint_triples;
        let big_asymmetric_bit = if ctx.asymmetric_bit == 1 { BigUint::one() } else { BigUint::zero() };
        for i in 0..batch_size {
            let u = &big_int_shares[i].0;
            let v = &big_int_shares[i].1;
            let w = &big_int_shares[i].2;
            let d = &big_uint_subtract(&x_list[i], u, &prime).mod_floor(&prime).add(&d_list[i]).mod_floor(&prime);
            let e = &big_uint_subtract(&y_list[i], v, &prime).mod_floor(&prime).add(&e_list[i]).mod_floor(&prime);
            result[i] = w.add(&d.mul(v)).add(&e.mul(u)).add(&(&d.mul(e)).mul(&big_asymmetric_bit)).mod_floor(&prime);
        }

        result
    }

    /* computes entrywise product modulo 2^64 of two vectors */
    pub fn batch_multiply(x_list: &Vec<Wrapping<u64>>, y_list: &Vec<Wrapping<u64>>, ctx: &mut ComputingParty) -> Vec<Wrapping<u64>> {
        let mut z_list: Vec<Wrapping<u64>> = vec![Wrapping(0); (*x_list).len()];

        let mut remainder = (*x_list).len();
        let mut index = 0;
        while remainder > BATCH_SIZE {

            //if ctx.debug_output { println!("[index={}][remainder={}]", index, remainder); }
            let mut x_sublist = [Wrapping(0); BATCH_SIZE];
            let mut y_sublist = [Wrapping(0); BATCH_SIZE];

            x_sublist.clone_from_slice(&(x_list[BATCH_SIZE * index..BATCH_SIZE * (index + 1)]));
            y_sublist.clone_from_slice(&(y_list[BATCH_SIZE * index..BATCH_SIZE * (index + 1)]));

            let z_sublist = batch_multiplication_submodule(x_sublist, y_sublist, BATCH_SIZE, ctx);

            z_list[BATCH_SIZE * index..BATCH_SIZE * (index + 1)].clone_from_slice(&z_sublist);

            remainder -= BATCH_SIZE;
            index += 1;
        }

        //if ctx.debug_output {println!("[index={}][remainder={}]", index, remainder);}
        let mut x_sublist = [Wrapping(0); BATCH_SIZE];
        let mut y_sublist = [Wrapping(0); BATCH_SIZE];

        x_sublist[0..remainder].clone_from_slice(&(x_list[BATCH_SIZE * index..]));
        y_sublist[0..remainder].clone_from_slice(&(y_list[BATCH_SIZE * index..]));

        let z_sublist = batch_multiplication_submodule(x_sublist, y_sublist, remainder, ctx);

        z_list[BATCH_SIZE * index..].clone_from_slice(&(z_sublist[..remainder]));


        z_list
    }

    // submodule does granular computations and alerts client/server threads
// to send and recv data
    pub fn batch_multiplication_submodule(x_list: [Wrapping<u64>; BATCH_SIZE],
                                          y_list: [Wrapping<u64>; BATCH_SIZE],
                                          tx_len: usize,
                                          ctx: &mut ComputingParty) -> [Wrapping<u64>; BATCH_SIZE] {
        let asymmetric_bit = Wrapping(ctx.asymmetric_bit as u64);

        let mut in_stream = ctx.in_stream.try_clone()
            .expect("failed cloning tcp o_stream");

        let mut o_stream = ctx.o_stream.try_clone()
            .expect("failed cloning tcp o_stream");

        let mut u_list = [Wrapping(0); BATCH_SIZE];
        let mut v_list = [Wrapping(0); BATCH_SIZE];
        let mut w_list = [Wrapping(0); BATCH_SIZE];
        let mut d_list = [Wrapping(0); BATCH_SIZE];
        let mut e_list = [Wrapping(0); BATCH_SIZE];
        let mut z_list = [Wrapping(0); BATCH_SIZE];

        {
            let corr_rand = &mut ctx.dt_shares.additive_triples;
            for i in 0..tx_len {
                let (u, v, w) = corr_rand.pop().unwrap();
                //let (u, v, w) = if ctx.asymmetric_bit == 1 {CR_1} else {CR_0};

                u_list[i] = u;
                v_list[i] = v;
                w_list[i] = w;

                d_list[i] = x_list[i] - u;
                e_list[i] = y_list[i] - v;
            }
        }

        let mut tx_buf = Xbuffer { u8_buf: [0u8; U8S_PER_TX] };
        let mut rx_buf = Xbuffer { u8_buf: [0u8; U8S_PER_TX] };

        for i in (0..2 * tx_len).step_by(2) {
            let d = d_list[i / 2].0;
            let e = e_list[i / 2].0;

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
            let d = d_list[i / 2] + unsafe { Wrapping(rx_buf.u64_buf[i]) };
            let e = e_list[i / 2] + unsafe { Wrapping(rx_buf.u64_buf[i + 1]) };

            let u = u_list[i / 2];
            let v = v_list[i / 2];
            let w = w_list[i / 2];

            z_list[i / 2] = w + d * v + u * e + d * e * asymmetric_bit;
        }

        z_list
    }

    pub fn bit_decomposition(input: u64, ctx: &mut ComputingParty) -> Vec<u8> {
        let mut input_shares = Vec::new();
        let mut d_shares = Vec::new();
        let mut e_shares = Vec::new();
        let mut c_shares = Vec::new();
        let mut y_shares = Vec::new();
        let mut x_shares = Vec::new();
        let binary_str = format!("{:b}", input);
        let input_binary_str_vec: Vec<&str> = binary_str.split("").collect();
        let mut temp: Vec<u8> = Vec::new();
        for item in input_binary_str_vec {
            temp.push(item.parse::<u8>().unwrap());
        }
        let bit_length = ctx.dt_training.bit_length as usize;
        let mut temp0 = vec![0u8; bit_length];
        let diff = abs(bit_length as isize - temp.len() as isize);
        for i in 0..diff {
            temp.push(0);
        }
        // hard-coded for two-party
        for i in 0..2 {
            if i == ctx.party_id {
                input_shares.push(&temp);
            } else {
                input_shares.push(&temp0);
            }
        }
        //initY in Java Lynx
        for i in 0..bit_length {
            let y = input_shares[0][i] + input_shares[1][i];
            y_shares.push(mod_floor(y, BINARY_PRIME as u8));
        }
        x_shares.push(y_shares[0]);

        //bit_multiplication in Java Lynx
        let first_c_share = multiplication_byte(input_shares[0][0], input_shares[1][0], ctx);
        increment_current_share_index(ctx, ShareType::BinaryShare);
        c_shares.push(mod_floor(first_c_share, BINARY_PRIME as u8));

        //computeDShares in Java Lynx
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
                let mut batch_mul_result = batch_multiplication_byte(&input_shares[0][i..to_index].to_vec(), &input_shares[1][i..to_index].to_vec(), &mut ctx_copied);
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
                d_shares.push(mod_floor(item + ctx.asymmetric_bit, BINARY_PRIME as u8));
                global_index += 1;
            }
        }

        for i in 0..bit_length {
            //computeVariables
            let e_result = multiplication_byte(y_shares[i], c_shares[i - 1], ctx) + ctx.asymmetric_bit;
            e_shares[i] = mod_floor(e_result, BINARY_PRIME as u8);
            let x_result = y_shares[i] + c_shares[i - 1];
            x_shares[i] = mod_floor(x_result, BINARY_PRIME as u8);
        }
        x_shares
    }


    pub fn multiplication_byte(x: u8, y: u8, ctx: &mut ComputingParty) -> u8 {
        let mut diff_list = Vec::new();
        let ti_share_index = *(ctx.dt_shares.current_binary_index.lock().unwrap());
        let ti_share_triple = ctx.dt_shares.binary_triples[ti_share_index];
        diff_list.push(mod_floor((Wrapping(x) - Wrapping(ti_share_triple.0)).0, BINARY_PRIME as u8));
        diff_list.push(mod_floor((Wrapping(y) - Wrapping(ti_share_triple.1)).0, BINARY_PRIME as u8));
        increment_current_share_index(ctx, ShareType::BinaryShare);

        let mut in_stream = ctx.in_stream.try_clone()
            .expect("failed cloning tcp o_stream");

        let mut o_stream = ctx.o_stream.try_clone()
            .expect("failed cloning tcp o_stream");
        o_stream.write((serde_json::to_string(&diff_list).unwrap() + "\n").as_bytes());

        let mut reader = BufReader::new(in_stream);
        let mut diff_list_message = String::new();
        reader.read_line(&mut diff_list_message).expect("multiplication_byte: fail to read diff list message");
        let diff_list: Vec<u8> = serde_json::from_str(&diff_list_message).unwrap();
        let mut d: u8 = 0;
        let mut e: u8 = 0;
        d = (Wrapping(d) + Wrapping(diff_list[0])).0;
        e = (Wrapping(e) + Wrapping(diff_list[1])).0;
        d = mod_floor((Wrapping(x) - Wrapping(ti_share_triple.0 as u64) + Wrapping(d)).0, BINARY_PRIME as u8);
        e = mod_floor((Wrapping(y) - Wrapping(ti_share_triple.1 as u64) + Wrapping(e)).0, BINARY_PRIME as u8);

        let mut result: u8 = (Wrapping(ti_share_triple.2 as u8) + (Wrapping(d) * Wrapping(ti_share_triple.1 as u8)) + (Wrapping(ti_share_triple.0 as u8) * Wrapping(e))
            + (Wrapping(d) * Wrapping(e) * Wrapping(ctx.asymmetric_bit as u8))).0;
        result = mod_floor(result, BINARY_PRIME as u8);
        result
    }

    pub fn batch_multiplication_byte(x_list: &Vec<u8>, y_list: &Vec<u8>, ctx: &mut ComputingParty) -> Vec<u8> {
        let batch_size = x_list.len();
        let mut diff_list = Vec::new();
        let mut output = Vec::new();

        for i in 0..batch_size {
            let mut new_row = Vec::new();
            let ti_share_index = *(ctx.dt_shares.current_binary_index.lock().unwrap());
            let ti_share_triple = ctx.dt_shares.binary_triples[ti_share_index];
            new_row.push(mod_floor((Wrapping(x) - Wrapping(ti_share_triple.0)).0, BINARY_PRIME as u8));
            new_row.push(mod_floor((Wrapping(y) - Wrapping(ti_share_triple.1)).0, BINARY_PRIME as u8));
            diff_list.push(new_row);
            increment_current_share_index(ctx, ShareType::BinaryShare);
        }

        let mut in_stream = ctx.in_stream.try_clone()
            .expect("failed cloning tcp o_stream");

        let mut o_stream = ctx.o_stream.try_clone()
            .expect("failed cloning tcp o_stream");
        o_stream.write((serde_json::to_string(&diff_list).unwrap() + "\n").as_bytes());

        let mut reader = BufReader::new(in_stream);
        let mut diff_list_message = String::new();
        reader.read_line(&mut diff_list_message).expect("multiplication_byte: fail to read diff list message");
        let diff_list: Vec<Vec<u8>> = serde_json::from_str(&diff_list_message).unwrap();
        for item in diff_list {
            let mut d: u8 = 0;
            let mut e: u8 = 0;
            d = (Wrapping(d) + Wrapping(item[0])).0;
            e = (Wrapping(e) + Wrapping(item[1])).0;
            d = mod_floor((Wrapping(x) - Wrapping(ti_share_triple.0 as u64) + Wrapping(d)).0, BINARY_PRIME as u8);
            e = mod_floor((Wrapping(y) - Wrapping(ti_share_triple.1 as u64) + Wrapping(e)).0, BINARY_PRIME as u8);

            let mut result: u8 = (Wrapping(ti_share_triple.2 as u8) + (Wrapping(d) * Wrapping(ti_share_triple.1 as u8)) + (Wrapping(ti_share_triple.0 as u8) * Wrapping(e))
                + (Wrapping(d) * Wrapping(e) * Wrapping(ctx.asymmetric_bit as u8))).0;
            result = mod_floor(result, BINARY_PRIME as u8);
            output.push(result);
        }
        output
    }

    pub fn arg_max(bit_shares: Vec<Vec<u8>>, ctx: &mut ComputingParty) -> Vec<u64> {
        let number_count = bit_shares.len();
        let mut bit_length = 0;
        bit_shares.iter().map(|x| bit_length = max(bit_length, x.len()));
        let mut w_intermediate = HashMap::new();
        for i in 0..number_count {
            w_intermediate.insert(i, Vec::new());
        }
        let mut result = Vec::new();
        if number_count == 1 {
            result.push(1 as u64);
        } else {
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
                            let mut output_map = &*(output_map.lock().unwrap());
                            output_map.insert(key, comparison_result);
                        });
                    }
                }
            }

            let output_map = &*(output_map.lock().unwrap());
            for i in 0..number_count * (number_count - 1) {
                let comparison = output_map.get(&i).unwrap();
                let key = i / (number_count - 1);
                w_intermediate.get(&key).unwrap().push(comparison);
            }

            let mut output_map = Arc::new(Mutex::new((HashMap::new())));
            //multi-threaded parallel multiplication
            for i in 0..number_count {
                let mut vec = Vec::new();
                for item in w_intermediate.get(&i).unwrap() {
                    vec.push(item as u64);
                }
                let mut output_map = Arc::clone(&output_map);
                let mut ctx_copied = ctx.clone();
                thread_pool.execute(move || {
                    let multi_result = parallel_multiplication(&vec, &mut ctx_copied);
                    let mut output_map = &*(output_map.lock().unwrap());
                    output_map.insert(i, multi_result);
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
        let mut e_shares = [0u8; bit_length];
        let mut d_shares = [0u8; bit_length];
        let mut c_shares = [0u8; bit_length];
        let mut multiplication_e = [0u8; bit_length];
        let mut w = -1;
        //computeEShares in Java Lynx
        for i in 0..bit_length {
            let e_share = x_list[i] + y_list[i] + ctx.asymmetric_bit;
            e_shares[i] = mod_floor(e_share, BINARY_PRIME as u8);
        }
        let thread_pool = ThreadPool::new(ctx.thread_count);
        //compute D shares
        thread_pool.execute(move || {
            let (batch_count, output_map) = multi_thread_batch_mul_byte(x_list, y_list, ctx, bit_length);
            let mut global_index = 0;
            for i in 0..batch_count {
                let product_result = output_map.get(&i).unwrap();
                for item in product_result {
                    let local_diff = y_list[global_index] - *item;
                    d_shares[global_index] = mod_floor(local_diff, BINARY_PRIME as u8);
                    global_index += 1;
                }
            }
        });
        //compute multiplication E parallel
        thread_pool.execute(move || {
            let mut main_index = bit_length - 1;
            main_index -= 1;
            multiplication_e[main_index] = e_shares[bit_length - 1];
            let mut temp_mul_e = vec![0u8; bit_length];
            while temp_mul_e.len() > 1 {
                let inner_pool = ThreadPool::new(ctx.thread_count);
                let mut i = 0;
                let mut batch_count = 0;
                let mut output_map = Arc::new(Mutex::new(HashMap::new()));
                while i < temp_mul_e.len() - 1 {
                    let mut output_map = Arc::clone(&output_map);
                    let to_index = min(i + ctx.batch_size, temp_mul_e.len());
                    let mut ctx_copied = ctx.clone();
                    let mut temp_mul_e = temp_mul_e.clone();
                    inner_pool.execute(move || {
                        let mut batch_mul_result = batch_multiplication_byte(&temp_mul_e[i..to_index - 1].to_vec(), &temp_mul_e[i + 1..to_index].to_vec(), &mut ctx_copied);
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
        let mut output_map = Arc::new(Mutex::new(HashMap::new()));
        while i < bit_length - 1 {
            let mut output_map = Arc::clone(&output_map);
            let to_index = min(i + ctx.batch_size, bit_length - 1);
            let mut ctx_copied = ctx.clone();
            let mut multiplication_e = multiplication_e.to_vec().clone();
            let mut d_shares = d_shares.to_vec().clone();
            thread_pool.execute(move || {
                let mut batch_mul_result = batch_multiplication_byte(&multiplication_e[i + 1..to_index + 1].to_vec(), &d_shares[i..to_index].to_vec(), &mut ctx_copied);
                let mut output_map = output_map.lock().unwrap();
                (*output_map).insert(batch_count, batch_mul_result);
            });
            i = to_index;
            batch_count += 1;
        }
        inner_pool.join();
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
        w = ctx.asymmetric_bit;
        for i in 0..bit_length {
            w += c_shares[i];
            w = mod_floor(w, BINARY_PRIME as u8);
        }
        w
    }

    pub fn parallel_multiplication(row: &Vec<u64>, ctx: &mut ComputingParty) -> u64 {
        let mut products = row.clone();
        while products.len() > 1 {
            let size = products.len();
            let mut push = -1;
            let to_index1 = size / 2;
            let to_index2 = size;
            if size % 2 == 1 {
                to_index2 -= 1;
                push = products[size - 1];
            }
            let thread_pool = ThreadPool::new(ctx.thread_count);
            let i1 = 0;
            let i2 = to_index1;
            while i1 < to_index1 && i2 < to_index2 {
                let temp_index1 = min(i1 + ctx.batch_size, to_index1);
                let temp_index2 = min(i2 + ctx.batch_size, to_index2);
            }
        }
        products[0]
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
            thread_pool.execute(move || {
                let multi_result = batch_multiply_bigint(&x_list[i..to_index].to_vec(), &y_list[i..to_index].to_vec(), &mut ctx_copied);
                let mut output_map = &*(output_map.lock().unwrap());
                output_map.insert(batch_count, multi_result);
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

    pub fn multiplication_bigint(x: &BigUint, y: &BigUint, ctx: &mut ComputingParty) -> BigUint {
        let ti_shares = &ctx.dt_shares.additive_bigint_triples;
        let mut current_index = &*(ctx.dt_shares.current_additive_bigint_index.lock().unwrap());
        let share = &ti_shares[*current_index];
        *current_index += 1;

        let mut diff_list = Vec::new();
        diff_list.push(big_uint_subtract(x, &share.0, &ctx.dt_training.big_int_prime));
        diff_list.push(big_uint_subtract(y, &share.1, &ctx.dt_training.big_int_prime));

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
        let mut d = BigUint::zero();
        let mut e = BigUint::zero();
        let prime = &ctx.dt_training.big_int_prime;
        d = d.add(&diff_list[0]).mod_floor(prime);
        e = e.add(&diff_list[1]).mod_floor(prime);
        let ti_share_index = *(ctx.dt_shares.current_additive_bigint_index.lock().unwrap());
        let share = &ctx.dt_shares.additive_bigint_triples[ti_share_index];
        ti_share_index += 1;
        d = big_uint_subtract(x, &share.0, prime).add(&d).mod_floor(&ctx.dt_training.big_int_prime);
        e = big_uint_subtract(y, &share.1, prime).add(&e).mod_floor(&ctx.dt_training.big_int_prime);

        let product = share.2.add(&d.mul(&share.1).mod_floor(prime)).mod_floor(prime)
            .add(&e.mul(&share.0).mod_floor(prime)).mod_floor(prime)
            .add(&d.mul(&e).mod_floor(prime).mul(BigUint::from(ctx.asymmetric_bit)).mod_floor(prime)).mod_floor(prime);
        product
    }

    pub fn parallel_multiplication_big_integer(row: &Vec<BigUint>, ctx: &mut ComputingParty) -> BigUint {
        let mut products = big_uint_vec_clone(row);
        let thread_pool = ThreadPool::new(ctx.thread_count);
        while products.len() > 1 {
            let size = products.len();
            let mut push = BigUint::from_i32(-1).unwrap();
            let to_index1 = size / 2;
            let to_index2 = size;
            if size % 2 == 1 {
                to_index2 -= 1;
                push = BigUint::from(&products[size - 1]);
            }
            let mut i1 = 0;
            let mut i2 = to_index1;
            let mut output_map = Arc::new(Mutex::new(HashMap::new()));
            let mut batch_count = 0;
            while i1 < to_index1 && i2 < to_index2 {
                let temp_index1 = min(i1 + ctx.batch_size, to_index1);
                let temp_index2 = min(i2 + ctx.batch_size, to_index2);
                let mut output_map = Arc::clone(&output_map);
                let mut ctx_copied = ctx.clone();
                thread_pool.execute(move || {
                    let multi_result = batch_multiply_bigint(&products[i1..temp_index1].to_vec(), &products[i2..temp_index2].to_vec(), &mut ctx_copied);
                    let mut output_map = &*(output_map.lock().unwrap());
                    output_map.insert(batch_count, multi_result);
                });
                i1 = temp_index1;
                i2 = temp_index2;
                batch_count += 1;
            }
            let mut new_products = Vec::new();
            let mut output_map = &*(output_map.lock().unwrap());
            for i in 0..batch_count {
                let multi_result = output_map.get(&i).unwrap();
                new_products.append(&multi_result);
            }
            products.clear();
            products = big_uint_vec_clone(&new_products);
            if !push.eq(&BigUint::from_i32(-1).unwrap()) {
                products.push(push);
            }
        }
        big_uint_clone(&products[0])
    }

    pub fn equality_big_integer() {}


    fn multi_thread_batch_mul_byte(x_list: &Vec<u8>, y_list: &Vec<u8>, ctx: &mut ComputingParty, bit_length: usize) -> (u32, HashMap<u32, Vec<u8>>) {
        let inner_pool = ThreadPool::new(ctx.thread_count);
        let mut i = 0;
        let mut batch_count = 0;
        let mut output_map = Arc::new(Mutex::new(HashMap::new()));
        while i < bit_length {
            let mut output_map = Arc::clone(&output_map);
            let to_index = min(i + ctx.batch_size, bit_length);
            let mut ctx_copied = ctx.clone();
            let mut x_list = x_list.clone();
            let mut y_list = y_list.clone();
            inner_pool.execute(move || {
                let mut batch_mul_result = batch_multiplication_byte(&x_list[i..to_index].to_vec(), &y_list[i..to_index].to_vec(), &mut ctx_copied);
                let mut output_map = output_map.lock().unwrap();
                (*output_map).insert(batch_count, batch_mul_result);
            });
            i = to_index;
            batch_count += 1;
        }
        inner_pool.join();
        let output_map = *(output_map.lock().unwrap());
        (batch_count, output_map)
    }
}
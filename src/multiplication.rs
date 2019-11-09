pub mod multiplication{
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

    union Xbuffer {
        u64_buf: [u64; U64S_PER_TX],
        u8_buf: [u8; U8S_PER_TX],
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

//    pub fn bit_decomposition(input: u64, ctx: &mut ComputingParty) -> Vec<u8> {
//        let mut input_shares = Vec::new();
//        let mut d_shares = Vec::new();
//        let mut e_shares = Vec::new();
//        let mut c_shares = Vec::new();
//        let mut y_shares = Vec::new();
//        let mut x_shares = Vec::new();
//        let binary_str = format!("{:b}", input);
//        let input_binary_str_vec: Vec<&str> = binary_str.split("").collect();
//        let mut temp: Vec<u8> = Vec::new();
//        for item in input_binary_str_vec {
//            temp.push(item.parse::<u8>().unwrap());
//        }
//        let bit_length = ctx.dt_training.bit_length as usize;
//        let mut temp0 = vec![0u8; bit_length];
//        let diff = abs(bit_length as isize - temp.len() as isize);
//        for i in 0..diff {
//            temp.push(0);
//        }
//        // hard-coded for two-party
//
//        for i in 0..2 {
//            let mut temp = temp.clone();
//            let mut temp0 = temp0.clone();
//            if i == ctx.party_id {
//                input_shares.push(temp);
//            } else {
//                input_shares.push(temp0);
//            }
//        }
//        //initY in Java Lynx
//        for i in 0..bit_length {
//            let y = input_shares[0][i] + input_shares[1][i];
//            y_shares.push(mod_floor(y, BINARY_PRIME as u8));
//        }
//        x_shares.push(y_shares[0]);
//
//        //bit_multiplication in Java Lynx
//        let first_c_share = multiplication_byte(input_shares[0][0], input_shares[1][0], ctx);
//        increment_current_share_index(ctx, ShareType::BinaryShare);
//        c_shares.push(mod_floor(first_c_share, BINARY_PRIME as u8));
//
//        //computeDShares in Java Lynx
//        let thread_pool = ThreadPool::new(ctx.thread_count);
//        let mut i = 0;
//        let mut output_map = Arc::new(Mutex::new(HashMap::new()));
//        let mut batch_count = 0;
//        while i < bit_length {
//            let mut output_map = Arc::clone(&output_map);
//            let to_index = min(i + ctx.batch_size, bit_length);
//            let mut ctx_copied = ctx.clone();
//            let mut input_shares = input_shares.clone();
//            thread_pool.execute(move || {
//                let mut batch_mul_result = batch_multiplication_byte(&input_shares[0][i..to_index].to_vec(), &input_shares[1][i..to_index].to_vec(), &mut ctx_copied);
//                let mut output_map = output_map.lock().unwrap();
//                (*output_map).insert(batch_count, batch_mul_result);
//            });
//            i = to_index;
//            batch_count += 1;
//        }
//        thread_pool.join();
//        let output_map = &(*(output_map.lock().unwrap()));
//        let mut global_index = 0;
//        for i in 0..batch_count {
//            let batch_result = output_map.get(&i).unwrap();
//            for item in batch_result.iter() {
//                d_shares.push(mod_floor(item + ctx.asymmetric_bit, BINARY_PRIME as u8));
//                global_index += 1;
//            }
//        }
//
//        for i in 0..bit_length {
//            //computeVariables
//            let e_result = multiplication_byte(y_shares[i], c_shares[i - 1], ctx) + ctx.asymmetric_bit;
//            e_shares[i] = mod_floor(e_result, BINARY_PRIME as u8);
//            let x_result = y_shares[i] + c_shares[i - 1];
//            x_shares[i] = mod_floor(x_result, BINARY_PRIME as u8);
//        }
//        x_shares
//    }
//
//
//    pub fn multiplication_byte(x: u8, y: u8, ctx: &mut ComputingParty) -> u8 {
//        let mut diff_list = Vec::new();
//        let ti_share_index = *(ctx.dt_shares.current_binary_index.lock().unwrap());
//        let ti_share_triple = ctx.dt_shares.binary_triples[ti_share_index];
//        diff_list.push(mod_floor((Wrapping(x) - Wrapping(ti_share_triple.0)).0, BINARY_PRIME as u8));
//        diff_list.push(mod_floor((Wrapping(y) - Wrapping(ti_share_triple.1)).0, BINARY_PRIME as u8));
//        increment_current_share_index(ctx, ShareType::BinaryShare);
//
//        let mut in_stream = ctx.in_stream.try_clone()
//            .expect("failed cloning tcp o_stream");
//
//        let mut o_stream = ctx.o_stream.try_clone()
//            .expect("failed cloning tcp o_stream");
//        o_stream.write((serde_json::to_string(&diff_list).unwrap() + "\n").as_bytes());
//
//        let mut reader = BufReader::new(in_stream);
//        let mut diff_list_message = String::new();
//        reader.read_line(&mut diff_list_message).expect("multiplication_byte: fail to read diff list message");
//        let diff_list: Vec<u8> = serde_json::from_str(&diff_list_message).unwrap();
//        let mut d: u8 = 0;
//        let mut e: u8 = 0;
//        d = (Wrapping(d) + Wrapping(diff_list[0])).0;
//        e = (Wrapping(e) + Wrapping(diff_list[1])).0;
//        d = mod_floor((Wrapping(x) - Wrapping(ti_share_triple.0 as u8) + Wrapping(d)).0, BINARY_PRIME as u8);
//        e = mod_floor((Wrapping(y) - Wrapping(ti_share_triple.1 as u8) + Wrapping(e)).0, BINARY_PRIME as u8);
//
//        let mut result: u8 = (Wrapping(ti_share_triple.2 as u8) + (Wrapping(d) * Wrapping(ti_share_triple.1 as u8)) + (Wrapping(ti_share_triple.0 as u8) * Wrapping(e))
//            + (Wrapping(d) * Wrapping(e) * Wrapping(ctx.asymmetric_bit as u8))).0;
//        result = mod_floor(result, BINARY_PRIME as u8);
//        result
//    }
//
//    pub fn batch_multiplication_byte(x_list: &Vec<u8>, y_list: &Vec<u8>, ctx: &mut ComputingParty) -> Vec<u8> {
//        let batch_size = x_list.len();
//        let mut diff_list = Vec::new();
//        let mut output = Vec::new();
//
//        let mut ti_shares = Vec::new();
//        for i in 0..batch_size {
//            let mut new_row = Vec::new();
//            let ti_share_index = *(ctx.dt_shares.current_binary_index.lock().unwrap());
//            let ti_share_triple = ctx.dt_shares.binary_triples[ti_share_index];
//            ti_shares.push(ti_share_triple);
//            new_row.push(mod_floor((Wrapping(x_list[i]) - Wrapping(ti_share_triple.0)).0, BINARY_PRIME as u8));
//            new_row.push(mod_floor((Wrapping(y_list[i]) - Wrapping(ti_share_triple.1)).0, BINARY_PRIME as u8));
//            diff_list.push(new_row);
//            increment_current_share_index(ctx, ShareType::BinaryShare);
//        }
//
//        let mut in_stream = ctx.in_stream.try_clone()
//            .expect("failed cloning tcp o_stream");
//
//        let mut o_stream = ctx.o_stream.try_clone()
//            .expect("failed cloning tcp o_stream");
//        o_stream.write((serde_json::to_string(&diff_list).unwrap() + "\n").as_bytes());
//
//        let mut reader = BufReader::new(in_stream);
//        let mut diff_list_message = String::new();
//        reader.read_line(&mut diff_list_message).expect("multiplication_byte: fail to read diff list message");
//        let diff_list: Vec<Vec<u8>> = serde_json::from_str(&diff_list_message).unwrap();
//
//        let mut d_list = vec![0u8; batch_size];
//        let mut e_list = vec![0u8; batch_size];
//
//        for i in 0..batch_size {
//            d_list[i] = (Wrapping(d_list[i]) + Wrapping(diff_list[i][0])).0;
//            e_list[i] = (Wrapping(e_list[i]) + Wrapping(diff_list[i][1])).0;
//        }
//
//        for i in 0..batch_size {
//            let ti_share_triple = ti_shares[i];
//            let d = mod_floor((Wrapping(x_list[i]) - Wrapping(ti_share_triple.0 as u8) + Wrapping(d_list[i])).0, BINARY_PRIME as u8);
//            let e = mod_floor((Wrapping(y_list[i]) - Wrapping(ti_share_triple.1 as u8) + Wrapping(e_list[i])).0, BINARY_PRIME as u8);
//            let mut result: u8 = (Wrapping(ti_share_triple.2 as u8) + (Wrapping(d) * Wrapping(ti_share_triple.1 as u8)) + (Wrapping(ti_share_triple.0 as u8) * Wrapping(e))
//                + (Wrapping(d) * Wrapping(e) * Wrapping(ctx.asymmetric_bit as u8))).0;
//            result = mod_floor(result, BINARY_PRIME as u8);
//            output.push(result);
//        }
//
//        output
//    }
//
//    pub fn parallel_multiplication(row: &Vec<u8>, ctx: &mut ComputingParty) -> u8 {
//        let mut products = row.clone();
//        let thread_pool = ThreadPool::new(ctx.thread_count);
//
//        while products.len() > 1 {
//            let size = products.len();
//            let mut push = -1;
//            let to_index1 = size / 2;
//            let to_index2 = size;
//            if size % 2 == 1 {
//                to_index2 -= 1;
//                push = products[size - 1] as i8;
//            }
//            let mut i1 = 0;
//            let mut i2 = to_index1;
//            let mut batch_count = 0;
//            let mut output_map = Arc::new(Mutex::new(HashMap::new()));
//
//            while i1 < to_index1 && i2 < to_index2 {
//                let temp_index1 = min(i1 + ctx.batch_size, to_index1);
//                let temp_index2 = min(i2 + ctx.batch_size, to_index2);
//                let mut products_copied = products.clone();
//                let mut output_map = Arc::clone(&output_map);
//                let mut ctx_copied = ctx.clone();
//
//                thread_pool.execute(move || {
//                    let mut batch_mul_result = batch_multiplication_byte(&products_copied[i1..temp_index1].to_vec(), &products_copied[i2..temp_index2].to_vec(), &mut ctx_copied);
//                    let mut output_map = output_map.lock().unwrap();
//                    (*output_map).insert(batch_count, batch_mul_result);
//                });
//                batch_count += 1;
//                i1 = temp_index1;
//                i2 = temp_index2;
//            }
//            thread_pool.join();
//            let mut new_products = Vec::new();
//            let mut output_map = &*(output_map.lock().unwrap());
//            for i in 0..batch_count {
//                let mut multi_result = output_map.get(&i).unwrap();
//                new_products.append(&mut multi_result);
//            }
//            products.clear();
//            products = new_products.clone();
//            if push != -1 {
//                products.push(push as u8);
//            }
//        }
//        products[0]
//    }
//
//    pub fn multiplication_bigint(x: &BigUint, y: &BigUint, ctx: &mut ComputingParty) -> BigUint {
//        let ti_shares = &ctx.dt_shares.additive_bigint_triples;
//        let mut current_index = &*(ctx.dt_shares.current_additive_bigint_index.lock().unwrap());
//        let share = &ti_shares[*current_index];
//        *current_index += 1;
//
//        let mut diff_list = Vec::new();
//        diff_list.push(big_uint_subtract(x, &share.0, &ctx.dt_training.big_int_prime));
//        diff_list.push(big_uint_subtract(y, &share.1, &ctx.dt_training.big_int_prime));
//
//        let mut in_stream = ctx.in_stream.try_clone()
//            .expect("failed cloning tcp o_stream");
//
//        let mut o_stream = ctx.o_stream.try_clone()
//            .expect("failed cloning tcp o_stream");
//
//        let mut message = serialize_biguint_vec(diff_list);
//        o_stream.write((message + "\n").as_bytes());
//
//        let mut reader = BufReader::new(in_stream);
//        let mut diff_list_message = String::new();
//        reader.read_line(&mut diff_list_message).expect("fail to read diff list message");
//
//        let mut diff_list = deserialize_biguint_vec(diff_list_message);
//        let mut d = BigUint::zero();
//        let mut e = BigUint::zero();
//        let prime = &ctx.dt_training.big_int_prime;
//        d = d.add(&diff_list[0]).mod_floor(prime);
//        e = e.add(&diff_list[1]).mod_floor(prime);
//        let ti_share_index = *(ctx.dt_shares.current_additive_bigint_index.lock().unwrap());
//        let share = &ctx.dt_shares.additive_bigint_triples[ti_share_index];
//        ti_share_index += 1;
//        d = big_uint_subtract(x, &share.0, prime).add(&d).mod_floor(&ctx.dt_training.big_int_prime);
//        e = big_uint_subtract(y, &share.1, prime).add(&e).mod_floor(&ctx.dt_training.big_int_prime);
//
//        let product = share.2.add(&d.mul(&share.1).mod_floor(prime)).mod_floor(prime)
//            .add(&e.mul(&share.0).mod_floor(prime)).mod_floor(prime)
//            .add(&d.mul(&e).mod_floor(prime).mul(BigUint::from(ctx.asymmetric_bit)).mod_floor(prime)).mod_floor(prime);
//        product
//    }
//
//    pub fn parallel_multiplication_big_integer(row: &Vec<BigUint>, ctx: &mut ComputingParty) -> BigUint {
//        let mut products = big_uint_vec_clone(row);
//        let thread_pool = ThreadPool::new(ctx.thread_count);
//        while products.len() > 1 {
//            let size = products.len();
//            let mut push = BigInt::from_i32(-1).unwrap();
//            let mut to_index1 = size / 2;
//            let mut to_index2 = size;
//            if size % 2 == 1 {
//                to_index2 -= 1;
//                push = products[size - 1].to_bigint().unwrap();
//            }
//            let mut i1 = 0;
//            let mut i2 = to_index1;
//            let mut output_map = Arc::new(Mutex::new(HashMap::new()));
//            let mut batch_count = 0;
//            while i1 < to_index1 && i2 < to_index2 {
//                let temp_index1 = min(i1 + ctx.batch_size, to_index1);
//                let temp_index2 = min(i2 + ctx.batch_size, to_index2);
//                let mut output_map = Arc::clone(&output_map);
//                let mut ctx_copied = ctx.clone();
//                let mut products_slice = big_uint_vec_clone(&products[i1..temp_index1].to_vec());
//                thread_pool.execute(move || {
//                    let multi_result = batch_multiply_bigint(&products_slice, &products_slice, &mut ctx_copied);
//                    let mut output_map = output_map.lock().unwrap();
//                    (*output_map).insert(batch_count, multi_result);
//                });
//                i1 = temp_index1;
//                i2 = temp_index2;
//                batch_count += 1;
//            }
//            let mut new_products = Vec::new();
//            let mut output_map = &*(output_map.lock().unwrap());
//            for i in 0..batch_count {
//                let mut multi_result = *output_map.get(&i).unwrap();
//                new_products.append(&mut multi_result);
//            }
//            products.clear();
//            products = big_uint_vec_clone(&new_products);
//            if !push.eq(&BigInt::from_i32(-1).unwrap()) {
//                products.push(push.to_biguint().unwrap());
//            }
//        }
//        big_uint_clone(&products[0])
//    }
//
//    pub fn multi_thread_batch_mul_byte(x_list: &Vec<u8>, y_list: &Vec<u8>, ctx: &mut ComputingParty, bit_length: usize) -> (u32, HashMap<u32, Vec<u8>>) {
//        let inner_pool = ThreadPool::new(ctx.thread_count);
//        let mut i = 0;
//        let mut batch_count = 0;
//        let mut output_map = Arc::new(Mutex::new(HashMap::new()));
//        while i < bit_length {
//            let mut output_map = Arc::clone(&output_map);
//            let to_index = min(i + ctx.batch_size, bit_length);
//            let mut ctx_copied = ctx.clone();
//            let mut x_list = x_list.clone();
//            let mut y_list = y_list.clone();
//            inner_pool.execute(move || {
//                let mut batch_mul_result = batch_multiplication_byte(&x_list[i..to_index].to_vec(), &y_list[i..to_index].to_vec(), &mut ctx_copied);
//                let mut output_map = output_map.lock().unwrap();
//                (*output_map).insert(batch_count, batch_mul_result);
//            });
//            i = to_index;
//            batch_count += 1;
//        }
//        inner_pool.join();
//        let output_map = *(output_map.lock().unwrap());
//        (batch_count, output_map)
//    }
}
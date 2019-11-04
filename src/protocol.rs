pub mod protocol {
    /**
    ** @author Davis.R, James.S
    **/
    use std::num::Wrapping;
    use crate::computing_party::computing_party::ComputingParty;
    use crate::constants::constants::{BATCH_SIZE, U8S_PER_TX, BUF_SIZE, U64S_PER_TX};
    use std::io::{Read, Write};
    use threadpool::ThreadPool;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::cmp::min;
    use num::integer::*;

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
        for i in 0..batch_count{
            let batch_result=output_map.get(&i).unwrap();
            for item in batch_result.iter(){
                output.push(Wrapping(mod_floor(x_list[global_index].0+y_list[global_index].0-(constant_multiplier*item.0),ctx.dt_training.dataset_size_prime)));
                global_index+=1;
            }
        }
        output
    }

    pub fn change_binary_to_decimal_field(binary_numbers:&Vec<Wrapping<u64>>,ctx:&mut ComputingParty) -> Vec<Wrapping<u64>> {
        let mut dummy_list = vec![Wrapping(0u64);binary_numbers.len()];
        let mut output = Vec::new();
        if ctx.asymmetric_bit==1 {
            output = or_xor(binary_numbers,&dummy_list,ctx,2);
        }else{
            output = or_xor(&dummy_list,binary_numbers,ctx,2);
        }
        output
    }

    /* computed the dp modulo 2^64 of two vectors with pre/post truncation options */
//    pub fn dot_product(x_list: &Vec<Wrapping<u64>>,
//                       y_list: &Vec<Wrapping<u64>>,
//                       ctx: &mut ComputingParty,
//                       decimal_precision: u32,
//                       truncate: bool,
//                       pretruncate: bool) -> Wrapping<u64> {
//
//        //println!("entering dot product");
//        let z_list = batch_multiply(x_list, y_list, ctx);
//
//        if !truncate {
//            return z_list.iter().sum();
//        }
//
//        if !pretruncate {
//            return utility::truncate_local(
//                z_list.iter().sum(), decimal_precision, (*ctx).asymmetric_bit,
//            );
//        }
//
//
//        let mut z_trunc_list = vec![Wrapping(0); z_list.len()];
//        for i in 0..z_list.len() {
//            z_trunc_list[i] = utility::truncate_local(
//                z_list[i], decimal_precision, (*ctx).asymmetric_bit,
//            );
//        }
//        z_trunc_list.iter().sum()
//    }

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
}
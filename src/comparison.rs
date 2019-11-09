pub mod comparison{
    use crate::computing_party::computing_party::ComputingParty;
    use threadpool::ThreadPool;
    use crate::constants::constants::BINARY_PRIME;
    use std::cmp::{max, min};
    use std::sync::{Arc, Mutex};
//    use crate::multiplication::multiplication::{multi_thread_batch_mul_byte, batch_multiplication_byte};
//    use std::collections::HashMap;

//    pub fn comparison(x_list: &Vec<u8>, y_list: &Vec<u8>, ctx: &mut ComputingParty) -> u8 {
//        let bit_length = max(x_list.len(), y_list.len());
//        let prime = BINARY_PRIME;
//        let ti_shares = &ctx.dt_shares.binary_triples;
//        let ti_shares_start_index = *(ctx.dt_shares.current_binary_index.lock().unwrap());
//        let mut e_shares = vec![0u8; bit_length];
//        let mut d_shares = vec![0u8; bit_length];
//        let mut c_shares = vec![0u8; bit_length];
//        let mut multiplication_e = vec![0u8; bit_length];
//        let mut w = -1;
//        //computeEShares in Java Lynx
//
//        //compute D shares
//        let thread_pool = ThreadPool::new(ctx.thread_count);
//        let mut x_list_copied = x_list.clone();
//        let mut y_list_copied = y_list.clone();
//        let mut ctx_copied = ctx.clone();
//        let mut d_shares_wrapper = Arc::new(Mutex::new(d_shares));
//        let mut d_shares_copied = Arc::clone(&d_shares_wrapper);
//        thread_pool.execute(move || {
//            let (batch_count, output_map) = multi_thread_batch_mul_byte(&x_list_copied, &y_list_copied, &mut ctx_copied, bit_length);
//            let mut global_index = 0;
//            let mut d_shares_copied = d_shares_copied.lock().unwrap();
//            for i in 0..batch_count {
//                let product_result = output_map.get(&i).unwrap();
//                for item in product_result {
//                    let local_diff = y_list_copied[global_index] - *item;
//
//                    (*d_shares_copied)[global_index] = mod_floor(local_diff, BINARY_PRIME as u8);
//                    global_index += 1;
//                }
//            }
//        });
//        //compute multiplication E parallel
//        let mut ctx_copied = ctx.clone();
//        thread_pool.execute(move || {
//            let mut main_index = bit_length - 1;
//            main_index -= 1;
//            multiplication_e[main_index] = e_shares[bit_length - 1];
//            let mut temp_mul_e = vec![0u8; bit_length];
//            while temp_mul_e.len() > 1 {
//                let inner_pool = ThreadPool::new(ctx_copied.thread_count);
//                let mut i = 0;
//                let mut batch_count = 0;
//                let mut output_map = Arc::new(Mutex::new(HashMap::new()));
//                while i < temp_mul_e.len() - 1 {
//                    let mut output_map = Arc::clone(&output_map);
//                    let to_index = min(i + ctx_copied.batch_size, temp_mul_e.len());
//                    let mut ctx_copied_inner = ctx_copied.clone();
//                    let mut temp_mul_e = temp_mul_e.clone();
//                    inner_pool.execute(move || {
//                        let mut batch_mul_result = batch_multiplication_byte(&temp_mul_e[i..to_index - 1].to_vec(), &temp_mul_e[i + 1..to_index].to_vec(), &mut ctx_copied_inner);
//                        let mut output_map = output_map.lock().unwrap();
//                        (*output_map).insert(batch_count, batch_mul_result);
//                    });
//                    i += to_index - 1;
//                    batch_count += 1;
//                }
//                inner_pool.join();
//                let output_map = &*(output_map.lock().unwrap());
//                let mut products = Vec::new();
//                for i in 0..batch_count {
//                    let product_result = output_map.get(&i).unwrap();
//                    for item in product_result {
//                        products.push(*item);
//                    }
//                }
//                temp_mul_e.clear();
//                temp_mul_e = products;
//                main_index -= 1;
//                multiplication_e[main_index] = *temp_mul_e.last().unwrap();
//            }
//            multiplication_e[0] = 0;
//        });
//        thread_pool.join();
//        //compute c shares
//        let mut i = 0;
//        let mut batch_count = 0;
//        let d_shares = &*(d_shares_wrapper.lock().unwrap());
//        let mut output_map = Arc::new(Mutex::new(HashMap::new()));
//        while i < bit_length - 1 {
//            let mut output_map = Arc::clone(&output_map);
//            let to_index = min(i + ctx.batch_size, bit_length - 1);
//            let mut ctx_copied = ctx.clone();
//            let mut d_shares_copied = d_shares[i..to_index].to_vec().to_vec().clone();
//            let mut multiplication_e_copied = multiplication_e[i + 1..to_index + 1].to_vec().clone();
//            thread_pool.execute(move || {
//                let mut batch_mul_result = batch_multiplication_byte(&multiplication_e_copied, &d_shares_copied, &mut ctx_copied);
//                let mut output_map = output_map.lock().unwrap();
//                (*output_map).insert(batch_count, batch_mul_result);
//            });
//            i = to_index;
//            batch_count += 1;
//        }
//        thread_pool.join();
//        let output_map = &*(output_map.lock().unwrap());
//        for i in 0..batch_count {
//            let products = output_map.get(&i).unwrap();
//            for j in 0..products.len() {
//                let global_index = i * 10 + j;
//                c_shares[global_index] = products[j];
//            }
//        }
//        c_shares[bit_length - 1] = d_shares[bit_length - 1];
//        //compute w shares
//        w = ctx.asymmetric_bit as i8;
//        for i in 0..bit_length {
//            w += c_shares[i] as i8;
//            w = mod_floor(w, BINARY_PRIME as i8);
//        }
//        w as u8
//    }
//
//    fn compute_e_shares(bit_length:usize,x_list: &Vec<u8>, y_list: &Vec<u8>,e_shares:&mut Vec<u8>,asymmetric_bit:u8){
//        for i in 0..bit_length {
//            let e_share = x_list[i] + y_list[i] + asymmetric_bit;
//            e_shares[i] = mod_floor(e_share, BINARY_PRIME as u8);
//        }
//    }
//
//    fn compute_d_shares(){
//
//    }
//
//    fn compute_mul_e_parallel(){
//
//    }
//
//    fn compute_c_shares(){
//
//    }
//
//    fn compute_w(){
//
//    }
}
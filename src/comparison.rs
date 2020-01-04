pub mod comparison {
    use crate::computing_party::computing_party::ComputingParty;
    use threadpool::ThreadPool;
    use crate::constants::constants::BINARY_PRIME;
    use std::cmp::{max, min};
    use num::integer::*;
    use std::sync::{Arc, Mutex};
    use crate::multiplication::multiplication::{multi_thread_batch_mul_byte, batch_multiplication_byte};
    use std::collections::HashMap;
    use num::{abs, BigUint};
    use crate::bit_decomposition::bit_decomposition::{bit_decomposition, bit_decomposition_bigint};
    use crate::utils::utils::{big_uint_clone, get_current_binary_share};
    use std::num::Wrapping;

    fn compute_e_shares(x_list: &Vec<u8>, y_list: &Vec<u8>, ctx: &mut ComputingParty) -> Vec<u8> {
        let bit_length = max(x_list.len(), y_list.len());
        let mut e_shares = vec![0u8; bit_length];
        for i in 0..bit_length {
            let e_share = x_list[i] + y_list[i] + ctx.asymmetric_bit;
            e_shares[i] = mod_floor(e_share, BINARY_PRIME as u8);
        }
        e_shares
    }

    fn compute_d_shares(x_list: &Vec<u8>, y_list: &Vec<u8>, ctx: &mut ComputingParty) -> Vec<u8> {
        let bit_length = max(x_list.len(), y_list.len());
        let mut d_shares = vec![0u8; bit_length];
        ctx.thread_hierarchy.push("compute_D_shares".to_string());
        let (batch_count, output_map) = multi_thread_batch_mul_byte(&x_list, &y_list, ctx, bit_length);
        let mut global_index = 0;
        for i in 0..batch_count {
            let product_result = output_map.get(&i).unwrap();
            for j in 0..product_result.len() {
                global_index = i as usize * 10 + j;
                let local_diff = Wrapping(y_list[global_index]) - Wrapping(product_result[j]);
                d_shares[global_index] = mod_floor(local_diff.0, BINARY_PRIME as u8);
            }
        }
        ctx.thread_hierarchy.pop();
        d_shares
    }

    fn compute_multi_e_parallel(x_list: &Vec<u8>, y_list: &Vec<u8>, ctx: &mut ComputingParty, e_shares: &Vec<u8>) -> Vec<u8> {
        let bit_length = max(x_list.len(), y_list.len());
        let mut multiplication_e = vec![0u8; bit_length];
        ctx.thread_hierarchy.push("compute_E_parallel".to_string());
        let mut main_index = bit_length - 1;
        main_index -= 1;
        multiplication_e[main_index] = e_shares[bit_length - 1];
        let mut temp_mul_e = e_shares.clone();

        let thread_pool = ThreadPool::new(ctx.thread_count);
        while temp_mul_e.len() > 1 {
            let mut i = 0;
            let mut batch_count = 0;
            let mut output_map = Arc::new(Mutex::new(HashMap::new()));
            while i < temp_mul_e.len() - 1 {
                let mut output_map_copied = Arc::clone(&output_map);
                let to_index = min(i + ctx.batch_size, temp_mul_e.len());
                let mut ctx_cloned = ctx.clone();
                ctx_cloned.thread_hierarchy.push(format!("{}", batch_count));
                let mut x_list_sliced = temp_mul_e[i..to_index - 1].to_vec().clone();
                let mut y_list_sliced = temp_mul_e[i + 1..to_index].to_vec().clone();
                thread_pool.execute(move || {
                    let mut batch_mul_result = batch_multiplication_byte(&x_list_sliced, &y_list_sliced, &mut ctx_cloned);
                    let mut output_map_copied = output_map_copied.lock().unwrap();
                    (*output_map_copied).insert(batch_count, batch_mul_result);
                });
                i += to_index - 1;
                batch_count += 1;
            }
            thread_pool.join();
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
            if main_index > 0 {
                main_index -= 1;
                multiplication_e[main_index] = *temp_mul_e.last().unwrap();
            }
        }
        multiplication_e[0] = 0;
        ctx.thread_hierarchy.pop();
        multiplication_e
    }

    fn compute_c_shares(bit_length: usize, multiplication_e: &Vec<u8>, d_shares: &Vec<u8>, ctx: &mut ComputingParty)->Vec<u8> {
        ctx.thread_hierarchy.push("compute_c_shares".to_string());
        let (batch_count, output_map) = multi_thread_batch_mul_byte(multiplication_e, d_shares, ctx, bit_length - 1);
        ctx.thread_hierarchy.pop();
        let mut c_shares = vec![0u8; bit_length];
        for i in 0..batch_count {
            let products = output_map.get(&i).unwrap();
            for j in 0..products.len() {
                let global_index = i as usize * 10 + j;
                c_shares[global_index] = products[j];
            }
        }
        c_shares[bit_length - 1] = d_shares[bit_length - 1];
        c_shares
    }

    //the size of x_list and y_list must be the same, exception would be thrown otherwise
    pub fn comparison(x_list: &Vec<u8>, y_list: &Vec<u8>, ctx: &mut ComputingParty) -> u8 {
        ctx.thread_hierarchy.push("comparison".to_string());
        let bit_length = max(x_list.len(), y_list.len());
        let prime = BINARY_PRIME;
        let ti_shares = get_current_binary_share(ctx);
        let e_shares = compute_e_shares(x_list,y_list,ctx);

        let thread_pool = ThreadPool::new(2);
        let mut output_map = Arc::new(Mutex::new(HashMap::new()));
        let mut x_list_cp = x_list.clone();
        let mut y_list_cp = y_list.clone();
        let mut ctx_cloned = ctx.clone();
        let mut output_map_cp = Arc::clone(&output_map);
        thread_pool.execute(move||{
            let mut d_shares_computed = compute_d_shares(&x_list_cp, &y_list_cp, &mut ctx_cloned);
            let mut output_map_cp = output_map_cp.lock().unwrap();
            (*output_map_cp).insert(0,d_shares_computed);
        });

        let mut x_list_cp = x_list.clone();
        let mut y_list_cp = y_list.clone();
        let mut ctx_cloned = ctx.clone();
        let mut e_shares_cp = e_shares.clone();
        let mut output_map_cp = Arc::clone(&output_map);
        thread_pool.execute(move||{
            let mut e_shares_computed = compute_multi_e_parallel(&x_list_cp, &y_list_cp, &mut ctx_cloned,&e_shares_cp);
            let mut output_map_cp = output_map_cp.lock().unwrap();
            (*output_map_cp).insert(1,e_shares_computed);
        });
        thread_pool.join();
        let mut output_map = output_map.lock().unwrap();
        let mut d_shares = (*output_map).get(&0).unwrap();
        let mut multiplication_e = (*output_map).get(&1).unwrap();
        //compute c shares
        let mut c_shares = compute_c_shares(bit_length,&multiplication_e,&d_shares,ctx);
        //compute w shares
        let mut w = ctx.asymmetric_bit;
        for i in 0..bit_length {
            w += c_shares[i];
            w = mod_floor(w, prime as u8);
        }

        ctx.thread_hierarchy.pop();
        w as u8
    }

    pub fn compare_bigint(x: &BigUint, y: &BigUint, ctx: &mut ComputingParty) -> BigUint {
        let thread_pool = ThreadPool::new(2);

        let bit_length = ctx.dt_training.bit_length;

        let mut bd_result_map = Arc::new(Mutex::new(HashMap::new()));
        let mut bd_result_map_cp = Arc::clone(&bd_result_map);
        let mut ctx_cp = ctx.clone();
        let x_cp = big_uint_clone(x);
        thread_pool.execute(move || {
            let x_bits = bit_decomposition_bigint(&x_cp, &mut ctx_cp);
            let mut bd_result_map_cp = bd_result_map_cp.lock().unwrap();
            (*bd_result_map_cp).insert(0, x_bits);
        });

        let mut ctx_cp = ctx.clone();
        let mut bd_result_map_cp = Arc::clone(&bd_result_map);
        let y_cp = big_uint_clone(y);
        thread_pool.execute(move || {
            let y_bits = bit_decomposition_bigint(&y_cp, &mut ctx_cp);
            let mut bd_result_map_cp = bd_result_map_cp.lock().unwrap();
            (*bd_result_map_cp).insert(1, y_bits);
        });

        thread_pool.join();

        let bd_result_map = bd_result_map.lock().unwrap();
        let x_shares = (*bd_result_map).get(&0).unwrap();
        let y_shares = (*bd_result_map).get(&1).unwrap();

        let result = comparison(x_shares, y_shares, ctx);
        BigUint::from(result)
    }
}
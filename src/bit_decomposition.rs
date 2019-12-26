pub mod bit_decomposition {
    use crate::computing_party::computing_party::ComputingParty;
    use num::{abs, BigUint};
    use num::integer::*;
    use crate::constants::constants::BINARY_PRIME;
    use crate::multiplication::multiplication::{multiplication_byte, batch_multiplication_byte};
    use crate::utils::utils::{increment_current_share_index, big_uint_clone};
    use std::sync::{Arc, Mutex};
    use threadpool::ThreadPool;
    use std::collections::HashMap;
    use std::cmp::min;
    use std::num::Wrapping;

    pub fn bit_decomposition(input: u64, ctx: &mut ComputingParty) -> Vec<u8> {
        ctx.thread_hierarchy.push("bit_decomposition".to_string());
        let mut input_shares = Vec::new();
        let bit_length = ctx.dt_training.bit_length as usize;

        let binary_str = format!("{:b}", input);
        let input_binary_str_vec: Vec<&str> = binary_str.split("").collect();
        let mut temp: Vec<u8> = Vec::new();
        for item in binary_str.chars() {
            temp.push(item as u8);
        }
        let mut temp0 = vec![0u8; bit_length];
        let diff = abs(bit_length as isize - temp.len() as isize);
        for i in 0..diff {
            temp.push(0);
        }
        // hard-coded for two-party

        for i in 0..2 {
            let mut temp = temp.clone();
            let mut temp0 = temp0.clone();
            if i == ctx.party_id {
                input_shares.push(temp);
            } else {
                input_shares.push(temp0);
            }
        }


        let x_shares = generate_bit_decomposition(bit_length,&input_shares,ctx);

        // pop bit_decomposition
        ctx.thread_hierarchy.pop();
        x_shares
    }

    pub fn bit_decomposition_bigint(input: &BigUint, ctx: &mut ComputingParty) -> Vec<u8> {
        let bit_length = ctx.dt_training.bit_length as usize;
        let prime = BINARY_PRIME as u8;
        let mut input_shares = Vec::new();
        let mut temp = input.to_bytes_le();
        let mut temp0 = vec![0u8; bit_length as usize];
        let diff = (abs((bit_length - temp.len()) as isize)) as usize;
        for i in 0..diff{
            temp.push(0);
        }

        //todo add partyCount to ctx
        for i in 0..2{
            if i==ctx.party_id {
                input_shares.push(temp.clone());
            }else{
                input_shares.push(temp0.clone());
            }
        }

        let x_shares = generate_bit_decomposition(bit_length,&input_shares,ctx);
        // pop bit_decomposition
        ctx.thread_hierarchy.pop();
        x_shares
    }

    fn generate_bit_decomposition(bit_length:usize,input_shares:&Vec<Vec<u8>>,ctx:&mut ComputingParty)->Vec<u8>{
        let mut e_shares = vec![0u8;bit_length as usize];
        let mut d_shares = vec![0u8;bit_length as usize];
        let mut c_shares = vec![0u8;bit_length as usize];
        let mut x_shares = vec![0u8;bit_length as usize];
        let mut y_shares = vec![0u8;bit_length as usize];

        //initY
        for i in 0..bit_length{
            let y = input_shares[0][i]+input_shares[1][i];
            y_shares[i] = mod_floor(y, BINARY_PRIME as u8);
        }
        x_shares[0] = y_shares[0] as u8;

        //Initialize c[1]
        let first_c_share = multiplication_byte(input_shares[0][0], input_shares[1][0], ctx);
        increment_current_share_index(Arc::clone(&ctx.dt_shares.current_binary_index));
        c_shares[0] = mod_floor(first_c_share, BINARY_PRIME as u8);

        //computeDShares in Java Lynx
        let thread_pool = ThreadPool::new(ctx.thread_count);
        let mut i = 0;
        let mut output_map = Arc::new(Mutex::new(HashMap::new()));
        let mut batch_count = 0;
        ctx.thread_hierarchy.push("compute_d_shares".to_string());
        while i < bit_length {
            let mut output_map = Arc::clone(&output_map);
            let to_index = min(i + ctx.batch_size, bit_length);
            let mut ctx_copied = ctx.clone();
            ctx_copied.thread_hierarchy.push(format!("{}", batch_count));
            let mut input_shares = input_shares.clone();
            thread_pool.execute(move || {
                let mut batch_mul_result = batch_multiplication_byte(&input_shares[0][i..to_index].to_vec(), &input_shares[1][i..to_index].to_vec(), &mut ctx_copied);
                let mut output_map = output_map.lock().unwrap();
                (*output_map).insert(batch_count, batch_mul_result);
            });
            i = to_index;
            batch_count += 1;
        }
        thread_pool.join();
        ctx.thread_hierarchy.pop();

        let output_map = &(*(output_map.lock().unwrap()));
        let mut global_index = 0;
        for i in 0..batch_count {
            let batch_result = output_map.get(&i).unwrap();
            for item in batch_result.iter() {
                d_shares[global_index] = mod_floor(item + ctx.asymmetric_bit, BINARY_PRIME as u8);
                global_index += 1;
            }
        }
        let mut e_shares = vec![0u8; bit_length];

        ctx.thread_hierarchy.push("compute_variables".to_string());

        for i in 1..bit_length {
            //computeVariables
            let e_result = (Wrapping(multiplication_byte(y_shares[i], c_shares[i - 1], ctx)) + Wrapping(ctx.asymmetric_bit)).0;
            e_shares[i] = mod_floor(e_result, BINARY_PRIME as u8);
            let x_result = (Wrapping(y_shares[i]) + Wrapping(c_shares[i - 1])).0;
            x_shares[i] = mod_floor(x_result, BINARY_PRIME as u8);
            let c_result = (Wrapping(multiplication_byte(e_shares[i], d_shares[i], ctx)) + Wrapping(ctx.asymmetric_bit)).0;
            c_shares[i] = mod_floor(c_result, BINARY_PRIME as u8);
        }
        // pop computeVariables
        ctx.thread_hierarchy.pop();
        x_shares
    }
}
pub mod bit_decomposition {
    use crate::computing_party::computing_party::ComputingParty;
    use num::{abs, BigUint, FromPrimitive, Zero, ToPrimitive};
    use num::integer::*;
    use crate::constants::constants::BINARY_PRIME;
    use crate::multiplication::multiplication::{multiplication_byte, batch_multiplication_byte};
    use crate::utils::utils::{increment_current_share_index, big_uint_clone};
    use std::sync::{Arc, Mutex};
    use threadpool::ThreadPool;
    use std::collections::HashMap;
    use std::cmp::min;
    use std::num::Wrapping;
    use std::ops::Div;

    pub fn bit_decomposition(input: u64, ctx: &mut ComputingParty,bit_length:usize) -> Vec<u8> {
        ctx.thread_hierarchy.push("bit_decomposition".to_string());
        let mut input_shares = Vec::new();

        let binary_str = format!("{:b}", input);
        let reversed_binary_vec:Vec<char> = binary_str.chars().rev().collect();
        let mut temp: Vec<u8> = Vec::new();
        for item in reversed_binary_vec {
            let item_parsed:u8 = format!("{}",item).parse().unwrap();
            temp.push(item_parsed);
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

    pub fn batch_bit_decomposition(input_vec:&Vec<Wrapping<u64>>,ctx: &mut ComputingParty,bit_length:usize)->Vec<Vec<u8>>{
        let mut input_share_list = Vec::new();
        for input in input_vec{
            let binary_str = format!("{:b}", input.0);
            let reversed_binary_vec:Vec<char> = binary_str.chars().rev().collect();
            let mut temp = Vec::new();
            for item in reversed_binary_vec {
                let item_parsed:u8 = format!("{}",item).parse().unwrap();
                temp.push(item_parsed);
            }
            let mut temp0 = vec![0u8; bit_length];
            let diff = abs(bit_length as isize - temp.len() as isize);
            for i in 0..diff {
                temp.push(0);
            }
            let mut input_shares = Vec::new();
            for i in 0..2 {
                let mut temp = temp.clone();
                let mut temp0 = temp0.clone();
                if i == ctx.party_id {
                    input_shares.push(temp);
                } else {
                    input_shares.push(temp0);
                }
            }
            input_share_list.push(input_shares)
        }

        let x_shares = generate_bit_decomposition_batch(bit_length,&mut input_share_list,ctx);

        x_shares
    }


    pub fn bit_decomposition_bigint(input: &BigUint, ctx: &mut ComputingParty) -> Vec<u8> {
        let bit_length = ctx.dt_training.bit_length as usize;
        let prime = BINARY_PRIME as u8;
        let mut input_shares = Vec::new();
        //convert bigint to bit vector
        let two = BigUint::from_u32(2).unwrap();
        let mut temp = Vec::new();
        let mut bigint_val = big_uint_clone(&input);
        while !bigint_val.eq(&BigUint::zero()) {
            temp.push(bigint_val.mod_floor(&two).to_u8().unwrap());
            bigint_val = bigint_val.div_floor(&two);
        }
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

    fn generate_bit_decomposition_batch(bit_length:usize,input_shares:&mut Vec<Vec<Vec<u8>>>,ctx:&mut ComputingParty)->Vec<Vec<u8>>{
        let list_len = input_shares.len();
        let mut e_shares = vec![vec![0u8;bit_length as usize];list_len];
        let mut d_shares = vec![vec![0u8;bit_length as usize];list_len];
        let mut c_shares = vec![vec![0u8;bit_length as usize];list_len];
        let mut x_shares = vec![vec![0u8;bit_length as usize];list_len];
        let mut y_shares = vec![vec![0u8;bit_length as usize];list_len];

        let mut first_c_multiple = Vec::new();
        let mut first_c_multiplicand = Vec::new();
        let mut input_share0 = Vec::new();
        let mut input_share1 = Vec::new();
        for j in 0..list_len{
            for i in 0..bit_length{
                let y = input_shares[j][0][i]+input_shares[j][1][i];
                y_shares[j][i] = mod_floor(y, BINARY_PRIME as u8);
            }
            input_share0.append(&mut input_shares[j][0]);
            input_share1.append(&mut input_shares[j][1]);

            first_c_multiple.push(input_share0[0]);
            first_c_multiplicand.push(input_share0[0]);
            x_shares[j][0] = y_shares[j][0] as u8;
        }
        //Initialize c[1]
        let first_c_shares = batch_multiplication_byte(&first_c_multiple, &first_c_multiplicand, ctx);
        for i in 0..first_c_shares.len(){
            c_shares[i][0] = mod_floor(first_c_shares[i], BINARY_PRIME as u8);
        }
        let mut i = 0;
        let mut batch_mul_result = batch_multiplication_byte(&input_share0, &input_share1, ctx);
        for j in 0..list_len{
            for i in 0..bit_length{
                d_shares[j][i]=mod_floor(batch_mul_result[j*bit_length+i] + ctx.asymmetric_bit,BINARY_PRIME as u8);
            }
        }
        let mut e_shares = vec![vec![0u8; bit_length];list_len];
        for j in 0..list_len{
            for i in 1..bit_length{
                let e_result = (Wrapping(multiplication_byte(y_shares[j][i], c_shares[j][i - 1], ctx)) + Wrapping(ctx.asymmetric_bit)).0;
                e_shares[j][i] = mod_floor(e_result, BINARY_PRIME as u8);
                let x_result = (Wrapping(y_shares[j][i]) + Wrapping(c_shares[j][i - 1])).0;
                x_shares[j][i] = mod_floor(x_result, BINARY_PRIME as u8);
                let c_result = (Wrapping(multiplication_byte(e_shares[j][i], d_shares[j][i], ctx)) + Wrapping(ctx.asymmetric_bit)).0;
                c_shares[j][i] = mod_floor(c_result, BINARY_PRIME as u8);
            }
        }

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
        c_shares[0] = mod_floor(first_c_share, BINARY_PRIME as u8);

        //computeDShares in Java Lynx
        let mut i = 0;

        if ctx.raw_tcp_communication{
            let mut global_index = 0;
            while i < bit_length {
                let to_index = min(i + ctx.batch_size, bit_length);
                let mut batch_mul_result = batch_multiplication_byte(&input_shares[0][i..to_index].to_vec(), &input_shares[1][i..to_index].to_vec(), ctx);
                for item in batch_mul_result.iter() {
                    d_shares[global_index] = mod_floor(item + ctx.asymmetric_bit, BINARY_PRIME as u8);
                    global_index += 1;
                }
                i = to_index;
            }
        }else{
            let mut output_map = Arc::new(Mutex::new(HashMap::new()));
            let mut batch_count = 0;
            let thread_pool = ThreadPool::new(ctx.thread_count);
            ctx.thread_hierarchy.push("compute_d_shares".to_string());
            while i < bit_length {
                let to_index = min(i + ctx.batch_size, bit_length);
                let mut output_map = Arc::clone(&output_map);
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
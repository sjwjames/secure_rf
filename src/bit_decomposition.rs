pub mod bit_decomposition{
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
}
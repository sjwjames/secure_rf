pub mod bit_decomposition {
    use crate::computing_party::computing_party::ComputingParty;
    use num::{abs, BigUint, FromPrimitive, Zero, ToPrimitive};
    use num::integer::*;
    use crate::constants::constants::BINARY_PRIME;
    use crate::multiplication::multiplication::{multiplication_byte, batch_multiplication_byte};
    use crate::utils::utils::{increment_current_share_index, big_uint_clone, send_u8_messages};
    use std::sync::{Arc, Mutex};
    use threadpool::ThreadPool;
    use std::collections::HashMap;
    use std::cmp::min;
    use std::num::Wrapping;
    use std::ops::Div;
    use crate::protocol::protocol::{batch_bitwise_and, convert_integer_to_bits};

    pub fn bit_decomposition(input: u64, ctx: &mut ComputingParty, bit_length: usize) -> Vec<u8> {
        ctx.thread_hierarchy.push("bit_decomposition".to_string());
        let mut input_shares = Vec::new();

        let mut temp0 = vec![0u8; bit_length];
        let mut temp = convert_integer_to_bits(input, bit_length);

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


        let x_shares = generate_bit_decomposition(bit_length, &input_shares, ctx);

        // pop bit_decomposition
        ctx.thread_hierarchy.pop();
        x_shares
    }

    pub fn batch_bit_decomposition(input_vec: &Vec<Wrapping<u64>>, ctx: &mut ComputingParty, bit_length: usize) -> Vec<Vec<u8>> {
        let mut input_share_list = Vec::new();
        for input in input_vec {
            let binary_str = format!("{:b}", input.0);
            let reversed_binary_vec: Vec<char> = binary_str.chars().rev().collect();
            let mut temp = Vec::new();
            for item in reversed_binary_vec {
                let item_parsed: u8 = format!("{}", item).parse().unwrap();
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

        let x_shares = generate_bit_decomposition_batch(bit_length, &mut input_share_list, ctx);

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
        for i in 0..diff {
            temp.push(0);
        }

        //todo add partyCount to ctx
        for i in 0..2 {
            if i == ctx.party_id {
                input_shares.push(temp.clone());
            } else {
                input_shares.push(temp0.clone());
            }
        }

        let x_shares = generate_bit_decomposition(bit_length, &input_shares, ctx);
        // pop bit_decomposition
        ctx.thread_hierarchy.pop();
        x_shares
    }

    fn generate_bit_decomposition_batch(bit_length: usize, input_shares: &mut Vec<Vec<Vec<u8>>>, ctx: &mut ComputingParty) -> Vec<Vec<u8>> {
        let list_len = input_shares.len();
        let mut e_shares = vec![vec![0u8; bit_length as usize]; list_len];
        let mut d_shares = vec![vec![0u8; bit_length as usize]; list_len];
        let mut c_shares = vec![vec![0u8; bit_length as usize]; list_len];
        let mut x_shares = vec![vec![0u8; bit_length as usize]; list_len];
        let mut y_shares = vec![vec![0u8; bit_length as usize]; list_len];

        let mut first_c_multiple = Vec::new();
        let mut first_c_multiplicand = Vec::new();
        let mut input_share0 = Vec::new();
        let mut input_share1 = Vec::new();
        for j in 0..list_len {
            for i in 0..bit_length {
                let y = input_shares[j][0][i] + input_shares[j][1][i];
                y_shares[j][i] = mod_floor(y, BINARY_PRIME as u8);
            }
            input_share0.append(&mut input_shares[j][0]);
            input_share1.append(&mut input_shares[j][1]);

            first_c_multiple.push(input_share0[0]);
            first_c_multiplicand.push(input_share1[0]);
            x_shares[j][0] = y_shares[j][0] as u8;
        }
        //Initialize c[1]
        let first_c_shares = batch_multiplication_byte(&first_c_multiple, &first_c_multiplicand, ctx);
        for i in 0..first_c_shares.len() {
            c_shares[i][0] = mod_floor(first_c_shares[i], BINARY_PRIME as u8);
        }
        let mut batch_mul_result = batch_multiplication_byte(&input_share0, &input_share1, ctx);
        for j in 0..list_len {
            for i in 1..bit_length {
                d_shares[j][i] = mod_floor(batch_mul_result[j * bit_length + i] + ctx.asymmetric_bit, BINARY_PRIME as u8);
            }
        }
        let mut e_shares = vec![vec![0u8; bit_length]; list_len];
        for j in 0..list_len {
            for i in 1..bit_length {
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

    fn generate_bit_decomposition(bit_length: usize, input_shares: &Vec<Vec<u8>>, ctx: &mut ComputingParty) -> Vec<u8> {
        let mut e_shares = vec![0u8; bit_length as usize];
        let mut d_shares = vec![0u8; bit_length as usize];
        let mut c_shares = vec![0u8; bit_length as usize];
        let mut x_shares = vec![0u8; bit_length as usize];
        let mut y_shares = vec![0u8; bit_length as usize];

        //initY
        for i in 0..bit_length {
            let y = input_shares[0][i] + input_shares[1][i];
            y_shares[i] = mod_floor(y, BINARY_PRIME as u8);
        }
        x_shares[0] = y_shares[0] as u8;

        //Initialize c[1]
        let first_c_share = multiplication_byte(input_shares[0][0], input_shares[1][0], ctx);
        c_shares[0] = mod_floor(first_c_share, BINARY_PRIME as u8);

        //computeDShares in Java Lynx
        let mut i = 1;

        if ctx.raw_tcp_communication {
            let batch_mul_result = batch_multiplication_byte(&input_shares[0][1..bit_length].to_vec(), &input_shares[1][1..bit_length].to_vec(), ctx);
            for i in 0..bit_length - 1 {
                d_shares[i + 1] = mod_floor(batch_mul_result[i] + ctx.asymmetric_bit, BINARY_PRIME as u8);
            }
//            let mut global_index = 0;
//            while i < bit_length {
//                let to_index = min(i + ctx.batch_size, bit_length);
//                let mut batch_mul_result = batch_multiplication_byte(&input_shares[0][i..to_index].to_vec(), &input_shares[1][i..to_index].to_vec(), ctx);
//                for item in batch_mul_result.iter() {
//                    d_shares[global_index] = mod_floor(item + ctx.asymmetric_bit, BINARY_PRIME as u8);
//                    global_index += 1;
//                }
//                i = to_index;
//            }
        } else {
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

    pub fn bit_decomposition_opt(input: Wrapping<u64>, ctx: &mut ComputingParty, bit_length: usize) -> Vec<u8> {
        let mut input_shares = Vec::new();

        let mut temp = convert_integer_to_bits(input.0, bit_length);
        let mut temp0 = vec![0u8; bit_length];
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

        let mut g = batch_multiplication_byte(&input_shares[0][0..bit_length - 1].to_vec(), &input_shares[1][0..bit_length - 1].to_vec(), ctx);
//        let mut g = vec![0u8;bit_length];

        let mut result = vec![0u8; bit_length];
        let mut c_list = Vec::new();
        for k in 0..bit_length - 1 {
            let mut c: u8 = 0;
            for i in k..0 {
                let mut temp_item = g[i];
                for j in k..i {
                    temp_item = multiplication_byte(temp_item, temp[j], ctx);
                }
                c = (c + temp_item).mod_floor(&(BINARY_PRIME as u8));
            }
            c_list.push(c);
        }
        result[0] = temp[0];
        for i in 1..bit_length - 1 {
            result[i] = (temp[i] + c_list[i - 1]).mod_floor(&(BINARY_PRIME as u8));
        }
        result[bit_length - 1] = temp[bit_length - 1];
        result
    }


    pub fn batch_log_decomp(x_additive_list: &Vec<Wrapping<u64>>,
                            size: usize,
                            depth: usize,
                            ctx: &mut ComputingParty) -> Vec<u64> {
        let len = x_additive_list.len();

        let dummy = vec![0u64; len];
        let x_u64 = x_additive_list.iter().map(|x| x.0).collect();
        let x_and = if ctx.asymmetric_bit == 0 {
            batch_bitwise_and(&x_u64, &dummy, ctx, false)
        } else {
            batch_bitwise_and(&dummy, &x_u64, ctx, false)
        };

        let mut propogate = Vec::new();
        let mut generate = Vec::new();

        for (x, x_and) in x_u64.iter().zip(x_and.iter()) {
            propogate.push(vec![*x]);
            generate.push(vec![*x_and]);
        }

        let decomp_26_circuit: [Vec<[(usize, usize); 2]>; 5] =

            [
                vec![[(0, 0), (0, 1)], [(0, 2), (0, 3)], [(0, 4), (0, 5)], [(0, 6), (0, 7)], [(0, 8), (0, 9)], [(0, 10), (0, 11)], [(0, 12), (0, 13)], [(0, 14), (0, 15)], [(0, 16), (0, 17)], [(0, 18), (0, 19)], [(0, 20), (0, 21)], [(0, 22), (0, 23)], [(0, 24), (0, 25)]],
                vec![[(1, 0), (0, 2)], [(1, 0), (1, 1)], [(1, 2), (0, 6)], [(1, 2), (1, 3)], [(1, 4), (0, 10)], [(1, 4), (1, 5)], [(1, 6), (0, 14)], [(1, 6), (1, 7)], [(1, 8), (0, 18)], [(1, 8), (1, 9)], [(1, 10), (0, 22)], [(1, 10), (1, 11)]],
                vec![[(2, 1), (0, 4)], [(2, 1), (1, 2)], [(2, 1), (2, 2)], [(2, 1), (2, 3)], [(2, 5), (0, 12)], [(2, 5), (1, 6)], [(2, 5), (2, 6)], [(2, 5), (2, 7)], [(2, 9), (0, 20)], [(2, 9), (1, 10)], [(2, 9), (2, 10)], [(2, 9), (2, 11)]],
                vec![[(3, 3), (0, 8)], [(3, 3), (1, 4)], [(3, 3), (2, 4)], [(3, 3), (2, 5)], [(3, 3), (3, 4)], [(3, 3), (3, 5)], [(3, 3), (3, 6)], [(3, 3), (3, 7)], [(3, 11), (0, 24)], [(3, 11), (1, 12)]],
                vec![[(4, 7), (0, 16)], [(4, 7), (1, 8)], [(4, 7), (2, 8)], [(4, 7), (2, 9)], [(4, 7), (3, 8)], [(4, 7), (3, 9)], [(4, 7), (3, 10)], [(4, 7), (3, 11)], [(4, 7), (4, 8)], [(4, 7), (4, 9)]],
            ];

        for i in 0..depth {
            let mut prop_layer = vec![0u64; len];
            let mut gen_layer = vec![0u64; len];

            let mut ops: Vec<(u64, u64, u64, u64)> = Vec::new();

            for j in 0..len {
                let mut p = 0u64;
                let mut g = 0u64;
                let mut p_next = 0u64;
                let mut g_next = 0u64;


                for (k, index_pair) in decomp_26_circuit[i].iter().enumerate() {
                    let rop_row = index_pair[0].0;
                    let rop_col = index_pair[0].1;
                    let lop_row = index_pair[1].0;
                    let lop_col = index_pair[1].1;

                    p |= ((propogate[j][rop_row] >> rop_col as u64) & 1u64) << k as u64;
                    g |= ((generate[j][rop_row] >> rop_col as u64) & 1u64) << k as u64;
                    p_next |= ((propogate[j][lop_row] >> lop_col as u64) & 1u64) << k as u64;
                    g_next |= ((generate[j][lop_row] >> lop_col as u64) & 1u64) << k as u64;
                }

                ops.push((p, g, p_next, g_next));
            }

            let mut l_op = vec![0u64; 2 * len];
            let mut r_op = vec![0u64; 2 * len];
            let mut g_next = vec![0u64; len];


            // let ops: Vec<(u64, u64, u64, u64)> =
            // 	handles.into_iter().map(|x| x.join().unwrap()).collect();

            for i in (0..len).step_by(2) {
                l_op[i] = ops[i].2;
                l_op[i + 1] = ops[i].2;
                r_op[i] = ops[i].0;
                r_op[i + 1] = ops[i].1;

                g_next[i / 2] = ops[i].3;
            }

            let layers = batch_bitwise_and(&l_op, &r_op, ctx, false);

            let mut p_layer = vec![0u64; len];
            let mut g_layer = vec![0u64; len];

            p_layer.clone_from_slice(&layers[..len]);
            g_layer.clone_from_slice(&layers[len..]);

            g_layer = g_layer.iter().zip(g_next.iter()).map(|(&x, &y)| x ^ y).collect();

            for i in 0..len {
                propogate[i].push(p_layer[i]);
                generate[i].push(g_layer[i]);
            }
        }

        let mut carry: Vec<u64> = Vec::new();

        for instance in 0..len {
            carry.push(generate[instance][0] & 1);

            for i in 1..depth + 1 {
                let mask = (1 << i) - 1;

                carry[instance] |= (generate[instance][i] & mask as u64) << (i - 1) as u64;
            }

            carry[instance] <<= 1;
        }


        x_u64.iter().zip(carry.iter()).map(|(&p, &c)| p ^ c).collect()
    }

    pub fn batch_log_decomp_new(x_additive_list: &Vec<Wrapping<u64>>,
                                ctx: &mut ComputingParty) -> Vec<u64> {
        let len = x_additive_list.len();
        let propogate: Vec<u64> = x_additive_list.iter().map(|x| x.0).collect();
        let mut p_layer = propogate.clone();
        let mut g_layer = if ctx.asymmetric_bit == 0 {
            batch_bitwise_and(&propogate, &vec![0u64; len], ctx, false)
        } else {
            batch_bitwise_and(&vec![0u64; len], &propogate, ctx, false)
        };

        let mut matrices = (ctx.integer_precision + ctx.decimal_precision + 1) as usize;
        while matrices > 1 {
            let pairs = matrices / 2;
            let remainder = matrices % 2;

            let mut p = vec![0u64; len];
            let mut p_next = vec![0u64; len];
            let mut g = vec![0u64; len];
            let mut g_next = vec![0u64; len];

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

            let mut p_layer_next = vec![0u64; len];
            let mut g_layer_next = vec![0u64; len];

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
        let mut result_temp = Vec::new();
        let pos = 64 as usize;
        for i in 0..g_layer.len() {
            let g = g_layer[i];
            let p = propogate[i];
            let mut bits = x_additive_list[i].0 & 1;
            for i in 1..pos {
                let bit = 1 & (g ^ (p >> i as u64));
                if bit == 1 {
                    bits += 2.0f64.powf(pos as f64) as u64;
                }
            }
            result_temp.push(bits);
        }
        result_temp
    }
}
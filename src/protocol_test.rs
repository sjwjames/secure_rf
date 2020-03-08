pub mod protocol_test {
    use crate::computing_party::computing_party::ComputingParty;
    use std::num::Wrapping;
    use crate::multiplication::multiplication::{multiplication_byte, batch_multiplication_byte, batch_multiplication_integer, multiplication_bigint, multi_thread_batch_mul_byte, parallel_multiplication, batch_multiply_bigint, parallel_multiplication_big_integer};
    use crate::utils::utils::{reveal_byte_result, reveal_byte_vec_result, reveal_int_vec_result, reveal_bigint_result, reveal_bigint_vec_result, reveal_int_result, send_u8_messages, send_u64_messages, truncate_local, u64_to_byte_array};
    use rand::{random, Rng};
    use num::integer::*;
    use num::{BigUint, FromPrimitive, ToPrimitive, Zero, One};
    use std::cmp::max;
    use crate::protocol::protocol::{equality_big_integer, arg_max, batch_equality_integer, equality_integer_over_field, batch_equality, convert_integer_to_bits};
    use crate::comparison::comparison::{compare_bigint, comparison, compute_e_shares, compute_d_shares, compute_multi_e_parallel, compute_c_shares};
    use std::ops::BitAnd;
    use crate::bit_decomposition::bit_decomposition::{bit_decomposition, bit_decomposition_bigint, batch_bit_decomposition, bit_decomposition_opt, batch_log_decomp, batch_log_decomp_new};
    use crate::dot_product::dot_product::{dot_product_bigint, dot_product_integer};
    use crate::or_xor::or_xor::{or_xor, or_xor_bigint};
    use crate::field_change::field_change::{change_binary_to_decimal_field, change_binary_to_bigint_field};
    use std::time::SystemTime;
    use crate::discretize::discretize::{reveal, discretize, binary_vector_to_ring, xor_share_to_additive};

    pub fn test_multi_byte(ctx: &mut ComputingParty) {
        for i in 0..2 {
            for j in 0..2 {
                for m in 0..2 {
                    for n in 0..2 {
                        let mut result = 0;
                        if ctx.party_id == 0 {
                            result = multiplication_byte(i, j, ctx);
                        } else {
                            result = multiplication_byte(m, n, ctx);
                        }
                        let result_revealed = reveal_byte_result(result, ctx);
                        assert_eq!(result_revealed, (i ^ m) * (j ^ n), "we are testing multiplication_byte with {} and {}", (i ^ m), (j ^ n));
                    }
                }
            }
        }
    }

    pub fn test_batch_multiplication_byte(ctx: &mut ComputingParty) {
        let mut x_vec: Vec<u8> = Vec::new();
        let mut y_vec: Vec<u8> = Vec::new();

        if ctx.asymmetric_bit == 1 {
            x_vec = [1, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1].to_vec();
            y_vec = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].to_vec();
        } else {
            x_vec = [1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 0, 0].to_vec();
            y_vec = [0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1].to_vec();
        }
//        println!("x_pub_vec {:?}", x_pub_vec);
//        println!("y_pub_vec {:?}", y_pub_vec);
//        println!("result_vec {:?}", result_vec);
        let result = batch_multiplication_byte(&x_vec, &y_vec, ctx);
//        println!("computed result {:?}", &result);

        println!("result {:?}", result);
    }

    pub fn test_batch_multiplication_integer(ctx: &mut ComputingParty) {
        let mut x_vec: Vec<Wrapping<u64>> = Vec::new();
        let mut y_vec: Vec<Wrapping<u64>> = Vec::new();
        let mut x_vec_pub: Vec<Wrapping<u64>> = Vec::new();
        let mut y_vec_pub: Vec<Wrapping<u64>> = Vec::new();
        let mut result_pub: Vec<Wrapping<u64>> = Vec::new();

        for i in 0..1 {
            for j in 0..10 {
                x_vec_pub.push(Wrapping(j));
                y_vec_pub.push(Wrapping(j));
                if ctx.party_id == 0 {
                    x_vec.push(Wrapping(1));
                    y_vec.push(Wrapping(1));
                } else {
                    x_vec.push(Wrapping((Wrapping(j) - Wrapping(1)).0.mod_floor(&ctx.dt_training.dataset_size_prime)));
                    y_vec.push(Wrapping((Wrapping(j) - Wrapping(1)).0.mod_floor(&ctx.dt_training.dataset_size_prime)));
                }

                result_pub.push(Wrapping((j * j).mod_floor(&ctx.dt_training.dataset_size_prime)));
            }
        }
//        println!("result_pub {:?}", &result_pub);
        let mut time = 0;
        for i in 0..5 {
            let mut now = SystemTime::now();
            let result = batch_multiplication_integer(&x_vec, &y_vec, ctx, ctx.dt_training.dataset_size_prime);
            time += now.elapsed().unwrap().as_millis();
//            let result_revealed = reveal_int_vec_result(&result, ctx);
//            assert!(result_pub.iter().zip(result_revealed.iter()).all(|(a, b)| a.0 == *b), "Arrays are not equal");
        }

        println!("batch_multiplication completed in {}ms", (time as f64 / 5.0));
    }

    pub fn test_multiplication_bigint(ctx: &mut ComputingParty) {
        let mut x_vec_pub = Vec::new();
        let mut y_vec_pub = Vec::new();
        let mut result_pub = Vec::new();
        let mut x_vec = Vec::new();
        let mut y_vec = Vec::new();

        for i in 10..15 {
            x_vec_pub.push(BigUint::from_u32(i).unwrap());
            y_vec_pub.push(BigUint::from_u32(i).unwrap());
            if ctx.party_id == 0 {
                x_vec.push(BigUint::from_u32(1).unwrap());
                y_vec.push(BigUint::from_u32(1).unwrap());
            } else {
                x_vec.push(BigUint::from_u32((i - 1)).unwrap().mod_floor(&ctx.dt_training.big_int_prime));
                y_vec.push(BigUint::from_u32((i - 1)).unwrap().mod_floor(&ctx.dt_training.big_int_prime));
            }

            result_pub.push(BigUint::from_u32(i * i).unwrap());
        }

        for i in 0..5 {
            let result = multiplication_bigint(&x_vec[i], &y_vec[i], ctx);
            let result_revealed = reveal_bigint_result(&result, ctx);
            println!("result_revealed: {}", result_revealed.to_usize().unwrap());
            assert_eq!(result_pub[i].to_u64().unwrap(), result_revealed.to_u64().unwrap());
        }
    }

    pub fn test_parallel_multiplication(ctx: &mut ComputingParty) {
        let mut row = Vec::new();
        for i in 0..100 {
            if ctx.party_id == 0 {
                row.push(1);
            } else {
                row.push(0);
            }
        }
        let result = parallel_multiplication(&row, ctx);
        let result_revealed = reveal_byte_result(result, ctx);
        assert_eq!(result_revealed, 1, "we are testing parallel_multiplication with 100 ones");
    }

    pub fn test_batch_multiply_bigint(ctx: &mut ComputingParty) {
        let mut x_vec_pub = Vec::new();
        let mut y_vec_pub = Vec::new();
        let mut result_pub = Vec::new();
        let mut x_vec = Vec::new();
        let mut y_vec = Vec::new();

        for i in 10..15 {
            x_vec_pub.push(BigUint::from_u32(i).unwrap());
            y_vec_pub.push(BigUint::from_u32(i).unwrap());
            if ctx.party_id == 0 {
                x_vec.push(BigUint::from_u32(1).unwrap());
                y_vec.push(BigUint::from_u32(1).unwrap());
            } else {
                x_vec.push(BigUint::from_u32((i - 1)).unwrap().mod_floor(&ctx.dt_training.big_int_prime));
                y_vec.push(BigUint::from_u32((i - 1)).unwrap().mod_floor(&ctx.dt_training.big_int_prime));
            }

            result_pub.push(BigUint::from_u32(i * i).unwrap());
        }

        let result_computed = batch_multiply_bigint(&x_vec, &y_vec, ctx);
        let result_revealed = reveal_bigint_vec_result(&result_computed, ctx);
        assert!(result_pub.iter().zip(result_revealed.iter()).all(|(a, b)| a.eq(b)), "Arrays are not equal");
    }

    pub fn test_parallel_multiplication_big_integer(ctx: &mut ComputingParty) {
        let mut row = Vec::new();
        let mut row_pub = Vec::new();
        let mut result = Wrapping(1);
        for i in 1..100 {
            row_pub.push(BigUint::from_u32(i).unwrap());
            if ctx.party_id == 0 {
                row.push(BigUint::from_u32(i / 2).unwrap());
            } else {
                row.push(BigUint::from_u32(i - i / 2).unwrap());
            }
            result = result * Wrapping(i);
        }
        let result_computed = parallel_multiplication_big_integer(&row, ctx);
        let result_revealed = reveal_bigint_result(&result_computed, ctx);
        assert_eq!(result_revealed.to_u32().unwrap(), result.0, "we are testing parallel_multiplication with 100 ones");
    }

    pub fn test_multi_thread_batch_mul_byte(ctx: &mut ComputingParty) {
        let mut x_vec: Vec<u8> = Vec::new();
        let mut y_vec: Vec<u8> = Vec::new();
        let mut result_vec: Vec<u8> = Vec::new();
        let mut x_pub_vec: Vec<u8> = Vec::new();
        let mut y_pub_vec: Vec<u8> = Vec::new();

        for i in 0..100 {
            if ctx.party_id == 0 {
                x_vec.push(1);
                y_vec.push(0);
            } else {
                x_vec.push(0);
                y_vec.push(1);
            }

            x_pub_vec.push(1);
            y_pub_vec.push(1);
            result_vec.push(1);
        }


//        println!("x_pub_vec {:?}", x_pub_vec);
//        println!("y_pub_vec {:?}", y_pub_vec);
//        println!("result_vec {:?}", result_vec);
        let (batch_count, result_map) = multi_thread_batch_mul_byte(&x_vec, &y_vec, ctx, 100);

        let mut revealed_all = Vec::new();
        for i in 0..batch_count {
            let result = result_map.get(&i).unwrap();
            let mut result_revealed = reveal_byte_vec_result(result, ctx);
            revealed_all.append(&mut result_revealed);
        }
        assert!(result_vec.iter().zip(revealed_all.iter()).all(|(a, b)| a == b), "Arrays are not equal");
    }

    pub fn test_equality_big_integer(ctx: &mut ComputingParty) {
        let mut result = BigUint::zero();
        if ctx.party_id == 0 {
            let x = BigUint::from_u32(0).unwrap();
            let y = BigUint::from_u32(0).unwrap();
            result = equality_big_integer(&x, &y, ctx);
        } else {
            let x = BigUint::from_u32(0).unwrap();
            let y = BigUint::from_u32(0).unwrap();
            result = equality_big_integer(&x, &y, ctx);
        }
        let result_revealed = reveal_bigint_result(&result, ctx);
        assert_eq!(result_revealed, BigUint::zero());
    }

    pub fn test_comparison(ctx: &mut ComputingParty) {
        let mut result = 0;
        for i in 0..20 {
            if ctx.party_id == 0 {
                let x: Vec<u8> = vec![0, 0, 0, 0];
                let y: Vec<u8> = vec![0, 0, 0, 0];
//            let e_shares = compute_e_shares(&x, &y, ctx);
//            assert_eq!(e_shares, [1, 1, 0, 0]);
//            let d_shares = compute_d_shares(&x, &y, ctx);
//            let revealed_d_shares = reveal_byte_vec_result(&d_shares, ctx);
//            assert_eq!(revealed_d_shares, [0, 1, 0, 0]);
//            let e_reverse = compute_multi_e_parallel(&x, &y, ctx, &e_shares);
//            let e_reverse_revealed = reveal_byte_vec_result(&e_reverse, ctx);
//            println!("{:?}", e_reverse);
//            assert_eq!(e_reverse_revealed, [0, 1, 0, 1]);
//            let c_share = compute_c_shares(x.len(), &e_reverse, &d_shares, ctx);
//            println!("c_share:{:?}", c_share);
//            let c_share_revealed = reveal_byte_vec_result(&c_share, ctx);
//            assert_eq!(c_share_revealed, [0, 0, 0, 0]);
                result = comparison(&x, &y, ctx);
            } else {
                let x: Vec<u8> = vec![0, 0, 0, 0];
                let y: Vec<u8> = vec![0, 0, 0, 0];
//            let e_shares = compute_e_shares(&x, &y, ctx);
//            assert_eq!(e_shares, [0, 1, 1, 1]);
//            let d_shares = compute_d_shares(&x, &y, ctx);
//            let revealed_d_shares = reveal_byte_vec_result(&d_shares, ctx);
//            assert_eq!(revealed_d_shares, [0, 1, 0, 0]);
//            let e_reverse = compute_multi_e_parallel(&x, &y, ctx, &e_shares);
//            let e_reverse_revealed = reveal_byte_vec_result(&e_reverse, ctx);
//            println!("{:?}", e_reverse);
//            assert_eq!(e_reverse_revealed, [0, 1, 1, 0]);
//            let c_share = compute_c_shares(x.len(), &e_reverse, &d_shares, ctx);
//            println!("c_share:{:?}", c_share);
//            let c_share_revealed = reveal_byte_vec_result(&c_share, ctx);
//            assert_eq!(c_share_revealed, [0, 0, 0, 0]);
                result = comparison(&x, &y, ctx);
            }
            let received = send_u8_messages(ctx, &[result].to_vec())[0];
            println!("{}", received ^ result);
        }

//        let result_revealed = reveal_byte_result(result, ctx);
//        assert_eq!(result_revealed, 0);
    }

    pub fn test_batch_bit_decomposition(ctx: &mut ComputingParty) {
        let length = 3;
        let mut result = Vec::new();
        let size = (ctx.integer_precision + ctx.decimal_precision + 1) as usize;
        let depth = ((size - 1) as f64).log2().ceil() as usize;
        if ctx.party_id == 0 {
            let x = vec![Wrapping(18446744073709551615 as u64), Wrapping(1 as u64), Wrapping(0 as u64), Wrapping(1 as u64)];
//            let x = vec![Wrapping(7 as u64)];

//            result = bit_decomposition(29, ctx, 5);
            result = batch_log_decomp_new(&x, ctx);

//            result= bit_decomposition_opt(x[0],ctx,4);
//            result = batch_log_decomp(&x, size, depth, ctx);
        } else {
            let x = vec![Wrapping(18446744073709551614 as u64), Wrapping(1 as u64), Wrapping(1 as u64), Wrapping(0 as u64)];
////            let x = vec![Wrapping(7 as u64)];
//            result = bit_decomposition(17, ctx, 5);
            result = batch_log_decomp_new(&x, ctx);

//            result= bit_decomposition_opt(x[0],ctx,4);
//            result = batch_log_decomp(&x, size, depth, ctx);
        }
//        println!("result:{:?}", result);
        for item in result {
            let bits = convert_integer_to_bits(item, size);
            let result_shared = send_u8_messages(ctx, &bits);
            for i in 0..size {
                print!("{}", bits[i] ^ result_shared[i]);
            }
            println!("");
        }
    }

    pub fn test_batch_comparison() {}

    pub fn test_comparison_bigint(ctx: &mut ComputingParty) {
        let mut result = BigUint::zero();
        if ctx.party_id == 0 {
            let x = BigUint::from_u32(4).unwrap();
            let y = BigUint::from_u32(2).unwrap();
            result = compare_bigint(&x, &y, ctx);
        } else {
            let x = BigUint::from_u32(4).unwrap();
            let y = BigUint::from_u32(3).unwrap();
            result = compare_bigint(&x, &y, ctx);
        }
        let result_revealed = reveal_bigint_result(&result, ctx);
        println!("{}", result_revealed.to_string());
        assert_eq!(result_revealed.mod_floor(&BigUint::from_u32(2).unwrap()), BigUint::one());
    }

    pub fn test_bit_decomposition(ctx: &mut ComputingParty) {
        let bit_length = (ctx.dt_training.prime as f64).log2().ceil() as usize;
        if ctx.party_id == 0 {
            let input = 4;
            let bit_decomposed = bit_decomposition(input, ctx, bit_length);
//            let bit_decomposed = batch_log_decomp(input, ctx, bit_length);
//            let bit_decomposed_revealed = reveal_byte_vec_result(&bit_decomposed, ctx);
//            println!("{:?}", bit_decomposed_revealed);
            println!("{:?}", bit_decomposed);
        } else {
            let input = 3;
            let bit_decomposed = bit_decomposition(input, ctx, bit_length);
//            let bit_decomposed = batch_log_decomp(input, ctx, bit_length);
//            let bit_decomposed_revealed = reveal_byte_vec_result(&bit_decomposed, ctx);
//            println!("{:?}", bit_decomposed_revealed);
            println!("{:?}", bit_decomposed);
        }
    }

    pub fn test_bit_decomposition_bigint(ctx: &mut ComputingParty) {
        if ctx.party_id == 0 {
            let input = BigUint::from_u32(2).unwrap();
            let bit_decomposed = bit_decomposition_bigint(&input, ctx);
            let bit_decomposed_revealed = reveal_byte_vec_result(&bit_decomposed, ctx);
            println!("{:?}", bit_decomposed_revealed);
        } else {
            let input = BigUint::from_u32(4).unwrap();
            let bit_decomposed = bit_decomposition_bigint(&input, ctx);
            let bit_decomposed_revealed = reveal_byte_vec_result(&bit_decomposed, ctx);
            println!("{:?}", bit_decomposed_revealed);
        }
    }

    pub fn test_dot_product_bigint(ctx: &mut ComputingParty) {
        if ctx.party_id == 0 {
            let x = vec![BigUint::from_u32(0).unwrap(), BigUint::from_u32(0).unwrap()];
            let y = vec![BigUint::from_u32(1).unwrap(), BigUint::from_u32(1).unwrap()];
            let result = dot_product_bigint(&x, &y, ctx);
            let result_bytes = result.to_bytes_le();
            println!("{:?}", result_bytes);
            let changed = change_binary_to_bigint_field(&result_bytes, ctx);


//            let result_revealed = reveal_bigint_result(&result, ctx);
//            println!("{}", result_revealed.to_string());
            for item in changed {
                println!("{}", item.to_string());
            }
        } else {
            let x = vec![BigUint::from_u32(1).unwrap(), BigUint::from_u32(0).unwrap()];
            let y = vec![BigUint::from_u32(1).unwrap(), BigUint::from_u32(0).unwrap()];
            let result = dot_product_bigint(&x, &y, ctx);
            let result_bytes = result.to_bytes_le();
            println!("{:?}", result_bytes);
            let changed = change_binary_to_bigint_field(&result_bytes, ctx);
//            let result_revealed = reveal_bigint_result(&result, ctx);
//            println!("{}", result_revealed.to_string());
            for item in changed {
                println!("{}", item.to_string());
            }
        }
    }

    pub fn test_or_xor(ctx: &mut ComputingParty) {
        if ctx.party_id == 0 {
            let x = vec![Wrapping(1), Wrapping(1)];
            let y = vec![Wrapping(0), Wrapping(0)];
            let result = or_xor(&x, &y, ctx, 2, ctx.dt_training.dataset_size_prime);
            println!("{:?}", result);
//            let result_revealed = reveal_byte_vec_result(&result,ctx);
//            println!("{}",result_revealed.to_string());
        } else {
            let x = vec![Wrapping(1), Wrapping(0)];
            let y = vec![Wrapping(0), Wrapping(0)];
            let result = or_xor(&x, &y, ctx, 2, ctx.dt_training.dataset_size_prime);
            println!("{:?}", result);
//            let result_revealed = reveal_bigint_result(&result,ctx);
//            println!("{}",result_revealed.to_string());
        }
    }

    pub fn test_or_xor_bigint(ctx: &mut ComputingParty) {
        if ctx.party_id == 0 {
            let x = vec![BigUint::one()];
            let y = vec![BigUint::zero()];
            let result = or_xor_bigint(&x, &y, ctx, &BigUint::from_u32(2).unwrap());
            println!("{}", result[0].to_string());
//            let result_revealed = reveal_bigint_vec_result(&result, ctx);
//            println!("{:?}", result_revealed);
        } else {
            let x = vec![BigUint::one()];
            let y = vec![BigUint::zero()];
            let result = or_xor_bigint(&x, &y, ctx, &BigUint::from_u32(2).unwrap());
            println!("{}", result[0].to_string());
//            let result_revealed = reveal_bigint_vec_result(&result, ctx);
//            println!("{:?}", result_revealed);
        }
    }


    pub fn test_change_binary_to_decimal_field(ctx: &mut ComputingParty) {
        if ctx.party_id == 0 {
            let input = vec![1, 0, 0, 1];
            let result = change_binary_to_decimal_field(&input, ctx, ctx.dt_training.dataset_size_prime);
            println!("{:?}", result);
        } else {
            let input = vec![0, 1, 1, 1];
            let result = change_binary_to_decimal_field(&input, ctx, ctx.dt_training.dataset_size_prime);
            println!("{:?}", result);
        }
    }

    pub fn test_change_binary_to_bigint_field(ctx: &mut ComputingParty) {
        if ctx.party_id == 0 {
            let input = vec![1, 1];
            let result = change_binary_to_bigint_field(&input, ctx);
            for item in result {
                println!("{}", item.to_string());
            }
        } else {
            let input = vec![1, 0];
            let result = change_binary_to_bigint_field(&input, ctx);
            for item in result {
                println!("{}", item.to_string());
            }
        }
    }

    pub fn test_argmax(ctx: &mut ComputingParty) {
        if ctx.party_id == 0 {
            let input = vec![vec![0, 0, 0, 0], vec![1, 0, 0, 0], vec![1, 0, 1, 0], vec![0, 0, 0, 1]];
            let result = arg_max(&input, ctx);
            println!("{:?}", result);
            let result_revealed = reveal_byte_vec_result(&result, ctx);
            println!("{:?}", result_revealed);
        } else {
            let input = vec![vec![0, 0, 0, 0], vec![0, 0, 0, 0], vec![0, 0, 0, 0], vec![0, 0, 0, 0]];
            let result = arg_max(&input, ctx);
            println!("{:?}", result);
            let result_revealed = reveal_byte_vec_result(&result, ctx);
            println!("{:?}", result_revealed);
        }
    }

    pub fn test_dot_product_integer(ctx: &mut ComputingParty) {
        if ctx.party_id == 0 {
            let x = vec![Wrapping(1 as u64), Wrapping(2 as u64), Wrapping(3 as u64), Wrapping(1 as u64)];
            let y = vec![Wrapping(1 as u64), Wrapping(0 as u64), Wrapping(2 as u64), Wrapping(1 as u64)];
            let result = dot_product_integer(&x, &y, ctx);
            println!("{:?}", result);
//            let result_revealed = reveal_int_result(&result, ctx);
//            println!("{:?}", result_revealed);
        } else {
            let x = vec![Wrapping(1 as u64), Wrapping(1 as u64), Wrapping(2 as u64), Wrapping(0 as u64)];
            let y = vec![Wrapping(0 as u64), Wrapping(0 as u64), Wrapping(0 as u64), Wrapping(0 as u64)];
            let result = dot_product_integer(&x, &y, ctx);
            println!("{:?}", result);
//            let result_revealed = reveal_int_result(&result, ctx);
//            println!("{:?}", result_revealed);
        }
    }

    pub fn test_batch_integer_equality(ctx: &mut ComputingParty) {
//        let mut result = Vec::new();
        let prime = ctx.dt_training.prime;
//        let shares_received = send_u64_messages(ctx,ctx.dt_shares.equality_integer_shares.get(&prime).unwrap());
//        let mut shares = Vec::new();
//        for i in 0..shares_received.len(){
//            shares.push((ctx.dt_shares.equality_integer_shares.get(&prime).unwrap()[i].0+shares_received[i].0).mod_floor(&prime));
//        }
//        println!("{:?}",shares);
        let mut temp = Vec::new();
        let mut x = Vec::new();
        let mut y = Vec::new();
        for i in 0..1 {
            if ctx.asymmetric_bit == 1 {
                x.push(Wrapping(15595373502427150336u64));
                y.push(Wrapping(9475888053360961536u64));
            } else {
                x.push(Wrapping(2851370571282402304u64));
                y.push(Wrapping(8970856020348591104u64));
            }
        }

        println!("{:?}", x);
        println!("{:?}", y);
        temp = batch_equality(&x, &y, ctx);
//        if ctx.party_id == 0 {
//            let x = vec![Wrapping(5 as u64), Wrapping(2 as u64), Wrapping(2 as u64), Wrapping(7 as u64)];
//            let y = vec![Wrapping(0 as u64), Wrapping(0 as u64), Wrapping(0 as u64), Wrapping(4 as u64)];
////            for i in 0..x.len() {
////                let temp = equality_integer_over_field(x[i].0, y[i].0, ctx, 3, 7);
////                println!("temp:{}", temp);
////            }
//            temp = batch_equality(&x, &y, ctx,4);
////                result = batch_equality_integer(&x, &y, ctx, prime);
////            let result_revealed = reveal_int_result(&result, ctx);
////            println!("{:?}", result_revealed);
//        } else {
//            let x = vec![Wrapping(7 as u64), Wrapping(6 as u64), Wrapping(2 as u64), Wrapping(1 as u64)];
//            let y = vec![Wrapping(0 as u64), Wrapping(0 as u64), Wrapping(0 as u64), Wrapping(4 as u64)];
////
////            for i in 0..x.len() {
////                let temp = equality_integer_over_field(x[i].0, y[i].0, ctx, 3, 7);
////                println!("temp:{}", temp);
////            }
//            temp = batch_equality(&x, &y, ctx,4);
////                result = batch_equality_integer(&x, &y, ctx, prime);
////            let result_revealed = reveal_int_result(&result, ctx);
////            println!("{:?}", result_revealed);
//        }
//        let result = send_u64_messages(ctx, &temp);
//        let result_shared:Vec<u64> = result.iter().zip(temp.iter()).map(|(a, b)| (a.0 ^ b.0)).collect();
        println!("{:?}", temp);
    }

    pub fn test_discretization(ctx: &mut ComputingParty) {
        let n = 11;

        let mut values = vec![
            Wrapping(6558502252784700000u64),
            Wrapping(8546166971241300000u64),
            Wrapping(16178756104899400000u64)

        ];

        if ctx.asymmetric_bit == 1 {
            values = vec![
                Wrapping(11888241820924800000u64),
                Wrapping(9900577102468270000u64),
                Wrapping(2267987968810130000u64)
            ];
        }

        let revealed = reveal(&values, ctx, ctx.decimal_precision, true, true);

        println!("TEST DISCRETIZE\nlist to discretize:      {:5?}", &revealed);
        let result = discretize(&values, 3, ctx);
        println!("discretization result:{:?}", reveal(&result, ctx, ctx.decimal_precision, true, true));

        let x = [result].to_vec();
        let category = 3 as usize;
        let rows = x.len();
        let cols = x[0].len();
        let mut equality_x = Vec::new();
        let mut equality_y = Vec::new();
        let decimal_mult = (2.0f64.powf(ctx.decimal_precision as f64) as u64);
        for j in 0..cols {
            for k in 0..category {
                let mut x_row = Vec::new();
                let mut y_row = Vec::new();

                for i in 0..rows {
                    x_row.push(x[i][j]);
                    if ctx.asymmetric_bit == 1 {
                        y_row.push(k as u64);
                    } else {
                        y_row.push(0u64);
                    }
                }
                equality_x.append(&mut x_row);
                equality_y.append(&mut y_row);
            }
        }
        println!("equality_x:{:?}", equality_x);
        println!("equality_y:{:?}", equality_y);

        let equality_y = binary_vector_to_ring(&equality_y, ctx);
        let equality_y = equality_y.iter().map(|a| Wrapping(a.0 << ctx.decimal_precision as u64)).collect();
        println!("equality_y:{:?}", equality_y);

        let eq_result = batch_equality(&equality_x, &equality_y, ctx);
        let eq_result = eq_result.iter().map(|a| Wrapping(*a)).collect();
        println!("eq_result:{:?}", eq_result);
        let eq_received = send_u64_messages(ctx, &eq_result);
        let eq_result_revealed: Vec<u64> = eq_result.iter().zip(eq_received).map(|(a, b)| a.0 ^ b.0).collect();
        println!("eq_result_revealed:{:?}", eq_result_revealed);

//        let b =
        let mut a = Vec::new();
        let mut b = Vec::new();
//
        for i in 9..14 {
            if ctx.asymmetric_bit == 1 {
                a.push(i as u64);
            } else {
                a.push(0 as u64);
            }
        }
        for i in [13, 12, 11, 10, 9].to_vec() {
            if ctx.asymmetric_bit == 1 {
                b.push(i as u64);
            } else {
                b.push(0 as u64);
            }
        }
        let a = binary_vector_to_ring(&a, ctx);
        let b = binary_vector_to_ring(&b, ctx);
//
        let d = batch_equality(&a, &b, ctx);
        println!("d:{:?}", d);
//        let b_received = send_u64_messages(ctx, &b);
//        let prime = (2.0f64.powf(64.0) as u64);
//
//        let c:Vec<u64>= b.iter().zip(b_received.iter()).map(|(a,b)|(a+b).0-prime).collect();
//        println!("c:{:?}", c);
//        let a_received = send_u64_messages(ctx, &a);
//        let a_combined:Vec<u64> = a.iter().zip(a_received).map(|(a,b)|(a+b).0.mod_floor(&prime)).collect();
//        println!("a_combined:{:?}",a_combined);
    }
}
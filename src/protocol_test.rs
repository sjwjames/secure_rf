pub mod protocol_test {
    use crate::computing_party::computing_party::ComputingParty;
    use std::num::Wrapping;
    use crate::multiplication::multiplication::{multiplication_byte, batch_multiplication_byte, batch_multiplication_integer, multiplication_bigint, multi_thread_batch_mul_byte, parallel_multiplication, batch_multiply_bigint, parallel_multiplication_big_integer};
    use crate::utils::utils::{reveal_byte_result, reveal_byte_vec_result, reveal_int_vec_result, reveal_bigint_result, reveal_bigint_vec_result};
    use rand::{random, Rng};
    use num::integer::*;
    use num::{BigUint, FromPrimitive, ToPrimitive, Zero, One};
    use std::cmp::max;
    use crate::protocol::protocol::equality_big_integer;
    use crate::comparison::comparison::{compare_bigint, comparison, compute_e_shares, compute_d_shares, compute_multi_e_parallel, compute_c_shares};
    use std::ops::BitAnd;
    use crate::bit_decomposition::bit_decomposition::bit_decomposition;

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
        let mut result_vec: Vec<u8> = Vec::new();
        let mut x_pub_vec: Vec<u8> = Vec::new();
        let mut y_pub_vec: Vec<u8> = Vec::new();

        for i in 0..2 {
            for j in 0..2 {
                for m in 0..2 {
                    for n in 0..2 {
                        if ctx.party_id == 0 {
                            x_vec.push(i as u8);
                            y_vec.push(j as u8);
                        } else {
                            x_vec.push(m as u8);
                            y_vec.push(n as u8);
                        }
                        x_pub_vec.push(i ^ m);
                        y_pub_vec.push(j ^ n);
                        result_vec.push((i ^ m) * (j ^ n));
                    }
                }
            }
        }
        println!("x_pub_vec {:?}", x_pub_vec);
        println!("y_pub_vec {:?}", y_pub_vec);
        println!("result_vec {:?}", result_vec);
        let result = batch_multiplication_byte(&x_vec, &y_vec, ctx);
        println!("computed result {:?}", &result);

        let result_revealed = reveal_byte_vec_result(&result, ctx);
        println!("result_revealed {:?}", result_revealed);

        assert!(result_vec.iter().zip(result_revealed.iter()).all(|(a, b)| a == b), "Arrays are not equal");
    }

    pub fn test_batch_multiplication_integer(ctx: &mut ComputingParty) {
        let mut x_vec: Vec<Wrapping<u64>> = Vec::new();
        let mut y_vec: Vec<Wrapping<u64>> = Vec::new();
        let mut x_vec_pub: Vec<Wrapping<u64>> = Vec::new();
        let mut y_vec_pub: Vec<Wrapping<u64>> = Vec::new();
        let mut result_pub: Vec<Wrapping<u64>> = Vec::new();

        for i in 0..10 {
            x_vec_pub.push(Wrapping(i));
            y_vec_pub.push(Wrapping(i));
            if ctx.party_id == 0 {
                x_vec.push(Wrapping(1));
                y_vec.push(Wrapping(1));
            } else {
                x_vec.push(Wrapping((Wrapping(i) - Wrapping(1)).0.mod_floor(&ctx.dt_training.dataset_size_prime)));
                y_vec.push(Wrapping((Wrapping(i) - Wrapping(1)).0.mod_floor(&ctx.dt_training.dataset_size_prime)));
            }

            result_pub.push(Wrapping((i * i).mod_floor(&ctx.dt_training.dataset_size_prime)));
        }
        println!("result_pub {:?}", &result_pub);
        let result = batch_multiplication_integer(&x_vec, &y_vec, ctx);
        println!("result computed {:?}", &result);

        let result_revealed = reveal_int_vec_result(&result, ctx);
        println!("result_revealed {:?}", &result_revealed);
        assert!(result_pub.iter().zip(result_revealed.iter()).all(|(a, b)| a.0 == *b), "Arrays are not equal");
    }

    pub fn test_or_xor(ctx: &mut ComputingParty) {}

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
            println!("result_pub: {}", result_pub[i].to_usize().unwrap());
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


        println!("x_pub_vec {:?}", x_pub_vec);
        println!("y_pub_vec {:?}", y_pub_vec);
        println!("result_vec {:?}", result_vec);
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
            let x = BigUint::from_u32(5).unwrap();
            let y = BigUint::from_u32(3).unwrap();
            result = equality_big_integer(&x, &y, ctx);
        } else {
            let x = BigUint::from_u32(5).unwrap();
            let y = BigUint::from_u32(7).unwrap();
            result = equality_big_integer(&x, &y, ctx);
        }
        let result_revealed = reveal_bigint_result(&result, ctx);
        println!("{}", result_revealed.to_string());
        assert_eq!(result_revealed, BigUint::zero());
    }

    pub fn test_comparison(ctx: &mut ComputingParty) {
        let mut result = 0;
        for i in 0..10 {
            if ctx.party_id == 0 {
                let x: Vec<u8> = vec![1, 0, 1, 0];
                let y: Vec<u8> = vec![0, 1, 0, 0];
//            let e_shares = compute_e_shares(&x, &y, ctx);
//            assert_eq!(e_shares, [1, 0, 1, 0]);
//            let d_shares = compute_d_shares(&x, &y, ctx);
//            let revealed_d_shares = reveal_byte_vec_result(&d_shares, ctx);
//            assert_eq!(revealed_d_shares, [1, 1, 0, 0]);
//            let e_reverse = compute_multi_e_parallel(&x, &y, ctx, &e_shares);
//            let e_reverse_revealed = reveal_byte_vec_result(&e_reverse, ctx);
//            println!("{:?}",e_reverse);
//            assert_eq!(e_reverse_revealed, [0, 1, 0, 1]);
//            let c_share = compute_c_shares(x.len(), &e_reverse, &d_shares, ctx);
//            println!("c_share:{:?}",c_share);
//            let c_share_revealed = reveal_byte_vec_result(&c_share, ctx);
//            assert_eq!(c_share_revealed,[0,0,0,0]);
                result = comparison(&x, &y, ctx);
            } else {
                let x: Vec<u8> = vec![0, 0, 0, 0];
                let y: Vec<u8> = vec![1, 0, 0, 0];
//            let e_shares = compute_e_shares(&x, &y, ctx);
//            assert_eq!(e_shares, [1, 0, 0, 1]);
//            let d_shares = compute_d_shares(&x, &y, ctx);
//            let revealed_d_shares = reveal_byte_vec_result(&d_shares, ctx);
//            assert_eq!(revealed_d_shares, [1, 1, 0, 0]);
//            let e_reverse = compute_multi_e_parallel(&x, &y, ctx, &e_shares);
//            let e_reverse_revealed = reveal_byte_vec_result(&e_reverse, ctx);
//            println!("{:?}",e_reverse);
//            assert_eq!(e_reverse_revealed, [0, 1, 1, 0]);
//            let c_share = compute_c_shares(x.len(), &e_reverse, &d_shares, ctx);
//            println!("c_share:{:?}",c_share);
//            let c_share_revealed = reveal_byte_vec_result(&c_share, ctx);
//            assert_eq!(c_share_revealed,[0,0,0,0]);
                result = comparison(&x, &y, ctx);
            }

            let result_revealed = reveal_byte_result(result, ctx);
            println!("{}", result_revealed);
            assert_eq!(result_revealed, 0);
        }
    }


    pub fn test_comparison_bigint(ctx: &mut ComputingParty) {
        let mut result = BigUint::zero();
        if ctx.party_id == 0 {
            let x = BigUint::from_u32(10).unwrap();
            let y = BigUint::from_u32(3).unwrap();
            result = compare_bigint(&x, &y, ctx);
        } else {
            let x = BigUint::from_u32(5).unwrap();
            let y = BigUint::from_u32(7).unwrap();
            result = compare_bigint(&x, &y, ctx);
        }
        let result_revealed = reveal_bigint_result(&result, ctx);
        println!("{}", result_revealed.to_string());
        assert_eq!(result_revealed, BigUint::one());
    }

    pub fn test_bit_decomposition(ctx: &mut ComputingParty) {
        let input = 4;
        let bit_decomposed = bit_decomposition(input, ctx);
        let bit_decomposed_revealed = reveal_byte_vec_result(&bit_decomposed, ctx);
        println!("{:?}", bit_decomposed_revealed);
    }
}
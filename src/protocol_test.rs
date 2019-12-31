pub mod protocol_test {
    use crate::computing_party::computing_party::ComputingParty;
    use std::num::Wrapping;
    use crate::multiplication::multiplication::{multiplication_byte, batch_multiplication_byte, batch_multiplication_integer};
    use crate::utils::utils::{reveal_byte_result, reveal_byte_vec_result, reveal_int_vec_result};
    use rand::{random, Rng};
    use num::integer::*;

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
        let mut x_pub_vec:Vec<u8> = Vec::new();
        let mut y_pub_vec:Vec<u8> = Vec::new();

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
        println!("x_pub_vec {:?}",x_pub_vec);
        println!("y_pub_vec {:?}",y_pub_vec);
        println!("result_vec {:?}",result_vec);
        let result = batch_multiplication_byte(&x_vec,&y_vec,ctx);
        println!("computed result {:?}",&result);

        let result_revealed = reveal_byte_vec_result(&result,ctx);
        println!("result_revealed {:?}",result_revealed);

        assert!(result_vec.iter().zip(result_revealed.iter()).all(|(a,b)| a == b), "Arrays are not equal");
    }

    pub fn test_batch_multiplication_integer(ctx: &mut ComputingParty){
        let mut x_vec:Vec<Wrapping<u64>> = Vec::new();
        let mut y_vec:Vec<Wrapping<u64>> = Vec::new();
        let mut x_vec_pub:Vec<Wrapping<u64>> = Vec::new();
        let mut y_vec_pub:Vec<Wrapping<u64>> = Vec::new();
        let mut result_pub:Vec<Wrapping<u64>> = Vec::new();
        let mut rng = rand::thread_rng();



        for i in 0..3{
            x_vec_pub.push(Wrapping(i));
            y_vec_pub.push(Wrapping(i));
            if ctx.party_id==0 {
                x_vec.push(Wrapping(1));
                y_vec.push(Wrapping(1));
            }else{
                x_vec.push(Wrapping((Wrapping(i)-Wrapping(1)).0.mod_floor(&ctx.dt_training.dataset_size_prime)));
                y_vec.push(Wrapping((Wrapping(i)-Wrapping(1)).0.mod_floor(&ctx.dt_training.dataset_size_prime)));
            }

            result_pub.push(Wrapping((i*i).mod_floor(&ctx.dt_training.dataset_size_prime)));
        }
        println!("result_pub {:?}",&result_pub);
        let result = batch_multiplication_integer(&x_vec,&y_vec,ctx);
        println!("result computed {:?}",&result);

        let result_revealed = reveal_int_vec_result(&result,ctx);
        println!("result_revealed {:?}",&result_revealed);
        assert!(result_pub.iter().zip(result_revealed.iter()).all(|(a,b)| a.0 == *b), "Arrays are not equal");

    }

    pub fn test_or_xor(ctx: &mut ComputingParty){

    }

}
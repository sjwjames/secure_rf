pub mod decision_tree {
    use crate::computing_party::computing_party::ComputingParty;
    use num::bigint::{BigInt, BigUint};
    use num::integer::*;
    use std::io::{Bytes, Write, BufReader, BufRead};
    use serde::{Serialize, Deserialize, Serializer};
    use std::num::Wrapping;
    use crate::utils::utils::{big_uint_clone, push_message_to_queue, receive_message_from_queue, big_uint_vec_clone, serialize_biguint_vec, serialize_biguint, deserialize_biguint, reveal_bigint_result, reveal_byte_vec_result, send_u8_messages, send_biguint_messages, send_receive_u64_matrix, send_u64_messages};
    use threadpool::ThreadPool;
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;
    //    use crate::dot_product::dot_product::dot_product;
    use crate::field_change::field_change::{change_binary_to_decimal_field, change_binary_to_bigint_field};
    use std::thread::sleep;
    use std::time::{Duration, SystemTime};
    use crate::dot_product::dot_product::{dot_product, dot_product_integer, dot_product_bigint};
    use crate::bit_decomposition::bit_decomposition::bit_decomposition;
    use crate::protocol::protocol::{arg_max, equality_big_integer, arg_max_ring, batch_equality};
    use crate::constants::constants::BINARY_PRIME;
    use crate::message::message::{RFMessage, search_pop_message};
    use crate::multiplication::multiplication::{batch_multiply_bigint, multiplication_bigint, batch_multiplication_byte, parallel_multiplication_big_integer};
    use num::{Zero, ToPrimitive, One, FromPrimitive};
    use std::ops::{Add, Mul};
    use crate::comparison::comparison::compare_bigint;
    use std::str::FromStr;
    use std::fs::File;
    use crate::discretize::discretize::binary_vector_to_ring;

    pub struct DecisionTreeData {
        pub attr_value_count: usize,
        pub class_value_count: usize,
        pub attribute_count: usize,
        pub instance_count: usize,
        pub attr_values: Vec<Vec<Vec<Wrapping<u64>>>>,
        pub class_values: Vec<Vec<Wrapping<u64>>>,
        pub attr_values_trans_vec: Vec<Vec<Vec<u8>>>,
        pub class_values_trans_vec: Vec<Vec<u8>>,
        pub attr_values_big_integer: Vec<Vec<Vec<BigUint>>>,
        pub class_values_big_integer: Vec<Vec<BigUint>>,
        pub discretized_x: Vec<Vec<Wrapping<u64>>>,
        pub discretized_y: Vec<u64>,
    }

    pub struct DecisionTreeTraining {
        pub max_depth: usize,
        pub alpha: BigUint,
        pub epsilon: f64,
        pub cutoff_transaction_set_size: usize,
        pub subset_transaction_bit_vector: Vec<u8>,
        pub attribute_bit_vector: Vec<u8>,
        pub prime: u64,
        pub big_int_prime: BigUint,
        pub dataset_size_prime: u64,
        pub dataset_size_bit_length: u64,
        pub bit_length: u64,
        pub big_int_ti_index: u64,
        pub rfs_field: u64,
        pub bagging_field: u64,
    }

    pub struct DecisionTreeShares {
        pub additive_triples: HashMap<u64, Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)>>,
        pub additive_bigint_triples: Vec<(BigUint, BigUint, BigUint)>,
        pub rfs_shares: Vec<Vec<Wrapping<u64>>>,
        pub bagging_shares: Vec<Vec<Wrapping<u64>>>,
        pub binary_triples: Vec<(u8, u8, u8)>,
        pub equality_shares: Vec<(BigUint)>,
        pub equality_integer_shares: HashMap<u64, Vec<Wrapping<u64>>>,
        pub current_additive_index: Arc<Mutex<usize>>,
        pub current_additive_bigint_index: Arc<Mutex<usize>>,
        pub current_equality_index: Arc<Mutex<usize>>,
        pub current_binary_index: Arc<Mutex<usize>>,
        pub sequential_additive_index: HashMap<u64, usize>,
        pub sequential_additive_bigint_index: usize,
        pub sequential_equality_index: usize,
        pub sequential_binary_index: usize,
        pub sequential_equality_integer_index: HashMap<u64, usize>,
        pub sequential_ohe_additive_index: usize,
        pub matrix_mul_shares: (Vec<Vec<Wrapping<u64>>>, Vec<Vec<Wrapping<u64>>>, Vec<Vec<Wrapping<u64>>>),
        pub bagging_matrix_mul_shares: (Vec<Vec<Wrapping<u64>>>, Vec<Vec<Wrapping<u64>>>, Vec<Vec<Wrapping<u64>>>),
    }


    #[derive(Serialize, Deserialize, Debug)]
    pub struct DecisionTreeTIShareMessage {
        pub additive_triples: String,
        pub additive_bigint_triples: String,
        pub binary_triples: String,
        pub equality_shares: String,
        pub rfs_shares: String,
        pub bagging_shares: String,
        pub matrix_mul_shares: String,
        pub bagging_matrix_mul_shares: String,
        pub equality_integer_shares: String,
    }

    pub struct DecisionTreeResult {
        pub result_list: Vec<String>
    }

    impl Clone for DecisionTreeResult {
        fn clone(&self) -> Self {
            DecisionTreeResult {
                result_list: self.result_list.clone()
            }
        }
    }

    impl Clone for DecisionTreeData {
        fn clone(&self) -> Self {
            DecisionTreeData {
                attr_value_count: self.attr_value_count,
                class_value_count: self.class_value_count,
                attribute_count: self.attribute_count,
                instance_count: self.instance_count,
                attr_values: self.attr_values.clone(),
                class_values: self.class_values.clone(),
                attr_values_trans_vec: self.attr_values_trans_vec.clone(),
                class_values_trans_vec: self.class_values_trans_vec.clone(),
                attr_values_big_integer: self.attr_values_big_integer.clone(),
                class_values_big_integer: self.class_values_big_integer.clone(),
                discretized_x: self.discretized_x.clone(),
                discretized_y: self.discretized_y.clone(),
            }
        }
    }

    impl Clone for DecisionTreeTraining {
        fn clone(&self) -> Self {
            DecisionTreeTraining {
                max_depth: self.max_depth,
                alpha: self.alpha.clone(),
                epsilon: self.epsilon.clone(),
                cutoff_transaction_set_size: self.cutoff_transaction_set_size,
                subset_transaction_bit_vector: self.subset_transaction_bit_vector.clone(),
                attribute_bit_vector: self.attribute_bit_vector.clone(),
                prime: self.prime,
                big_int_prime: big_uint_clone(&self.big_int_prime),
                dataset_size_prime: self.dataset_size_prime,
                dataset_size_bit_length: self.dataset_size_bit_length,
                bit_length: self.bit_length,
                big_int_ti_index: self.big_int_ti_index,
                rfs_field: self.rfs_field,
                bagging_field: self.bagging_field,
            }
        }
    }


    impl Clone for DecisionTreeShares {
        fn clone(&self) -> Self {
            let mut additive_bigint_triples = Vec::new();
            let mut binary_triples = Vec::new();
            let mut equality_shares = Vec::new();
            let mut rfs_shares = Vec::new();
            let mut bagging_shares = Vec::new();

            for item in self.rfs_shares.iter() {
                rfs_shares.push(item.clone());
            }

            for item in self.bagging_shares.iter() {
                bagging_shares.push(item.clone());
            }


            for item in self.binary_triples.iter() {
                binary_triples.push((item.0.clone(), item.1.clone(), item.2.clone()));
            }

            for item in self.additive_bigint_triples.iter() {
                additive_bigint_triples.push((BigUint::from_bytes_le(&item.0.to_bytes_le().clone()),
                                              BigUint::from_bytes_le(&item.1.to_bytes_le().clone()),
                                              BigUint::from_bytes_le(&item.2.to_bytes_le().clone())));
            }
            for item in self.equality_shares.iter() {
                equality_shares.push(BigUint::from_bytes_le(&item.to_bytes_le().clone()));
            }

            DecisionTreeShares {
                additive_triples: self.additive_triples.clone(),
                additive_bigint_triples,
                rfs_shares,
                bagging_shares,
                binary_triples,
                equality_shares,
                equality_integer_shares: self.equality_integer_shares.clone(),
                current_additive_index: Arc::clone(&self.current_additive_index),
                current_additive_bigint_index: Arc::clone(&self.current_additive_bigint_index),
                current_equality_index: Arc::clone(&self.current_equality_index),
                current_binary_index: Arc::clone(&self.current_binary_index),
                sequential_additive_index: self.sequential_additive_index.clone(),
                sequential_additive_bigint_index: self.sequential_additive_bigint_index,
                sequential_equality_index: self.sequential_equality_index,
                sequential_binary_index: self.sequential_binary_index,
                sequential_equality_integer_index: self.sequential_equality_integer_index.clone(),
                sequential_ohe_additive_index: self.sequential_ohe_additive_index,
                matrix_mul_shares: (self.matrix_mul_shares.0.clone(), self.matrix_mul_shares.1.clone(), self.matrix_mul_shares.2.clone()),
                bagging_matrix_mul_shares: (self.bagging_matrix_mul_shares.0.clone(), self.bagging_matrix_mul_shares.1.clone(), self.bagging_matrix_mul_shares.2.clone()),
            }
        }
    }

    fn same_class_stop_check(major_class_index: usize, ctx: &mut ComputingParty) -> u64 {
        let mut subset_transaction: Vec<u64> = ctx.dt_training.subset_transaction_bit_vector.iter().map(|x| *x as u64).collect();
        let mut subset_transaction: Vec<Wrapping<u64>> = binary_vector_to_ring(&subset_transaction, ctx).iter().map(|x| Wrapping(x.0 << ctx.decimal_precision as u64)).collect();

//        let mut transactions_decimal = change_binary_to_bigint_field(&subset_transaction, ctx);

        let class_value_count = ctx.dt_data.class_value_count;
        let dataset_size = ctx.instance_selected as usize;

        let mut major_class_index_ring = if ctx.asymmetric_bit == 1 {
            vec![Wrapping(1u64 << ctx.decimal_precision as u64); dataset_size]
        } else {
            vec![Wrapping(0u64); dataset_size]
        };

        let mut major_class_trans_count = dot_product(&ctx.dt_data.class_values[major_class_index].clone(), &major_class_index_ring, ctx, ctx.decimal_precision, false, false);

        println!("Majority Class Transaction Count: {}", major_class_trans_count.to_string());

        let mut transaction_count = BigUint::zero();
        let list = if ctx.asymmetric_bit == 1 {
            vec![Wrapping(1u64 << ctx.decimal_precision as u64); dataset_size]
        } else {
            vec![Wrapping(0u64); dataset_size]
        };
        let transaction_count = dot_product(&subset_transaction, &list, ctx, ctx.decimal_precision, false, false);
        println!("Transactions in current subset: {}", transaction_count.to_string());
        let eq_test_result = batch_equality(&[major_class_trans_count].to_vec(), &[transaction_count].to_vec(), ctx);

        let received_eq_result = send_u64_messages(ctx, &[Wrapping(eq_test_result[0])].to_vec());
        let stopping_check = received_eq_result[0].0 ^ eq_test_result[0];
        stopping_check
    }


    pub fn train(ctx: &mut ComputingParty, r: usize, result_file: &mut File) -> DecisionTreeResult {
        println!("start building model");
        if ctx.debug_output {
//            let current_transactions = ctx.dt_training.subset_transaction_bit_vector.clone();
//            let revealed = reveal_byte_vec_result(&current_transactions, ctx);
//            println!("current transactions{:?}", revealed);
        }
        let major_class_index = find_common_class_index(ctx);
        // Make majority class index one-hot encoding public
        // Share major class index

        let mut major_class_index_receive: Vec<u8> = Vec::new();
        major_class_index_receive = send_u8_messages(ctx, &major_class_index);


        let class_value_count = ctx.dt_data.class_value_count;
        let mut major_class_index_shared = vec![0u8; class_value_count];
        for i in 0..class_value_count {
            major_class_index_shared[i] = mod_floor((Wrapping((&major_class_index)[i]) + Wrapping((&major_class_index_receive)[i])).0, BINARY_PRIME as u8);
        }

        let mut major_index = 0;
        for i in 0..class_value_count {
            if major_class_index_shared[i] == 1 {
                major_index = i;
                break;
            }
        }

        for i in major_index + 1..class_value_count {
            major_class_index_shared[i] = 0;
        }

        println!("Major class {}", major_index);
        println!("Major class OHE {:?}", major_class_index_shared);

        // quit if reaching max depth
        if r == 0 {
            println!("Exited on base case: Recursion Level == 0");
            if ctx.asymmetric_bit == 1 {
                result_file.write_all(format!("class={},", major_index).as_bytes());
            }
            return ctx.dt_results.clone();
        }


        let mut subset_transaction = ctx.dt_training.subset_transaction_bit_vector.clone();
        let class_value_count = ctx.dt_data.class_value_count;
        let dataset_size = ctx.instance_selected as usize;
        let mut bigint_prime = big_uint_clone(&ctx.dt_training.big_int_prime);

        let stopping_check = same_class_stop_check(major_index, ctx);
        if stopping_check == 1 {
            println!("Exited on base case: All transactions predict same outcome");
//            ctx.dt_results.result_list.push(format!("class={}", major_index));
            if ctx.asymmetric_bit == 1 {
                let n = (((ctx.dt_data.attr_value_count as f64).powf((ctx.dt_training.max_depth-r+1) as f64)-1.0)/(ctx.dt_data.attr_value_count as f64 - 1.0)) as usize;
                for i in 0..n{
                    result_file.write_all(format!("class={},", major_index).as_bytes());
                }
            }

            return ctx.dt_results.clone();
        }
        println!("Base case not reached. Continuing.");

        let mut class_value_trans_vector = ctx.dt_data.class_values_trans_vec.clone();
        let mut u_list = Vec::new();
        //todo multi-thread
        for i in 0..class_value_count {
            let batch_multi_result = batch_multiplication_byte(&subset_transaction, &class_value_trans_vector[i], ctx);
            u_list.push(batch_multi_result);
        }

        //todo multi-thread
        let mut u_decimal = Vec::new();
        for i in 0..class_value_count {
            let bigint_u = change_binary_to_bigint_field(&u_list[i], ctx);
            u_decimal.push(bigint_u);
        }


        let attr_count = ctx.dt_data.attribute_count;
        let attr_val_count = ctx.dt_data.attr_value_count;
        let mut x: Vec<Vec<Vec<BigUint>>> = vec![vec![vec![BigUint::zero(); attr_val_count]; class_value_count]; attr_count];
        let mut x2: Vec<Vec<Vec<BigUint>>> = vec![vec![vec![BigUint::zero(); attr_val_count]; class_value_count]; attr_count];
        let mut y: Vec<Vec<BigUint>> = vec![vec![BigUint::zero(); attr_val_count]; attr_count];
        let mut gini_numerators: Vec<BigUint> = vec![BigUint::zero(); attr_count];
        let mut gini_denominators: Vec<BigUint> = vec![BigUint::zero(); attr_count];
        let mut attributes = ctx.dt_training.attribute_bit_vector.clone();
        let attr_value_trans_bigint_vec = &ctx.dt_data.attr_values_big_integer;
        let alpha = &ctx.dt_training.alpha;

        for k in 0..attr_count {
            if attributes[k] != 0 {
                for j in 0..attr_val_count {
                    // Determine the number of transactions that are:
                    // 1. in the current subset
                    // 2. predict the i-th class value
                    // 3. and have the j-th value of the k-th attribute

                    for i in 0..class_value_count {
                        let mut ctx_cp = ctx.clone();
                        let u_decimal_cp = big_uint_vec_clone(&u_decimal[i]);
                        let attr_value_trans_bigint = big_uint_vec_clone(&attr_value_trans_bigint_vec[k][j]);

                        let dp_result = dot_product_bigint(&u_decimal_cp, &attr_value_trans_bigint, &mut ctx_cp);
                        x[k][i][j] = big_uint_clone(&dp_result);
                        let y_temp = big_uint_clone(&y[k][j]);
                        y[k][j] = y_temp.add(&dp_result).mod_floor(&bigint_prime);
                    }


                    y[k][j] = alpha.mul(&y[k][j]).add(if ctx.asymmetric_bit == 1 {
                        BigUint::one()
                    } else {
                        BigUint::zero()
                    }).mod_floor(&bigint_prime);
                }

                let mut ctx_cp = ctx.clone();
                for i in 0..class_value_count {
                    let batch_multi_result = batch_multiply_bigint(&x[k][i], &x[k][i], &mut ctx_cp);
                    x2[k][i] = batch_multi_result;
                }


                let mut ctx_cloned = ctx.clone();
                let p_multi = parallel_multiplication_big_integer(&y[k], &mut ctx_cloned);
                gini_denominators[k] = p_multi;

                gini_numerators[k] = BigUint::zero();
                for j in 0..attr_val_count {
                    let mut y_without_j = Vec::new();
                    for l in 0..y[k].len() {
                        if j != l {
                            y_without_j.push(big_uint_clone(&y[k][l]));
                        }
                    }
                    let mut ctx = ctx.clone();
                    let mut y_prod_without_j = parallel_multiplication_big_integer(&y_without_j, &mut ctx);
                    let mut sum_x2 = BigUint::zero();
                    for i in 0..class_value_count {
                        sum_x2 = sum_x2.add(&x2[k][i][j]).mod_floor(&bigint_prime);
                    }
                    let mut ctx = ctx.clone();
                    let mut multi0 = multiplication_bigint(&sum_x2, &y_prod_without_j, &mut ctx);
                    let mut temp_gini_numerator = &gini_numerators[k];
                    gini_numerators[k] = temp_gini_numerator.add(&multi0).mod_floor(&bigint_prime);
                }
            }
        }

        let mut k = 0;
        while attributes[k] == 0 {
            k += 1;
        }

        let mut gini_max_numerator = big_uint_clone(&gini_numerators[k]);
        let mut gini_max_denominator = big_uint_clone(&gini_denominators[k]);
        let mut gini_argmax = if ctx.asymmetric_bit == 1 { BigUint::from(k) } else { BigUint::zero() };
        k += 1;
        while k < attr_count {
            if attributes[k] == 1 {
                let mut gini_argmaxes = [if ctx.asymmetric_bit == 1 { BigUint::from(k) } else { BigUint::zero() }, big_uint_clone(&gini_argmax)];
                let mut numerators = [big_uint_clone(&gini_numerators[k]), big_uint_clone(&gini_max_numerator)];
                let mut denominators = [big_uint_clone(&gini_denominators[k]), big_uint_clone(&gini_max_denominator)];
                let mut new_assignments_bin = vec![BigUint::zero(); 2];

                let mult0 = multiplication_bigint(&numerators[0], &denominators[1], ctx);
                let mult1 = multiplication_bigint(&numerators[1], &denominators[0], ctx);
                let mut left_operand = mult0;
                let mut right_operand = mult1;

                new_assignments_bin[0] = compare_bigint(&left_operand, &right_operand, ctx).mod_floor(&BigUint::from_usize(BINARY_PRIME).unwrap());
                new_assignments_bin[1] = big_uint_clone(&new_assignments_bin[0]).add(&(if ctx.asymmetric_bit == 1 { BigUint::one() } else { BigUint::zero() })).mod_floor(&BigUint::from_usize(BINARY_PRIME).unwrap());
                let new_assignments_bin_converted = [new_assignments_bin[0].to_u8().unwrap(), new_assignments_bin[1].to_u8().unwrap()].to_vec();
                let new_assignments_bin_converted_big_binary = change_binary_to_bigint_field(&new_assignments_bin_converted, ctx);

                gini_argmax = dot_product_bigint(&new_assignments_bin_converted_big_binary, &gini_argmaxes.to_vec(), ctx);
                println!("gini_argmax:{}", gini_argmax.to_string());
                gini_max_numerator = dot_product_bigint(&new_assignments_bin_converted_big_binary, &numerators.to_vec(), ctx);
                gini_max_denominator = dot_product_bigint(&new_assignments_bin_converted_big_binary, &denominators.to_vec(), ctx);
            }
            k += 1;
        }

        let mut argmax_received = BigUint::zero();
        let list_received = send_biguint_messages(ctx, &[big_uint_clone(&gini_argmax)].to_vec());
        argmax_received = big_uint_clone(&list_received[0]);

        let shared_gini_argmax = gini_argmax.add(&argmax_received).mod_floor(&bigint_prime);
        let mut shared_gini_argmax = shared_gini_argmax.to_usize().unwrap();
        attributes[shared_gini_argmax] = 0;
        if ctx.asymmetric_bit == 1 {
            result_file.write_all(format!("attr={},", shared_gini_argmax).as_bytes());
        }
//        ctx.dt_results.result_list.push(format!("attr={}", shared_gini_argmax));
        ctx.dt_training.attribute_bit_vector = attributes;

        let mut dt_ctx = ctx.clone();
        let mut result_map = HashMap::new();
        let attr_values_trans_vec = &ctx.dt_data.attr_values_trans_vec;
        for j in 0..attr_val_count {
            let batch_multi = batch_multiplication_byte(&subset_transaction, &attr_values_trans_vec[shared_gini_argmax][j], &mut dt_ctx);
            result_map.insert(j, batch_multi);
        }
        for j in 0..attr_val_count {
            let mut dt_ctx = ctx.clone();
            dt_ctx.dt_training.subset_transaction_bit_vector = result_map.get(&j).unwrap().clone();
            train(&mut dt_ctx, r - 1, result_file);
        }

        ctx.dt_results.clone()
    }

    fn find_common_class_index(ctx: &mut ComputingParty) -> Vec<u8> {
        let mut now = SystemTime::now();
        let mut subset_transaction_bit_vector = ctx.dt_training.subset_transaction_bit_vector.clone();
//        let mut subset_decimal = change_binary_to_decimal_field(&subset_transaction_bit_vector, ctx, ctx.dt_training.dataset_size_prime);
        let mut subset_decimal = binary_vector_to_ring(&subset_transaction_bit_vector.iter().map(|x| *x as u64).collect(), ctx).iter().map(|x| Wrapping(x.0 << ctx.decimal_precision as u64)).collect();

        let mut s = Vec::new();
        let mut argmax_result = Vec::new();
        let mut class_value_transaction = ctx.dt_data.class_values.clone();
//            let received = send_receive_u64_matrix(&class_value_transaction,ctx);
//            let combined:Vec<Vec<u64>> = received.iter().zip(class_value_transaction.iter()).map(|(a,b)|a.iter().zip(b.iter()).map(|(c,d)|(c.0+d.0).mod_floor(&(ctx.dt_training.dataset_size_prime as u64))).collect()).collect();
//            println!("combined:{:?}",combined);
        for i in 0..ctx.dt_data.class_value_count {
            let dp_result = dot_product(&subset_decimal, &class_value_transaction[i], ctx, ctx.decimal_precision, false, false);
            s.push(dp_result);
        }
        argmax_result = arg_max_ring(&s, ctx);
//            for i in 0..ctx.dt_data.class_value_count {
//                let s_copied = s[i];
//                let bd_result = bit_decomposition(s[i], ctx, ctx.dt_training.dataset_size_bit_length as usize);
//                bit_shares.push(bd_result);
//            }
//            argmax_result = arg_max(&bit_shares, ctx);
        println!("find common class index completes in {}ms", now.elapsed().unwrap().as_millis());
        argmax_result
    }
}
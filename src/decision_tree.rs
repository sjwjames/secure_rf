pub mod decision_tree {
    use crate::computing_party::computing_party::ComputingParty;
    use num::bigint::{BigInt, BigUint};
    use num::integer::*;
    use std::io::{Bytes, Write, BufReader, BufRead};
    use serde::{Serialize, Deserialize, Serializer};
    use std::num::Wrapping;
    use crate::utils::utils::{big_uint_clone, push_message_to_queue, receive_message_from_queue, big_uint_vec_clone, serialize_biguint_vec, serialize_biguint, deserialize_biguint, reveal_bigint_result};
    use threadpool::ThreadPool;
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;
    //    use crate::dot_product::dot_product::dot_product;
    use crate::field_change::field_change::{change_binary_to_decimal_field, change_binary_to_bigint_field};
    use std::thread::sleep;
    use std::time::{Duration, SystemTime};
    use crate::dot_product::dot_product::{dot_product, dot_product_integer, dot_product_bigint};
    use crate::bit_decomposition::bit_decomposition::bit_decomposition;
    use crate::protocol::protocol::{arg_max, equality_big_integer};
    use crate::constants::constants::BINARY_PRIME;
    use crate::message::message::{RFMessage, search_pop_message};
    use crate::multiplication::multiplication::{batch_multiply_bigint, multiplication_bigint, batch_multiplication_byte, parallel_multiplication_big_integer};
    use num::{Zero, ToPrimitive, One};
    use std::ops::{Add, Mul};
    use crate::comparison::comparison::compare_bigint;
    use std::str::FromStr;

    pub struct DecisionTreeData {
        pub attr_value_count: usize,
        pub class_value_count: usize,
        pub attribute_count: usize,
        pub instance_count: usize,
        pub attr_values: Vec<Vec<Vec<Wrapping<u64>>>>,
        pub class_values: Vec<Vec<Wrapping<u64>>>,
        pub attr_values_bytes: Vec<Vec<Vec<u8>>>,
        pub class_values_bytes: Vec<Vec<u8>>,
        pub attr_values_big_integer: Vec<Vec<Vec<BigUint>>>,
        pub class_values_big_integer: Vec<Vec<BigUint>>,
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
    }

    pub struct DecisionTreeShares {
        pub additive_triples: Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)>,
        pub additive_bigint_triples: Vec<(BigUint, BigUint, BigUint)>,
        pub binary_triples: Vec<(u8, u8, u8)>,
        pub equality_shares: Vec<(BigUint)>,
        pub current_additive_index: Arc<Mutex<usize>>,
        pub current_additive_bigint_index: Arc<Mutex<usize>>,
        pub current_equality_index: Arc<Mutex<usize>>,
        pub current_binary_index: Arc<Mutex<usize>>,
        pub sequential_additive_index: usize,
        pub sequential_additive_bigint_index: usize,
        pub sequential_equality_index: usize,
        pub sequential_binary_index: usize,
    }


    #[derive(Serialize, Deserialize, Debug)]
    pub struct DecisionTreeTIShareMessage {
        pub additive_triples: String,
        pub additive_bigint_triples: String,
        pub binary_triples: String,
        pub equality_shares: String,
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
                attr_values_bytes: self.attr_values_bytes.clone(),
                class_values_bytes: self.class_values_bytes.clone(),
                attr_values_big_integer: self.attr_values_big_integer.clone(),
                class_values_big_integer: self.class_values_big_integer.clone(),
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
            }
        }
    }


    impl Clone for DecisionTreeShares {
        fn clone(&self) -> Self {
            let mut additive_triples = Vec::new();
            let mut additive_bigint_triples = Vec::new();
            let mut binary_triples = Vec::new();
            let mut equality_shares = Vec::new();

            for item in self.additive_triples.iter() {
                additive_triples.push((item.0.clone(), item.1.clone(), item.2.clone()));
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
                additive_triples,
                additive_bigint_triples,
                binary_triples,
                equality_shares,
                current_additive_index: Arc::clone(&self.current_additive_index),
                current_additive_bigint_index: Arc::clone(&self.current_additive_bigint_index),
                current_equality_index: Arc::clone(&self.current_equality_index),
                current_binary_index: Arc::clone(&self.current_binary_index),
                sequential_additive_index: self.sequential_additive_index,
                sequential_additive_bigint_index: self.sequential_additive_bigint_index,
                sequential_equality_index: self.sequential_equality_index,
                sequential_binary_index: self.sequential_binary_index,
            }
        }
    }


    pub fn train(ctx: &mut ComputingParty, r: usize) -> DecisionTreeResult {
        println!("start building model");
        ctx.thread_hierarchy.push(format!("DT_level_{}", r));
        let major_class_index = find_common_class_index(ctx);
        // Make majority class index one-hot encoding public
        // Share major class index

        let mut major_class_index_receive: Vec<u8> = Vec::new();

        if ctx.raw_tcp_communication {
            let mut o_stream = ctx.o_stream.try_clone()
                .expect("failed cloning tcp o_stream");
            let mut in_stream = ctx.in_stream.try_clone().expect("failed cloning tcp o_stream");
            let mut reader = BufReader::new(in_stream);
            let mut share_message = String::new();

            if ctx.asymmetric_bit == 1 {
                o_stream.write(format!("{}\n", serde_json::to_string(&major_class_index).unwrap()).as_bytes());
                reader.read_line(&mut share_message).expect("fail to read share message str");
                major_class_index_receive = serde_json::from_str(&share_message).unwrap();
            } else {
                reader.read_line(&mut share_message).expect("fail to read share message str");
                major_class_index_receive = serde_json::from_str(&share_message).unwrap();
                o_stream.write(format!("{}\n", serde_json::to_string(&major_class_index).unwrap()).as_bytes());
            }

        } else {
            ctx.thread_hierarchy.push("share_major_class_index".to_string());
            let message_id = ctx.thread_hierarchy.join(":");
            println!("current message_id:{}", message_id);
            let message_content = serde_json::to_string(&major_class_index).unwrap();
            push_message_to_queue(&ctx.remote_mq_address, &message_id, &message_content);
            let message_received = receive_message_from_queue(&ctx.local_mq_address, &message_id, 1);
            let mut major_class_index_receive: Vec<u8> = Vec::new();
            major_class_index_receive = serde_json::from_str(&message_received[0]).unwrap();
            ctx.thread_hierarchy.pop();
        }

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
//        ctx.thread_hierarchy.pop();

        // quit if reaching max depth
        if r == 0 {
            println!("Exited on base case: Recursion Level == 0");
            ctx.dt_results.result_list.push(format!("class={}", major_index));
//            ctx.thread_hierarchy.pop();
            return ctx.dt_results.clone();
        }

        let mut major_class_index_decimal = if ctx.asymmetric_bit == 1 {
            let mut result = change_binary_to_bigint_field(&major_class_index_shared, ctx);
            result
        } else {
            let mut result = change_binary_to_bigint_field(&vec![0u8; ctx.dt_data.class_value_count], ctx);
            result
        };
        let mut subset_transaction = ctx.dt_training.subset_transaction_bit_vector.clone();
        ctx.thread_hierarchy.push("change_subset_trans".to_string());
        let mut transactions_decimal = change_binary_to_bigint_field(&subset_transaction, ctx);
        ctx.thread_hierarchy.pop();

        let class_value_count = ctx.dt_data.class_value_count;

        let mut major_class_trans_count = BigUint::zero();
        let mut bigint_prime = big_uint_clone(&ctx.dt_training.big_int_prime);
        let dataset_size = ctx.dt_data.instance_count;
        let thread_pool = ThreadPool::new(ctx.thread_count);

        if ctx.raw_tcp_communication{
            for i in 0..class_value_count{
                let major_class_index_value = big_uint_clone(&major_class_index_decimal[i]);
                let mut dp_result = dot_product_bigint(&transactions_decimal, &vec![major_class_index_value; dataset_size], ctx);
                major_class_trans_count = major_class_trans_count.add(&dp_result).mod_floor(&bigint_prime);
            }
        }else{
            let mut dp_result_map = Arc::new(Mutex::new(HashMap::new()));
            ctx.thread_hierarchy.push("major_class_trans_count".to_string());
            for i in 0..class_value_count {
                let mut dp_result = Arc::clone(&dp_result_map);
                let mut transactions_decimal_cp = transactions_decimal.clone();
                let mut dt_ctx = ctx.clone();
                let major_class_index_value = big_uint_clone(&major_class_index_decimal[i]);
                dt_ctx.thread_hierarchy.push(format!("{}", i));
                thread_pool.execute(move || {
                    let mut result = dot_product_bigint(&transactions_decimal_cp, &vec![major_class_index_value; dataset_size], &mut dt_ctx);
                    let mut dp_result = dp_result.lock().unwrap();
                    (*dp_result).insert(i, result);
                });
            }

            thread_pool.join();
            ctx.thread_hierarchy.pop();

            let mut dp_result_map = dp_result_map.lock().unwrap();
            for i in 0..class_value_count {
                let mut dp_result = (*dp_result_map).get(&i).unwrap();
                major_class_trans_count = major_class_trans_count.add(dp_result).mod_floor(&bigint_prime);
            }
        }


        println!("Majority Class Transaction Count: {}", major_class_trans_count.to_string());

        let mut transaction_count = BigUint::zero();
        let list = if ctx.asymmetric_bit == 1 {
            vec![BigUint::one(); dataset_size]
        } else {
            vec![BigUint::zero(); dataset_size]
        };
        let transaction_count = dot_product_bigint(&transactions_decimal, &list, ctx);
        println!("Transactions in current subset: {}", transaction_count.to_string());

        let eq_test_result = equality_big_integer(&transaction_count, &major_class_trans_count, ctx);
        let eq_test_revealed = reveal_bigint_result(&eq_test_result, ctx);
        println!("MajClassTrans = SubsetTrans? (Non-zero -> not equal):{}", eq_test_result.to_string());

        ctx.thread_hierarchy.push("early_stop_criteria".to_string());
        let mut compute_result = BigUint::one();
        let stopping_bit = multiplication_bigint(&eq_test_result, &compute_result, ctx);

        let mut stopping_bit_received = BigUint::zero();
        if ctx.raw_tcp_communication{
            let mut o_stream = ctx.o_stream.try_clone()
                .expect("failed cloning tcp o_stream");
            let mut in_stream = ctx.in_stream.try_clone().expect("failed cloning tcp o_stream");
            let mut reader = BufReader::new(in_stream);
            let mut share_message = String::new();

            if ctx.asymmetric_bit == 1 {
                o_stream.write((serialize_biguint(&stopping_bit)+"\n").as_bytes());
                reader.read_line(&mut share_message).expect("fail to read share message str");
                stopping_bit_received = deserialize_biguint(&share_message);
            } else {
                reader.read_line(&mut share_message).expect("fail to read share message str");
                stopping_bit_received = deserialize_biguint(&share_message);
                o_stream.write((serialize_biguint(&stopping_bit)+"\n").as_bytes());
            }
        }else{
            let message_id = ctx.thread_hierarchy.join(":");
            let message_content = stopping_bit.to_string();
            push_message_to_queue(&ctx.remote_mq_address, &message_id, &message_content);
            let message_received = receive_message_from_queue(&ctx.local_mq_address, &message_id, 1);
            stopping_bit_received = BigUint::from_str(&message_received[0]).unwrap();
        }


        println!("Stopping bit received:{}", stopping_bit_received.to_string());
        let stopping_check = stopping_bit.add(&stopping_bit_received).mod_floor(&bigint_prime);
        if stopping_check.eq(&BigUint::one()) {
            println!("Exited on base case: All transactions predict same outcome");
            ctx.dt_results.result_list.push(format!("class={}", major_index));
            ctx.thread_hierarchy.pop();
            return ctx.dt_results.clone();
        }
        ctx.thread_hierarchy.pop();

        println!("Base case not reached. Continuing.");

        let mut class_value_trans_vector = ctx.dt_data.class_values_bytes.clone();
        let mut u_list = Vec::new();
        //todo multi-thread
        for i in 0..class_value_count {
            let batch_multi_result = batch_multiplication_byte(&subset_transaction, &class_value_trans_vector[i], ctx);
            u_list.push(batch_multi_result);
        }

        //todo multi-thread
        let mut u_decimal = Vec::new();
        for i in 0..class_value_count {
            u_decimal.push(change_binary_to_bigint_field(&u_list[i], ctx));
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

        ctx.thread_hierarchy.push("main_computation".to_string());
        for k in 0..attr_count {
            if attributes[k] != 0 {
                for j in 0..attr_val_count {
                    // Determine the number of transactions that are:
                    // 1. in the current subset
                    // 2. predict the i-th class value
                    // 3. and have the j-th value of the k-th attribute
                    let mut dp_result_map = Arc::new(Mutex::new(HashMap::new()));

                    for i in 0..class_value_count {
                        if ctx.raw_tcp_communication{
                            let mut ctx_cp = ctx.clone();
                            let u_decimal_cp = big_uint_vec_clone(&u_decimal[i]);
                            let attr_value_trans_bigint = big_uint_vec_clone(&attr_value_trans_bigint_vec[k][j]);

                            let dp_result = dot_product_bigint(&u_decimal_cp, &attr_value_trans_bigint, &mut ctx_cp);
                            x[k][i][j] = big_uint_clone(&dp_result);
                            let y_temp = big_uint_clone(&y[k][j]);
                            y[k][j] = y_temp.add(&dp_result).mod_floor(&bigint_prime);
                        }else{
                            let mut ctx_cp = ctx.clone();
                            let mut dp_result_map_cp = Arc::clone(&dp_result_map);
                            ctx_cp.thread_hierarchy.push(format!("{}", i));
                            let u_decimal_cp = big_uint_vec_clone(&u_decimal[i]);
                            let attr_value_trans_bigint = big_uint_vec_clone(&attr_value_trans_bigint_vec[k][j]);

                            thread_pool.execute(move || {
                                let dp_result = dot_product_bigint(&u_decimal_cp, &attr_value_trans_bigint, &mut ctx_cp);
                                let mut dp_result_map_cp = dp_result_map_cp.lock().unwrap();
                                (*dp_result_map_cp).insert(i, dp_result);
                            });
                        }

                    }

                    if !ctx.raw_tcp_communication{
                        thread_pool.join();
                        let mut dp_result_map = dp_result_map.lock().unwrap();
                        for i in 0..class_value_count {
                            let dp_result = (*dp_result_map).get(&i).unwrap();
                            x[k][i][j] = big_uint_clone(dp_result);
                            let y_temp = big_uint_clone(&y[k][j]);
                            y[k][j] = y_temp.add(dp_result).mod_floor(&bigint_prime);
                        }
                    }


                    y[k][j] = alpha.mul(&y[k][j]).add(if ctx.asymmetric_bit == 1 {
                        BigUint::one()
                    } else {
                        BigUint::zero()
                    }).mod_floor(&bigint_prime);
                }

                if ctx.raw_tcp_communication{
                    let mut ctx_cp = ctx.clone();
                    for i in 0..class_value_count{
                        let batch_multi_result = batch_multiply_bigint(&x[k][i], &x[k][i], &mut ctx_cp);
                        x2[k][i] = batch_multi_result;
                    }

                }else{
                    ctx.thread_hierarchy.push("compute_x_square".to_string());
                    let mut batch_multi_result_map = Arc::new(Mutex::new(HashMap::new()));
                    for i in 0..class_value_count {
                        let mut batch_multi_result_map_cp = Arc::clone(&batch_multi_result_map);
                        let mut ctx_cp = ctx.clone();
                        ctx_cp.thread_hierarchy.push(format!("{}", i));
                        let x_list = big_uint_vec_clone(&x[k][i]);
                        thread_pool.execute(move || {
                            let batch_multi_result = batch_multiply_bigint(&x_list, &x_list, &mut ctx_cp);
                            let mut batch_multi_result_map_cp = batch_multi_result_map_cp.lock().unwrap();
                            (*batch_multi_result_map_cp).insert(i, batch_multi_result);
                        });
                    }
                    thread_pool.join();
                    let mut batch_multi_result_map = batch_multi_result_map.lock().unwrap();
                    for i in 0..class_value_count {
                        x2[k][i] = big_uint_vec_clone((*batch_multi_result_map).get(&i).unwrap());
                    }
                    ctx.thread_hierarchy.pop();
                }



                let mut ctx_cloned = ctx.clone();
                let p_multi = parallel_multiplication_big_integer(&y[k], &mut ctx_cloned);
                gini_denominators[k] = p_multi;

                gini_numerators[k] = BigUint::zero();
                ctx.thread_hierarchy.push("gini_numerators_computation".to_string());
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
                ctx.thread_hierarchy.pop();
            }
        }
        ctx.thread_hierarchy.pop();

        let mut k = 0;
        while attributes[k] == 0 {
            k += 1;
        }

        let mut gini_max_numerator = big_uint_clone(&gini_numerators[k]);
        let mut gini_max_denominator = big_uint_clone(&gini_denominators[k]);
        let mut gini_argmax = if ctx.asymmetric_bit == 1 { BigUint::from(k) } else { BigUint::zero() };
        k += 1;
        ctx.thread_hierarchy.push("gini_argmax_computation".to_string());
        while k < attr_count {
            if attributes[k] == 1 {
                let mut left_operand = BigUint::zero();
                let mut right_operand = BigUint::zero();
                let mut gini_argmaxes = [if ctx.asymmetric_bit == 1 { BigUint::from(k) } else { BigUint::zero() }, big_uint_clone(&gini_argmax)];
                let mut numerators = [big_uint_clone(&gini_numerators[k]), big_uint_clone(&gini_max_numerator)];
                let mut denominators = [big_uint_clone(&gini_denominators[k]), big_uint_clone(&gini_max_denominator)];
                let mut new_assignments_bin = vec![BigUint::zero(); 2];
                let mut new_assignments = vec![BigUint::zero(); 2];

                let mult0 = multiplication_bigint(&numerators[0], &denominators[1], ctx);
                let mult1 = multiplication_bigint(&numerators[1], &denominators[0], ctx);

                new_assignments_bin[0] = compare_bigint(&left_operand, &right_operand, ctx);
                new_assignments_bin[1] = compare_bigint(&new_assignments_bin[0], &(if ctx.asymmetric_bit == 1 { BigUint::one() } else { BigUint::zero() }), ctx);

                gini_argmax = dot_product_bigint(&new_assignments, &gini_argmaxes.to_vec(), ctx);
                gini_max_numerator = dot_product_bigint(&new_assignments, &numerators.to_vec(), ctx);
                gini_max_denominator = dot_product_bigint(&new_assignments, &denominators.to_vec(), ctx);
            }
            k += 1;
        }
        ctx.thread_hierarchy.pop();

        let mut argmax_received = BigUint::zero();
        if ctx.raw_tcp_communication{
            let mut o_stream = ctx.o_stream.try_clone()
                .expect("failed cloning tcp o_stream");
            let mut in_stream = ctx.in_stream.try_clone().expect("failed cloning tcp o_stream");
            let mut reader = BufReader::new(in_stream);
            let mut share_message = String::new();

            if ctx.asymmetric_bit == 1 {
                o_stream.write((serialize_biguint(&gini_argmax)+"\n").as_bytes());
                reader.read_line(&mut share_message).expect("fail to read share message str");
                argmax_received = deserialize_biguint(&share_message);
            } else {
                reader.read_line(&mut share_message).expect("fail to read share message str");
                argmax_received = deserialize_biguint(&share_message);
                o_stream.write((serialize_biguint(&gini_argmax)+"\n").as_bytes());
            }
        }else{
            ctx.thread_hierarchy.push("gini_argmax_public".to_string());
            let message_id = ctx.thread_hierarchy.join(":");
            let message_content = serialize_biguint(&gini_argmax);
            push_message_to_queue(&ctx.remote_mq_address, &message_id, &message_content);
            let message_received = receive_message_from_queue(&ctx.local_mq_address, &message_id, 1);
            argmax_received = deserialize_biguint(&message_received[0]);
            ctx.thread_hierarchy.pop();
        }

        let mut shared_gini_argmax = gini_argmax.add(&argmax_received).mod_floor(&bigint_prime).to_usize().unwrap();
        attributes[shared_gini_argmax] = 0;
        ctx.dt_results.result_list.push(format!("attr={}", shared_gini_argmax));


        if ctx.raw_tcp_communication{
            let mut dt_ctx = ctx.clone();
            let mut result_map = HashMap::new();
            for j in 0..attr_val_count {
                let attr_values_bytes = &ctx.dt_data.attr_values_bytes;
                let batch_multi = batch_multiplication_byte(&subset_transaction, &attr_values_bytes[shared_gini_argmax][j], &mut dt_ctx);
                result_map.insert(j,batch_multi);
            }

            for j in 0..attr_val_count {
                ctx.dt_training.subset_transaction_bit_vector = result_map.get(&j).unwrap().clone();
                train(ctx, r - 1);
            }
        }else{
            ctx.thread_hierarchy.push("update_transactions".to_string());
            let mut batch_multi_result_map = Arc::new(Mutex::new(HashMap::new()));
            for j in 0..attr_val_count {
                let mut dt_ctx = ctx.clone();
                let mut batch_multi_result_cp = Arc::clone(&batch_multi_result_map);
                let mut subset_transaction_cp = subset_transaction.clone();
                thread_pool.execute(move || {
                    let attr_values_bytes = &dt_ctx.dt_data.attr_values_bytes;
                    let temp_value = attr_values_bytes[shared_gini_argmax][j].clone();
                    let batch_multi = batch_multiplication_byte(&subset_transaction_cp, &temp_value, &mut dt_ctx);
                    let mut batch_multi_result_cp = batch_multi_result_cp.lock().unwrap();
                    (*batch_multi_result_cp).insert(j, batch_multi);
                });
            }
            thread_pool.join();
            ctx.thread_hierarchy.pop();
            let mut batch_multi_result_map = batch_multi_result_map.lock().unwrap();
            for j in 0..attr_val_count {
                ctx.dt_training.subset_transaction_bit_vector = (*batch_multi_result_map).get(&j).unwrap().clone();
                train(ctx, r - 1);
            }

            ctx.thread_hierarchy.pop();
        }

        ctx.dt_results.clone()
    }

    fn find_common_class_index(ctx: &mut ComputingParty) -> Vec<u8> {
        let mut now = SystemTime::now();
        ctx.thread_hierarchy.push("find_common_class_index".to_string());
        let mut subset_transaction_bit_vector = ctx.dt_training.subset_transaction_bit_vector.clone();
        let mut subset_decimal = change_binary_to_decimal_field(&subset_transaction_bit_vector, ctx);
        let mut s = Vec::new();
        let mut bit_shares = Vec::new();
        let mut argmax_result = Vec::new();
        if ctx.raw_tcp_communication {
            let mut class_value_transaction = ctx.dt_data.class_values.clone();
            for i in 0..ctx.dt_data.class_value_count {
                let dp_result = dot_product_integer(&subset_decimal, &class_value_transaction[i], ctx);
                s.push(dp_result.0);
            }
            for i in 0..ctx.dt_data.class_value_count {
                let s_copied = s[i];
                let bd_result = bit_decomposition(s[i], ctx);
                bit_shares.push(bd_result);
                argmax_result = arg_max(&bit_shares, ctx);
            }
        } else {
            let thread_pool = ThreadPool::new(ctx.thread_count);
            let mut dp_result_map = Arc::new(Mutex::new(HashMap::new()));
            let mut ctx_copied = ctx.clone();

            ctx_copied.thread_hierarchy.push("compute_dp".to_string());
            for i in 0..ctx.dt_data.class_value_count {
                let mut dp_result_map = Arc::clone(&dp_result_map);
                let mut subset_decimal_cloned = subset_decimal.clone();
                let mut class_value_transaction = ctx.dt_data.class_values.clone();
                let mut ctx = ctx_copied.clone();
                ctx.thread_hierarchy.push(format!("{}", i));
                thread_pool.execute(move || {
//                    let precision = ctx.decimal_precision;
                    let dp_result = dot_product_integer(&subset_decimal_cloned, &class_value_transaction[i], &mut ctx);
                    let mut dp_result_map = dp_result_map.lock().unwrap();
                    (*dp_result_map).insert(i, (dp_result.0));
                });
            }
            thread_pool.join();
            println!("compute_dp completes in {}ms", now.elapsed().unwrap().as_millis());
            ctx_copied.thread_hierarchy.pop();

            let mut dp_result_map = &*(dp_result_map.lock().unwrap());
            for i in 0..ctx.dt_data.class_value_count {
                s.push((dp_result_map.get(&i).unwrap()).clone());
            }

            let mut ctx_copied = ctx.clone();
            let mut bd_result_map = Arc::new(Mutex::new(HashMap::new()));

            ctx_copied.thread_hierarchy.push("compute_bd".to_string());
            for i in 0..ctx.dt_data.class_value_count {
                let mut bd_result_map = Arc::clone(&bd_result_map);
                let mut ctx = ctx_copied.clone();
                ctx.thread_hierarchy.push(format!("{}", i));
                let s_copied = s[i];
                thread_pool.execute(move || {
                    let bd_result = bit_decomposition(s_copied, &mut ctx);
                    let mut bd_result_map = bd_result_map.lock().unwrap();
                    (*bd_result_map).insert(i, bd_result);
                });
            }
            thread_pool.join();
            println!("compute_bd completes in {}ms", now.elapsed().unwrap().as_millis());
            ctx_copied.thread_hierarchy.pop();

            let mut bd_result_map = &*(bd_result_map.lock().unwrap());
            for i in 0..ctx.dt_data.class_value_count {
                bit_shares.push((bd_result_map.get(&i).unwrap()).clone());
            }

            argmax_result = arg_max(&bit_shares, ctx);
            ctx.thread_hierarchy.pop();
        }

        println!("find common class index completes in {}ms", now.elapsed().unwrap().as_millis());
        argmax_result
    }
}
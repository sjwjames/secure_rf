pub mod decision_tree {
    use crate::computing_party::computing_party::ComputingParty;
    use num::bigint::{BigInt, BigUint};
    use num::integer::*;
    use std::io::{Bytes, Write, BufReader, BufRead};
    use serde::{Serialize, Deserialize, Serializer};
    use std::num::Wrapping;
    use crate::utils::utils::{big_uint_clone, push_message_to_queue, receive_message_from_queue, big_uint_vec_clone};
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
    use crate::multiplication::multiplication::{batch_multiply_bigint, multiplication_bigint, batch_multiplication_byte};
    use num::{Zero, ToPrimitive, One};
    use std::ops::{Add, Mul};

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
        pub alpha: BigInt,
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
            }
        }
    }


    pub fn train(ctx: &mut ComputingParty, r: usize) {
        println!("start building model");
        ctx.thread_hierarchy.push("DT".to_string());
        let major_class_index = find_common_class_index(ctx);
        // Make majority class index one-hot encoding public
        // Share major class index
        ctx.thread_hierarchy.push("share_major_class_index".to_string());


        let message_id = ctx.thread_hierarchy.join(":");
        let message_content = serde_json::to_string(&major_class_index).unwrap();
        push_message_to_queue(&ctx.local_mq_address, &message_id, &message_content);
        let message_received = receive_message_from_queue(&ctx.remote_mq_address, &message_id, 1);
        let mut major_class_index_receive: Vec<u8> = Vec::new();
        major_class_index_receive = serde_json::from_str(&message_received[0]).unwrap();

//        let mut o_stream = ctx.o_stream.try_clone()
//            .expect("failed cloning tcp o_stream");
//        let mut message = RFMessage {
//            message_id: ctx.thread_hierarchy.join(":"),
//            message_content: serde_json::to_string(&major_class_index).unwrap(),
//        };
//        let mut major_class_index_receive: Vec<u8> = Vec::new();
//        if ctx.asymmetric_bit == 1 {
//            o_stream.write(format!("{}\n", serde_json::to_string(&message).unwrap()).as_bytes());
//            let mut received_message = search_pop_message(ctx, message.message_id.clone()).unwrap();
//            major_class_index_receive = serde_json::from_str(&received_message.message_content).unwrap();
//        } else {
//            let mut received_message = search_pop_message(ctx, message.message_id.clone()).unwrap();
//            major_class_index_receive = serde_json::from_str(&received_message.message_content).unwrap();
//            o_stream.write(format!("{}\n", serde_json::to_string(&message).unwrap()).as_bytes());
//        }
//        o_stream.write(format!("{}\n", serde_json::to_string(&message).unwrap()).as_bytes());
//        let mut received_message = search_pop_message(ctx, message.message_id.clone()).unwrap();
//        major_class_index_receive = serde_json::from_str(&received_message.message_content).unwrap();

        let class_value_count = ctx.dt_data.class_value_count;
        let mut major_class_index_shared = vec![0u8; class_value_count];
        for i in 0..class_value_count {
            major_class_index_shared[i] = mod_floor((Wrapping((&major_class_index)[i]) + Wrapping((&major_class_index_receive)[i])).0, BINARY_PRIME as u8);
        }
        ctx.thread_hierarchy.pop();

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
        ctx.thread_hierarchy.pop();

        // quit if reaching max depth
        if r == 0 {
            println!("Exited on base case: Recursion Level == 0");
            ctx.dt_results.result_list.push(format!("class={}", major_index));
            return;
        }

        let mut major_class_index_decimal = if ctx.asymmetric_bit == 1 {
            let mut result = change_binary_to_bigint_field(&major_class_index_shared, ctx);
            result
        } else {
            let mut result = change_binary_to_bigint_field(&vec![0u8; ctx.dt_data.class_value_count], ctx);
            result
        };
        let mut subset_transaction = ctx.dt_training.subset_transaction_bit_vector.clone();
        let mut transactions_decimal = change_binary_to_bigint_field(&subset_transaction, ctx);


        let class_value_count = ctx.dt_data.class_value_count;
        let thread_pool = ThreadPool::new(ctx.thread_count);
        let mut dp_result_map = Arc::new(Mutex::new(HashMap::new()));
        let dataset_size = ctx.dt_data.instance_count;
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
        let mut bigint_prime = big_uint_clone(&ctx.dt_training.big_int_prime);
        let mut major_class_trans_count = BigUint::zero();
        let mut dp_result_map = dp_result_map.lock().unwrap();
        for i in 0..class_value_count {
            let mut dp_result = (*dp_result_map).get(&i).unwrap();
            major_class_trans_count = major_class_trans_count.add(big_uint_clone(dp_result)).mod_floor(&bigint_prime);
        }

        println!("Majority Class Transaction Count: {}", major_class_trans_count.to_u64().unwrap());

        let mut transaction_count = BigUint::zero();
        let list = if ctx.asymmetric_bit == 1 {
            vec![BigUint::one(); dataset_size]
        } else {
            vec![BigUint::zero(); dataset_size]
        };
        let transaction_count = dot_product_bigint(&transactions_decimal, &list, ctx);
        println!("Transactions in current subset: {}", transaction_count.to_u64().unwrap());

        let eq_test_result = equality_big_integer(&transaction_count, &major_class_trans_count, ctx);
        println!("MajClassTrans = SubsetTrans? (Non-zero -> not equal):{}", eq_test_result.to_u64().unwrap());

        let mut compute_result = BigUint::one();
        let stopping_bit = multiplication_bigint(&eq_test_result, &compute_result, ctx);

        ctx.thread_hierarchy.push("stop_criteria".to_string());
        let message_id = ctx.thread_hierarchy.join(":");
        let message_content = serde_json::to_string(&(stopping_bit.to_bytes_le())).unwrap();
        push_message_to_queue(&ctx.local_mq_address, &message_id, &message_content);
        let message_received = receive_message_from_queue(&ctx.remote_mq_address, &message_id, 1);
        ctx.thread_hierarchy.pop();
        let stopping_bit_received: Vec<u8> = serde_json::from_str(&message_received[0]).unwrap();
        let stopping_bit_received = BigUint::from_bytes_le(&stopping_bit_received);

        let stopping_check = stopping_bit.add(&stopping_bit_received).mod_floor(&bigint_prime);
        if stopping_check.eq(&BigUint::zero()) {
            println!("Exited on base case: All transactions predict same outcome");
            ctx.dt_results.result_list.push(format!("class={}", major_index));
            return;
        }

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
        let mut x: Vec<Vec<Vec<BigUint>>> = vec![vec![vec![BigUint::zero();attr_val_count];class_value_count];attr_count];
        let mut x2: Vec<Vec<Vec<BigUint>>> = vec![vec![vec![BigUint::zero();attr_val_count];class_value_count];attr_count];
        let mut y: Vec<Vec<BigUint>> = vec![vec![BigUint::zero();attr_val_count];attr_count];
        let mut gini_numerators: Vec<BigUint> = vec![BigUint::zero();attr_count];
        let mut gini_denominators: Vec<BigUint> = vec![BigUint::zero();attr_count];
        let attributes = &ctx.dt_training.attribute_bit_vector;
        let attr_value_trans_bigint_vec = &ctx.dt_data.attr_values_big_integer;

        ctx.thread_hierarchy.push("main_computation".to_string());
        for k in 0..attr_count {
            if attributes[k] != 0 {
                for j in 0..attr_val_count{
                    // Determine the number of transactions that are:
                    // 1. in the current subset
                    // 2. predict the i-th class value
                    // 3. and have the j-th value of the k-th attribute
                    let attr_value_trans_bigint = big_uint_vec_clone(&attr_value_trans_bigint_vec[k][j]);
                    let mut dp_result_map = Arc::new(Mutex::new(HashMap::new()));

                    for i in 0..class_value_count{
                        let mut ctx_cp = ctx.clone();
                        let mut dp_result_map_cp = Arc::clone(&dp_result_map);
                        ctx_cp.thread_hierarchy.push(format!("{}",i));
                        thread_pool.execute(move||{
                            let dp_result = dot_product_bigint(&u_decimal[i],&attr_value_trans_bigint,&mut ctx_cp);
                            let mut dp_result_map_cp = dp_result_map_cp.lock().unwrap();
                            (*dp_result_map_cp).insert(i,dp_result);
                        });
                    }

                    thread_pool.join();
                    let mut dp_result_map = dp_result_map.lock().unwrap();
                    for i in 0..class_value_count{
                        let dp_result = (*dp_result_map).get(&i).unwrap();
                        x[k][i][j] = big_uint_clone(dp_result);
                        y[k][j] = (y[k][j]).add(dp_result).mod_floor(&bigint_prime);
                    }
                    y[k][j] = ctx.dt_training.alpha.mul(&y[k][j]).add(if ctx.asymmetric_bit==1{
                        BigUint::one()
                    }else{
                        BigUint::zero()
                    }).mod_floor(bigint_prime);
                }

                ctx.thread_hierarchy.push("compute_x_square".to_string());
                let mut batch_multi_result_map = Arc::new(Mutex::new(HashMap::new()));
                for i in 0..class_value_count{
                    let mut batch_multi_result_map_cp = Arc::clone(&batch_multi_result_map);
                    let mut ctx_cp = ctx.clone();
                    ctx_cp.thread_hierarchy.push(format!("{}",i));
                    thread_pool.execute(move||{
                        let batch_multi_result=batch_multiply_bigint(&x[k][i],&x[k][i],&mut ctx_cp);
                        let mut batch_multi_result_map_cp = batch_multi_result_map_cp.lock().unwrap();
                        (*batch_multi_result_map_cp).insert(i,batch_multi_result);
                    });
                }
                thread_pool.join();
                let mut batch_multi_result_map = batch_multi_result_map.lock().unwrap();
                for i in 0..class_value_count{
                    x2[k][i] = big_uint_vec_clone((*batch_multi_result_map).get(&i).unwrap());
                }
                ctx.thread_hierarchy.pop();

            }
        }
        ctx.thread_hierarchy.pop();

        ctx.thread_hierarchy.pop();
    }

    fn find_common_class_index(ctx: &mut ComputingParty) -> Vec<u8> {
        let mut now = SystemTime::now();
        ctx.thread_hierarchy.push("find_common_class_index".to_string());
        let mut subset_transaction_bit_vector = ctx.dt_training.subset_transaction_bit_vector.clone();
        let mut subset_decimal = change_binary_to_decimal_field(&subset_transaction_bit_vector, ctx);
        let mut s = Vec::new();
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
                let precision = ctx.decimal_precision;
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
        let mut bit_shares = Vec::new();
        for i in 0..ctx.dt_data.class_value_count {
            bit_shares.push((bd_result_map.get(&i).unwrap()).clone());
        }

        let mut arg_max = arg_max(&bit_shares, ctx);

        ctx.thread_hierarchy.pop();
        println!("find common class index completes in {}ms", now.elapsed().unwrap().as_millis());
        arg_max
    }
}
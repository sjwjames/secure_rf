pub mod decision_tree {
    use crate::computing_party::computing_party::ComputingParty;
    use num::bigint::{BigInt, BigUint};
    use num::integer::*;
    use std::io::{Bytes, Write, BufReader, BufRead};
    use serde::{Serialize, Deserialize, Serializer};
    use std::num::Wrapping;
    use crate::utils::utils::big_uint_clone;
    use threadpool::ThreadPool;
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;
    //    use crate::dot_product::dot_product::dot_product;
    use crate::field_change::field_change::change_binary_to_decimal_field;
    use std::thread::sleep;
    use std::time::Duration;
    use crate::dot_product::dot_product::dot_product;
    use crate::bit_decomposition::bit_decomposition::bit_decomposition;
    use crate::protocol::protocol::arg_max;
    use crate::constants::constants::BINARY_PRIME;

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


    pub fn train(ctx: &mut ComputingParty) {
        let major_class_index = find_common_class_index(ctx);
        // Make majority class index one-hot encoding public
        // Share major class index
        let mut in_stream = ctx.in_stream.try_clone()
            .expect("failed cloning tcp o_stream");

        let mut o_stream = ctx.o_stream.try_clone()
            .expect("failed cloning tcp o_stream");
        o_stream.write(format!("{}\n", serde_json::to_string(&major_class_index).unwrap()).as_bytes());

        let mut reader = BufReader::new(in_stream);
        let mut major_class_index_message = String::new();
        reader.read_line(&mut major_class_index_message).expect("fail to read major class index message");
        let mut major_class_index_receive:Vec<u8> = serde_json::from_str(&major_class_index_message).unwrap();
        let class_value_count = ctx.dt_data.class_value_count;
        let mut major_class_index_shared = vec![0u8;class_value_count];
        for i in 0..class_value_count{
            major_class_index_shared[i] = mod_floor((Wrapping((&major_class_index)[i])+Wrapping((&major_class_index_receive)[i])).0,BINARY_PRIME as u8);
        }

        let mut major_index = 0;
        for i in 0..class_value_count{
            if major_class_index_shared[i] == 1{
                major_index = i;
                break;
            }
        }

        for i in major_index+1..class_value_count{
            major_class_index_shared[i] = 0;
        }


    }

    fn find_common_class_index(ctx: &mut ComputingParty) -> Vec<u8> {
        let mut subset_transaction_bit_vector = ctx.dt_training.subset_transaction_bit_vector.clone();
        let mut subset_decimal = change_binary_to_decimal_field(&subset_transaction_bit_vector, ctx);
        let mut s = Vec::new();
        let thread_pool = ThreadPool::new(ctx.thread_count);
        let mut dp_result_map = Arc::new(Mutex::new(HashMap::new()));
        let mut ctx_copied = ctx.clone();

        for i in 0..ctx.dt_data.class_value_count {
            let mut dp_result_map = Arc::clone(&dp_result_map);
            let mut subset_decimal = subset_decimal.clone();
            let mut class_value_transaction = ctx.dt_data.class_values.clone();
            let mut ctx = ctx_copied.clone();
            thread_pool.execute(move || {
                let precision = ctx.decimal_precision;
                let dp_result = dot_product(&subset_decimal, &class_value_transaction[i], &mut ctx, precision, true, false);
                let mut dp_result_map = dp_result_map.lock().unwrap();
                (*dp_result_map).insert(i, (dp_result.0));
            });
        }
        thread_pool.join();
        let mut dp_result_map = &*(dp_result_map.lock().unwrap());
        for i in 0..ctx.dt_data.class_value_count {
            s.push((dp_result_map.get(&i).unwrap()).clone());
        }

        let mut ctx_copied = ctx.clone();
        let mut bd_result_map = Arc::new(Mutex::new(HashMap::new()));
        for i in 0..ctx.dt_data.class_value_count {
            let mut bd_result_map = Arc::clone(&bd_result_map);
            let mut ctx = ctx_copied.clone();
            let s_copied = s[i];
            //run in parallel would cause data corruption
            thread_pool.execute(move || {
                let bd_result = bit_decomposition(s_copied, &mut ctx);
                let mut bd_result_map = bd_result_map.lock().unwrap();
                (*bd_result_map).insert(i, bd_result);
            });
        }
        thread_pool.join();
        let mut bd_result_map = &*(bd_result_map.lock().unwrap());
        let mut bit_shares = Vec::new();
        for i in 0..ctx.dt_data.class_value_count {
            bit_shares.push((bd_result_map.get(&i).unwrap()).clone());
        }

        let mut arg_max = arg_max(&bit_shares, ctx);

        arg_max
    }
}
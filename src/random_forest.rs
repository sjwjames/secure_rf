pub mod random_forest {
    use crate::computing_party::computing_party::{ComputingParty, get_formatted_address, try_setup_socket, initialize_party_context, ti_receive, produce_dt_data, load_dt_raw_data, receive_preprocessing_shares};
    use crate::decision_tree::decision_tree;
    use std::sync::{Arc, Mutex};
    use threadpool::ThreadPool;
    use crate::field_change::field_change::{change_binary_to_bigint_field, change_binary_to_decimal_field};
    use std::thread::current;
    use crate::message::message::MessageManager;
    use std::collections::HashMap;
    use std::num::Wrapping;
    use crate::protocol::protocol::{matrix_multiplication_integer, batch_equality_integer};
    use crate::utils::utils::get_additive_shares;
    use crate::bit_decomposition::bit_decomposition::{bit_decomposition, batch_bit_decomposition};
    use crate::comparison::comparison::{comparison, batch_comparison};
    use crate::constants::constants::BINARY_PRIME;
    use std::fs::File;
    use std::io::{BufReader, BufRead};
    use crate::or_xor::or_xor::or_xor;

    pub fn random_feature_selection(attr_values: &Vec<Vec<u8>>, ctx: &mut ComputingParty) -> Vec<Vec<u8>> {
        let mut transformed_attr_values = Vec::new();
        for row in attr_values {
            let mut row_new = Vec::new();
            for item in row {
                row_new.push(Wrapping(*item as u64));
            }
            transformed_attr_values.push(row_new);
        }
        let rfs_shares = &ctx.dt_shares.rfs_shares.clone();
        let matrix_mul_shares = &ctx.dt_shares.matrix_mul_shares;
        let mut result = matrix_multiplication_integer(&rfs_shares, &transformed_attr_values, ctx, BINARY_PRIME as u64, matrix_mul_shares);
        let attribute_cnt = ctx.dt_shares.rfs_shares.len() / ctx.dt_data.attr_value_count;
        ctx.dt_training.attribute_bit_vector = vec![1u8; attribute_cnt];
        let mut result_transformed = Vec::new();
        for row in result {
            let mut row_new = Vec::new();
            for item in row {
                row_new.push(item.0 as u8);
            }
            result_transformed.push(row_new);
        }
        result_transformed
    }

    pub fn sample_with_replacement(ctx: &mut ComputingParty, attr_value_vec: &Vec<Vec<u8>>) {
        println!("bagging shares {:?}", ctx.dt_shares.bagging_shares);
        let bagging_shares = ctx.dt_shares.bagging_shares.clone();
        let mut temp = attr_value_vec.clone();
        let mut class_values = ctx.dt_data.class_values_bytes.clone();
        temp.append(&mut class_values);
        let mut x_list = Vec::new();
        for row in temp {
            let mut temp_row = Vec::new();
            for item in row {
                temp_row.push(Wrapping(item as u64));
            }
            x_list.push(temp_row);
        }
        let mut result = matrix_multiplication_integer(&x_list, &bagging_shares, ctx, BINARY_PRIME as u64, &ctx.dt_shares.bagging_matrix_mul_shares);
        let mut result_u8 = Vec::new();
        for row in result {
            let mut temp_row = Vec::new();
            for item in row {
                temp_row.push(item.0 as u8);
            }
            result_u8.push(temp_row);
        }
        let class_value_count = ctx.dt_data.class_value_count;
        let instance_cnt = ctx.dt_shares.bagging_shares[0].len();
        let attribute_cnt = ctx.dt_shares.rfs_shares.len() / ctx.dt_data.attr_value_count;
        ctx.dt_data = produce_dt_data(result_u8, ctx.dt_data.class_value_count, ctx.dt_data.attr_value_count, attribute_cnt, instance_cnt, ctx.asymmetric_bit);

        ctx.dt_training.subset_transaction_bit_vector = vec![ctx.asymmetric_bit as u8; instance_cnt];
        ctx.dt_training.cutoff_transaction_set_size = (ctx.dt_training.epsilon * instance_cnt as f64) as usize;
    }

    pub fn discretize_data(x: Vec<Vec<Wrapping<u64>>>, ctx: &mut ComputingParty) {
        //temporarily
        ctx.dt_data.discretized_x = x;
//        if ctx.asymmetric_bit == 1 {
//            ctx.dt_data.discretized_x = {
//                [
//                    [Wrapping(1 as u64), Wrapping(0 as u64), Wrapping(0 as u64)].to_vec(),
//                    [Wrapping(0 as u64), Wrapping(0 as u64), Wrapping(1 as u64)].to_vec(),
//                    [Wrapping(1 as u64), Wrapping(0 as u64), Wrapping(0 as u64)].to_vec(),
//                    [Wrapping(0 as u64), Wrapping(0 as u64), Wrapping(0 as u64)].to_vec()
//                ].to_vec()
//            };
//        } else {
//            ctx.dt_data.discretized_x = {
//                [
//                    [Wrapping(1 as u64), Wrapping(0 as u64), Wrapping(1 as u64)].to_vec(),
//                    [Wrapping(1 as u64), Wrapping(1 as u64), Wrapping(1 as u64)].to_vec(),
//                    [Wrapping(0 as u64), Wrapping(0 as u64), Wrapping(1 as u64)].to_vec(),
//                    [Wrapping(0 as u64), Wrapping(1 as u64), Wrapping(0 as u64)].to_vec()
//                ].to_vec()
//            };
//        }
    }

    pub fn ohe_conversion(x: &Vec<Vec<Wrapping<u64>>>, ctx: &mut ComputingParty, category: usize, prime: u64) -> Vec<Vec<u8>> {
        let rows = x.len();
        let cols = x[0].len();
        let mut equality_x = Vec::new();
        let mut equality_y = Vec::new();
        for j in 0..cols {
            for k in 0..category {
                let mut x_row = Vec::new();
                let mut y_row = Vec::new();

                for i in 0..rows {
                    x_row.push(x[i][j]);
                    if ctx.asymmetric_bit == 1 {
                        y_row.push(Wrapping(k as u64));
                    } else {
                        y_row.push(Wrapping(0));
                    }
                }
                equality_x.append(&mut x_row);
                equality_y.append(&mut y_row);
            }
        }
        let result = batch_equality_integer(&equality_x, &equality_y, ctx, prime);
        let bit_length = (prime as f64).log2().ceil() as usize;

        let mut bits_list = batch_bit_decomposition(&result,ctx,bit_length);
        let mut compared = vec![vec![0u8; bit_length];bits_list.len()];
        let comparison_results = batch_comparison(&mut compared,&mut bits_list,ctx,bit_length);

        let mut count = 0;
        let mut result = Vec::new();
        for j in 0..cols {
            for k in 0..category {
                let mut row = Vec::new();
                for i in 0..rows {
                    row.push(comparison_results[count]);
                    count += 1;
                }
                result.push(row);
            }
        }
        result
    }


    pub fn train(ctx: &mut ComputingParty) {
        ctx.thread_hierarchy.push("RF".to_string());
        let thread_pool = ThreadPool::with_name(format!("{}", "RF"), ctx.thread_count);
        let mut remainder = ctx.tree_count;
        let mut current_p0_port = ctx.party0_port + 1;
        let mut current_p1_port = ctx.party1_port + 1;
        let mut x = load_dt_raw_data(&ctx.x_input_path);
        let mut y = load_dt_raw_data(&ctx.y_input_path);

        discretize_data(x, ctx);
        receive_preprocessing_shares(ctx);

        let attr_value_count = ctx.dt_data.attr_value_count;
        let rfs_field = ctx.dt_training.rfs_field;
        let class_value_count = ctx.dt_data.class_value_count;
        let bagging_field = ctx.dt_training.bagging_field;

        let ohe_prime = if ctx.dt_training.rfs_field > ctx.dt_data.class_value_count as u64 { ctx.dt_training.rfs_field } else { ctx.dt_data.class_value_count as u64 };
        let discretized_x = ctx.dt_data.discretized_x.clone();
        let mut attr_values_bytes = ohe_conversion(&discretized_x, ctx, attr_value_count, ohe_prime);
        let mut class_values_bytes = ohe_conversion(&y, ctx, class_value_count, ohe_prime);

        for current_tree_index in 0..remainder {
            let dt_shares = ti_receive(
                ctx.ti_stream.try_clone().expect("failed to clone ti recvr"), ctx);
            let mut dt_ctx = ctx.clone();
            dt_ctx.dt_shares = dt_shares;

            let mut rfs_x = random_feature_selection(&attr_values_bytes, &mut dt_ctx);
            rfs_x.append(&mut class_values_bytes);

            sample_with_replacement(&mut dt_ctx, &rfs_x);
//
//            thread_pool.execute(move || {
//                dt_ctx.party0_port = current_p0_port;
//                dt_ctx.party1_port = current_p1_port;
//                reset_share_indices(&mut dt_ctx);
//                let (internal_addr, external_addr) = get_formatted_address(dt_ctx.party_id, &dt_ctx.party0_ip, dt_ctx.party0_port, &dt_ctx.party1_ip, dt_ctx.party1_port);
//                let (in_stream, o_stream) = try_setup_socket(&internal_addr, &external_addr, &dt_ctx.message_manager);
//                dt_ctx.in_stream = in_stream;
//                dt_ctx.o_stream = o_stream;
//
//                dt_ctx.thread_hierarchy.push(format!("{}", current_tree_index));
//                //init in java Lynx
//                let mut attr_values = Vec::new();
//                let mut class_values = Vec::new();
//                let mut attr_values_bigint = Vec::new();
//                let mut class_values_bigint = Vec::new();
//                let mut attr_values_bytes = dt_ctx.dt_data.attr_values_bytes.clone();
//                for item in attr_values_bytes.iter() {
//                    let mut attr_data_item = Vec::new();
//                    let mut attr_data_bigint_item = Vec::new();
//                    for data_item in item.iter() {
//                        attr_data_item.push(change_binary_to_decimal_field(data_item, &mut dt_ctx));
//                        attr_data_bigint_item.push(change_binary_to_bigint_field(data_item, &mut dt_ctx));
//                    }
//                    attr_values.push(attr_data_item);
//                    attr_values_bigint.push(attr_data_bigint_item);
//                }
//                dt_ctx.dt_data.attr_values = attr_values;
//                dt_ctx.dt_data.attr_values_big_integer = attr_values_bigint;
//
//                let mut class_value_bytes = dt_ctx.dt_data.class_values_bytes.clone();
//                for item in class_value_bytes.iter() {
//                    class_values.push(change_binary_to_decimal_field(item, &mut dt_ctx));
//                    class_values_bigint.push(change_binary_to_bigint_field(item, &mut dt_ctx));
//                }
//
//                dt_ctx.dt_data.class_values = class_values;
//                dt_ctx.dt_data.class_values_big_integer = class_values_bigint;
//                let depth = dt_ctx.dt_training.max_depth;
//                let dt_training = decision_tree::train(&mut dt_ctx,depth);
//                println!("{:?}",dt_training.result_list);
//            });

            dt_ctx.party0_port = current_p0_port;
            dt_ctx.party1_port = current_p1_port;

            let (internal_addr, external_addr) = get_formatted_address(dt_ctx.party_id, &dt_ctx.party0_ip, dt_ctx.party0_port, &dt_ctx.party1_ip, dt_ctx.party1_port);
            let (in_stream, o_stream) = try_setup_socket(&internal_addr, &external_addr, &dt_ctx.message_manager);
            dt_ctx.in_stream = in_stream;
            dt_ctx.o_stream = o_stream;

            dt_ctx.thread_hierarchy.push(format!("{}", current_tree_index));
            //init in java Lynx
            let mut attr_values = Vec::new();
            let mut class_values = Vec::new();
            let mut attr_values_bigint = Vec::new();
            let mut class_values_bigint = Vec::new();
            let mut attr_values_bytes = dt_ctx.dt_data.attr_values_bytes.clone();
            let dataset_size_prime = dt_ctx.dt_training.dataset_size_prime;
            // multi-thread could help
            for item in attr_values_bytes.iter() {
                let mut attr_data_item = Vec::new();
                let mut attr_data_bigint_item = Vec::new();
                for data_item in item.iter() {
                    attr_data_item.push(change_binary_to_decimal_field(data_item, &mut dt_ctx, dataset_size_prime));
                    attr_data_bigint_item.push(change_binary_to_bigint_field(data_item, &mut dt_ctx));
                }
                attr_values.push(attr_data_item);
                attr_values_bigint.push(attr_data_bigint_item);
            }
            dt_ctx.dt_data.attr_values = attr_values;
            dt_ctx.dt_data.attr_values_big_integer = attr_values_bigint;

            let mut class_value_bytes = dt_ctx.dt_data.class_values_bytes.clone();
            for item in class_value_bytes.iter() {
                class_values.push(change_binary_to_decimal_field(item, &mut dt_ctx, dataset_size_prime));
                class_values_bigint.push(change_binary_to_bigint_field(item, &mut dt_ctx));
            }

            dt_ctx.dt_data.class_values = class_values;
            dt_ctx.dt_data.class_values_big_integer = class_values_bigint;
            let max_depth = (&dt_ctx.dt_training).max_depth;
            let dt_training = decision_tree::train(&mut dt_ctx, max_depth);

            current_p0_port += 1;
            current_p1_port += 1;
//            println!("tree {} result {:?}",current_tree_index,dt_training.result_list);
        }

        thread_pool.join();
        ctx.thread_hierarchy.pop();
    }
}
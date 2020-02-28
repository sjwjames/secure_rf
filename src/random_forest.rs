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
    use std::io::{BufReader, BufRead, Write};
    use crate::or_xor::or_xor::or_xor;
    use std::time::SystemTime;

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
        let attribute_cnt = ctx.feature_selected;
        let mut result_transformed = Vec::new();
        for row in result {
            let mut row_new = Vec::new();
            for item in row {
                row_new.push(item.0 as u8);
            }
            result_transformed.push(row_new);
        }
        ctx.dt_data.attribute_count = ctx.feature_selected as usize;
        result_transformed
    }

    pub fn sample_with_replacement(ctx: &mut ComputingParty, data: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
        let bagging_shares = ctx.dt_shares.bagging_shares.clone();
        let mut x_list = Vec::new();
        for row in data {
            let mut temp_row = Vec::new();
            for item in row {
                temp_row.push(Wrapping(*item as u64));
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
        ctx.dt_data.instance_count = ctx.instance_selected as usize;
        result_u8
    }

    pub fn discretize_data(x: Vec<Vec<Wrapping<u64>>>, ctx: &mut ComputingParty) {
        //temporarily
//        ctx.dt_data.discretized_x = x;

        if ctx.asymmetric_bit == 1 {
            ctx.dt_data.discretized_x = {
                [
                    [Wrapping(1 as u64), Wrapping(0 as u64), Wrapping(0 as u64)].to_vec(),
                    [Wrapping(0 as u64), Wrapping(0 as u64), Wrapping(1 as u64)].to_vec(),
                    [Wrapping(1 as u64), Wrapping(0 as u64), Wrapping(0 as u64)].to_vec(),
                    [Wrapping(0 as u64), Wrapping(0 as u64), Wrapping(0 as u64)].to_vec()
                ].to_vec()
            };
        } else {
            ctx.dt_data.discretized_x = {
                [
                    [Wrapping(1 as u64), Wrapping(0 as u64), Wrapping(1 as u64)].to_vec(),
                    [Wrapping(1 as u64), Wrapping(1 as u64), Wrapping(1 as u64)].to_vec(),
                    [Wrapping(0 as u64), Wrapping(0 as u64), Wrapping(1 as u64)].to_vec(),
                    [Wrapping(0 as u64), Wrapping(1 as u64), Wrapping(0 as u64)].to_vec()
                ].to_vec()
            };
        }
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

        let mut bits_list = batch_bit_decomposition(&result, ctx, bit_length);
//        let mut comparison_results = Vec::new();
        //        for item in result {
//            let bits = bit_decomposition(item.0, ctx, bit_length);
//            let mut compared = vec![0; bits.len()];
//            let comparison_result = comparison(&compared, &bits, ctx);
//            comparison_results.push(comparison_result);
//        }

//        let mut comparison_results = Vec::new();
//        for mut bits in bits_list {
//            let mut compared = vec![0u8; bit_length];
//            let comparison_result = comparison(&mut compared, &mut bits, ctx);
//            comparison_results.push(comparison_result);
//        }
//        println!("comparison_results:{:?}", comparison_results);


        let mut compared = vec![vec![0u8; bit_length]; bits_list.len()];
        let comparison_results = batch_comparison(&mut compared, &mut bits_list, ctx, bit_length);

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
        let now = SystemTime::now();

        let thread_pool = ThreadPool::with_name(format!("{}", "RF"), ctx.thread_count);

        let mut remainder = ctx.tree_count;
        let mut current_p0_port = ctx.party0_port + 1;
        let mut current_p1_port = ctx.party1_port + 1;
        let mut x = load_dt_raw_data(&ctx.x_input_path);
        let mut y = load_dt_raw_data(&ctx.y_input_path);
        if ctx.asymmetric_bit == 1 {
            y = [
                [Wrapping(1)].to_vec(),
                [Wrapping(0)].to_vec(),
                [Wrapping(0)].to_vec(),
                [Wrapping(1)].to_vec()
            ].to_vec();
        } else {
            y = [
                [Wrapping(0)].to_vec(),
                [Wrapping(0)].to_vec(),
                [Wrapping(0)].to_vec(),
                [Wrapping(0)].to_vec()
            ].to_vec();
        }

        discretize_data(x, ctx);
        receive_preprocessing_shares(ctx);

        let attr_value_count = ctx.dt_data.attr_value_count;
        let class_value_count = ctx.dt_data.class_value_count;

        let class_val_prime = 2.0_f64.powf((ctx.dt_data.class_value_count as f64).log2().ceil()) as u64;
        let ohe_prime = if ctx.dt_training.rfs_field > class_val_prime as u64 { ctx.dt_training.rfs_field } else { class_val_prime as u64 };
        let discretized_x = ctx.dt_data.discretized_x.clone();
        let mut attr_values_bytes = ohe_conversion(&discretized_x, ctx, attr_value_count, ohe_prime);
        let mut class_values_bytes = ohe_conversion(&y, ctx, class_value_count, ohe_prime);
//        let mut result_vec = Vec::new();
        for current_tree_index in 0..remainder {
//            let dt_shares = ti_receive(
//                ctx.ti_stream.try_clone().expect("failed to clone ti recvr"), ctx);
//            let mut dt_ctx = ctx.clone();
//            dt_ctx.dt_shares = dt_shares;
//            let mut attr_values_bytes_copied = attr_values_bytes.clone();
//            let mut class_values_bytes_copied = class_values_bytes.clone();

//            thread_pool.execute(move||{
//                let mut rfs_x = random_feature_selection(&attr_values_bytes_copied, &mut dt_ctx);
//                rfs_x.append(&mut class_values_bytes_copied);
//
//                let sampling_result = sample_with_replacement(&mut dt_ctx, &rfs_x);
//                dt_ctx.party0_port = current_p0_port;
//                dt_ctx.party1_port = current_p1_port;
//
//                let (internal_addr, external_addr) = get_formatted_address(dt_ctx.party_id, &dt_ctx.party0_ip, dt_ctx.party0_port, &dt_ctx.party1_ip, dt_ctx.party1_port);
//                let (in_stream, o_stream) = try_setup_socket(&internal_addr, &external_addr, &dt_ctx.message_manager);
//                dt_ctx.in_stream = in_stream;
//                dt_ctx.o_stream = o_stream;
//                produce_dt_data(sampling_result, &mut dt_ctx);
//                let max_depth = (&dt_ctx.dt_training).max_depth;
//                let dt_training = decision_tree::train(&mut dt_ctx, max_depth);
//                result_vec.push(dt_ctx.dt_results.result_list);
//                current_p0_port += 1;
//                current_p1_port += 1;
//            });

            let dt_shares = ti_receive(
                ctx.ti_stream.try_clone().expect("failed to clone ti recvr"), ctx);
            let mut dt_ctx = ctx.clone();
            dt_ctx.dt_shares = dt_shares;
            let mut attr_values_bytes_copied = attr_values_bytes.clone();
            let mut class_values_bytes_copied = class_values_bytes.clone();
            let mut rfs_x = random_feature_selection(&attr_values_bytes_copied, &mut dt_ctx);
            rfs_x.append(&mut class_values_bytes_copied);

            let sampling_result = sample_with_replacement(&mut dt_ctx, &rfs_x);
            dt_ctx.party0_port = current_p0_port;
            dt_ctx.party1_port = current_p1_port;

            let (internal_addr, external_addr) = get_formatted_address(dt_ctx.party_id, &dt_ctx.party0_ip, dt_ctx.party0_port, &dt_ctx.party1_ip, dt_ctx.party1_port);
            let (in_stream, o_stream) = try_setup_socket(&internal_addr, &external_addr, &dt_ctx.message_manager);
            dt_ctx.in_stream = in_stream;
            dt_ctx.o_stream = o_stream;
            produce_dt_data(sampling_result, &mut dt_ctx);
            let max_depth = (&dt_ctx.dt_training).max_depth;
            let dt_training = decision_tree::train(&mut dt_ctx, max_depth);
            if ctx.asymmetric_bit == 1 {
                dt_ctx.result_file.write_all("\n".as_bytes());
            }
//            current_p0_port += 1;
//            current_p1_port += 1;
        }

        thread_pool.join();
        ctx.thread_hierarchy.pop();
        let runtime =now.elapsed().unwrap().as_millis();
        println!("complete -- work time = {:5} (ms)", runtime);
        let mut file = File::create(ctx.output_path.clone()+&format!("{}_", ctx.tree_count).to_string()+"runtime.txt").unwrap();
        file.write_all(format!("{}",runtime).as_bytes());
//        println!("{:?}",result_vec);
//        if ctx.asymmetric_bit==1{
//            let mut result_to_write = Vec::new();
//            for result in result_vec{
//                result_to_write.push(result.join(","));
//            }
//            let write_str = result_to_write.join("\n");
//            ctx.result_file.write_all(write_str.as_bytes());
//        }
    }
}
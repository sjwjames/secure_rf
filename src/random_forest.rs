pub mod random_forest {
    use crate::computing_party::computing_party::{ComputingParty, get_formatted_address, try_setup_socket, initialize_party_context, ti_receive, produce_dt_data, load_dt_raw_data, receive_preprocessing_shares, read_shares};
    use crate::decision_tree::decision_tree;
    use std::sync::{Arc, Mutex};
    use threadpool::ThreadPool;
    use crate::field_change::field_change::{change_binary_to_bigint_field, change_binary_to_decimal_field};
    use std::thread::current;
    use crate::message::message::MessageManager;
    use std::collections::HashMap;
    use std::num::Wrapping;
    use crate::protocol::protocol::{matrix_multiplication_integer, batch_equality_integer, batch_equality};
    use crate::utils::utils::{get_additive_shares, send_u64_messages, send_u8_messages, truncate_local};
    use crate::bit_decomposition::bit_decomposition::{bit_decomposition, batch_bit_decomposition};
    use crate::comparison::comparison::{comparison, batch_comparison};
    use crate::constants::constants::BINARY_PRIME;
    use std::fs::File;
    use std::io::{BufReader, BufRead, Write};
    use crate::or_xor::or_xor::or_xor;
    use std::time::SystemTime;
    use num::integer::*;
    use crate::discretize::discretize::{binary_vector_to_ring, discretize, discretize_into_ohe};


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

    pub fn discretize_data(x: &mut Vec<Vec<Wrapping<u64>>>, ctx: &mut ComputingParty) {
        let mut x_temp = Vec::new();
        let x_rows = x.len();
        let x_cols = x[0].len();
        for j in 0..x_cols {
            let mut col = Vec::new();
            for i in 0..x_rows {
                col.push(x[i][j]);
            }
            let discretized = discretize(&col, ctx.dt_data.attr_value_count, ctx);
            x_temp.push(discretized);
        }
        for i in 0..x_rows {
            let mut row = Vec::new();
            for j in 0..x_cols {
                row.push(x_temp[j][i]);
            }
            ctx.dt_data.discretized_x.push(row);
        }
    }

    pub fn discretize_data_ohe(x: &mut Vec<Vec<Wrapping<u64>>>,buckets:usize, ctx: &mut ComputingParty)->Vec<Vec<u8>>{
        let mut x_temp = Vec::new();
        let x_rows = x.len();
        let x_cols = x[0].len();
        for j in 0..x_cols {
            let mut col = Vec::new();
            for i in 0..x_rows {
                col.push(x[i][j]);
            }
            let mut discretized = discretize_into_ohe(&col, buckets, ctx);
            x_temp.append(&mut discretized);
        }
        x_temp
    }

    pub fn ohe_conversion(x: &Vec<Vec<Wrapping<u64>>>, ctx: &mut ComputingParty, category: usize) -> Vec<Vec<u8>> {
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
                        y_row.push(k as u64);
                    } else {
                        y_row.push(0u64);
                    }
                }
                equality_x.append(&mut x_row);
                equality_y.append(&mut y_row);
            }
        }

        let equality_y = binary_vector_to_ring(&equality_y, ctx);
        let equality_y: Vec<Wrapping<u64>> = equality_y.iter().map(|a| Wrapping(a.0 << ctx.decimal_precision as u64)).collect();

        let eq_result = batch_equality(&equality_x, &equality_y, ctx);

        let mut count = 0;
        let mut result = Vec::new();
        for j in 0..cols {
            for k in 0..category {
                let mut row = Vec::new();
                for i in 0..rows {
                    row.push(eq_result[count] as u8);
                    count += 1;
                }
                result.push(row);
            }
        }
        result
    }


    pub fn train(ctx: &mut ComputingParty) {
        ctx.thread_hierarchy.push("RF".to_string());
        let total_now = SystemTime::now();
        let now = SystemTime::now();

        let thread_pool = ThreadPool::with_name(format!("{}", "RF"), ctx.thread_count);

        let mut remainder = ctx.tree_count;
        let mut current_p0_port = ctx.party0_port + 1;
        let mut current_p1_port = ctx.party1_port + 1;
        let mut x = load_dt_raw_data(&ctx.x_input_path);
        let mut y = load_dt_raw_data(&ctx.y_input_path);

        let attr_value_count = ctx.dt_data.attr_value_count;
        let class_value_count = ctx.dt_data.class_value_count;

        discretize_into_ohe(&x,attr_value_count,ctx);
        discretize_data(&mut x, ctx);
        let runtime = now.elapsed().unwrap().as_millis();
        println!("loading & discretization completes -- work time = {:5} (ms)", runtime);

//        receive_preprocessing_shares(ctx);


        let discretized_x = ctx.dt_data.discretized_x.clone();
        let now = SystemTime::now();
        let mut attr_values_bytes = ohe_conversion(&discretized_x, ctx, attr_value_count);
        let runtime = now.elapsed().unwrap().as_millis();
        println!("OHE of attributes completes -- work time = {:5} (ms)", runtime);

        let mut y_temp = Vec::new();
        for mut row in &y {
            y_temp.append(&mut row.iter().map(|a| a.0).collect());
        }
        let y_temp = binary_vector_to_ring(&y_temp, ctx);
        let mut y_converted = Vec::new();
        for i in 0..y.len() {
            let mut row = Vec::new();
            for j in 0..y[0].len() {
                row.push(y_temp[i * y[0].len() + j]);
            }
            y_converted.push(row);
        }

        let now = SystemTime::now();

        let y_converted = y_converted.iter().map(|a| [Wrapping(a[0].0 << ctx.decimal_precision as u64)].to_vec()).collect();
        let mut class_values_bytes = ohe_conversion(&y_converted, ctx, class_value_count);
        let runtime = now.elapsed().unwrap().as_millis();
        println!("OHE of classes completes -- work time = {:5} (ms)", runtime);

        for current_tree_index in 0..remainder {
            let mut dt_ctx = ctx.clone();
            let mut attr_values_bytes_copied = attr_values_bytes.clone();
            let mut class_values_bytes_copied = class_values_bytes.clone();
            thread_pool.execute(move || {
                dt_ctx.party0_port = current_p0_port;
                dt_ctx.party1_port = current_p1_port;
                let (internal_addr, external_addr) = get_formatted_address(dt_ctx.party_id, &dt_ctx.party0_ip, dt_ctx.party0_port, &dt_ctx.party1_ip, dt_ctx.party1_port);
                let (in_stream, o_stream) = try_setup_socket(&internal_addr, &external_addr, &dt_ctx.message_manager);
                dt_ctx.in_stream = in_stream;
                dt_ctx.o_stream = o_stream;
                dt_ctx.dt_shares = read_shares(&mut dt_ctx, current_tree_index);

                let now = SystemTime::now();
                let mut rfs_x = random_feature_selection(&attr_values_bytes_copied, &mut dt_ctx);
                let runtime = now.elapsed().unwrap().as_millis();
                println!("RF completes -- work time = {:5} (ms)", runtime);

                rfs_x.append(&mut class_values_bytes_copied);

                let now = SystemTime::now();
                let sampling_result = sample_with_replacement(&mut dt_ctx, &rfs_x);
                let runtime = now.elapsed().unwrap().as_millis();
                println!("RF completes -- work time = {:5} (ms)", runtime);

                dt_ctx.party0_port = current_p0_port;
                dt_ctx.party1_port = current_p1_port;

                let (internal_addr, external_addr) = get_formatted_address(dt_ctx.party_id, &dt_ctx.party0_ip, dt_ctx.party0_port, &dt_ctx.party1_ip, dt_ctx.party1_port);
                let (in_stream, o_stream) = try_setup_socket(&internal_addr, &external_addr, &dt_ctx.message_manager);
                dt_ctx.in_stream = in_stream;
                dt_ctx.o_stream = o_stream;
                produce_dt_data(sampling_result, &mut dt_ctx);
                let max_depth = (&dt_ctx.dt_training).max_depth;
                let now = SystemTime::now();
                let mut result_file = File::create(format!("{}{}trees_{}", &dt_ctx.output_path, dt_ctx.tree_count, current_tree_index).as_str()).unwrap();

                let dt_training = decision_tree::train(&mut dt_ctx, max_depth, &mut result_file);
                let runtime = now.elapsed().unwrap().as_millis();
                println!("One tree completes -- work time = {:5} (ms)", runtime);
            });
            current_p0_port += 1;
            current_p1_port += 1;
        }

        thread_pool.join();
        ctx.thread_hierarchy.pop();
        let runtime = total_now.elapsed().unwrap().as_millis();
        println!("complete -- work time = {:5} (ms)", runtime);
        let mut file = File::create(ctx.output_path.clone() + &format!("{}_", ctx.tree_count).to_string() + "runtime.txt").unwrap();
        file.write_all(format!("{}", runtime).as_bytes());


        println!("{}", ctx.dt_shares.sequential_binary_index);
        println!("{:?}", ctx.dt_shares.sequential_equality_integer_index);
        println!("{:?}", ctx.dt_shares.sequential_additive_index);
        println!("{}", ctx.dt_shares.sequential_equality_index);
        println!("{}", ctx.dt_shares.sequential_additive_bigint_index);

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
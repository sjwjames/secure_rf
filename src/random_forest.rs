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
    use crate::discretize::discretize::{binary_vector_to_ring, discretize};


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
            for i in 0..x_rows{
                col.push(x[i][j]);
            }
            let discretized = discretize(&col, ctx.dt_data.attr_value_count, ctx);
            x_temp.push(discretized);
        }
        println!("x_temp:{:?}",x_temp);
        for i in 0..x_rows {
            let mut row = Vec::new();
            for j in 0..x_cols {
                row.push(x_temp[j][i]);
            }
            ctx.dt_data.discretized_x.push(row);
        }
        println!("x:{:?}",ctx.dt_data.discretized_x[0]);


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
//                    x_row.push(truncate_local(Wrapping((x[i][j].0 as f64 * 2f64.powf(ctx.decimal_precision as f64)) as u64),ctx.decimal_precision,ctx.asymmetric_bit));
//                    if ctx.asymmetric_bit == 1 {
//                        y_row.push(truncate_local(Wrapping((k as f64 * 2f64.powf(ctx.decimal_precision as f64)) as u64),ctx.decimal_precision,ctx.asymmetric_bit));
//                    } else {
//                        y_row.push(Wrapping(0));
//                    }

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
        let equality_y:Vec<Wrapping<u64>> = equality_y.iter().map(|a| Wrapping(a.0 << ctx.decimal_precision as u64)).collect();

        let eq_result = batch_equality(&equality_x, &equality_y, ctx);
//        let eq_received = send_u64_messages(ctx,&eq_result);
//        let eq_result_revealed:Vec<u64> = eq_result.iter().zip(eq_received).map(|(a,b)|a.0^b.0).collect();
//        println!("eq_result_revealed:{:?}",eq_result_revealed);

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
        let now = SystemTime::now();

        let thread_pool = ThreadPool::with_name(format!("{}", "RF"), ctx.thread_count);

        let mut remainder = ctx.tree_count;
        let mut current_p0_port = ctx.party0_port + 1;
        let mut current_p1_port = ctx.party1_port + 1;
        let mut x = load_dt_raw_data(&ctx.x_input_path);
        let mut y = load_dt_raw_data(&ctx.y_input_path);

        discretize_data(&mut x, ctx);
        receive_preprocessing_shares(ctx);

        let attr_value_count = ctx.dt_data.attr_value_count;
        let class_value_count = ctx.dt_data.class_value_count;

        let attr_val_bit_length = ((attr_value_count as f64).log2().ceil() + 1.0) as usize;
        let class_val_bit_length = ((class_value_count as f64).log2().ceil() + 1.0) as usize;

        let discretized_x = ctx.dt_data.discretized_x.clone();
        let mut attr_values_bytes = ohe_conversion(&discretized_x, ctx, attr_value_count);
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
//        let mut y_temp = Vec::new();
//        for i in 0..y.len(){
//            y_temp.push(y[i][0]);
//        }
//        let mut y_converted = discretize(&y_temp,ctx.dt_data.class_value_count,ctx);
//        let mut y_temp = Vec::new();
//        for item in y_converted{
//            y_temp.push([item].to_vec());
//        }
        let y_converted = y_converted.iter().map(|a|[Wrapping(a[0].0<<ctx.decimal_precision as u64)].to_vec()).collect();
        let mut class_values_bytes = ohe_conversion(&y_converted, ctx, class_value_count);
//        println!("{:?}", ctx.dt_shares.sequential_equality_integer_index);
//        println!("{:?}", ctx.dt_shares.sequential_additive_index);
//        println!("{}", ctx.dt_shares.sequential_equality_index);
//        println!("{}", ctx.dt_shares.sequential_additive_bigint_index);
//        println!("attr_values_bytes:{:?}", attr_values_bytes[attr_value_count-1]);
//        println!("class_values_bytes:{:?}", class_values_bytes[0]);

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
        let runtime = now.elapsed().unwrap().as_millis();
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
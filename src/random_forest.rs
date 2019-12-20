pub mod random_forest {
    use crate::computing_party::computing_party::{ComputingParty, get_formatted_address, try_setup_socket, initialize_party_context, ti_receive, reset_share_indices};
    use crate::decision_tree::decision_tree;
    use std::sync::{Arc, Mutex};
    use threadpool::ThreadPool;
    use crate::field_change::field_change::{change_binary_to_bigint_field, change_binary_to_decimal_field};
    use std::thread::current;
    use crate::message::message::MessageManager;
    use std::collections::HashMap;


    pub fn train(ctx: &mut ComputingParty) {
        ctx.thread_hierarchy.push("RF".to_string());
        let thread_pool = ThreadPool::with_name(format!("{}", "RF"), ctx.thread_count);
        let mut remainder = ctx.tree_count;
        let mut current_p0_port = ctx.party0_port + 1;
        let mut current_p1_port = ctx.party1_port + 1;

        for current_tree_index in 0..remainder {
            let dt_shares = ti_receive(
                ctx.ti_stream.try_clone().expect("failed to clone ti recvr"));
            let mut dt_ctx = ctx.clone();
            dt_ctx.dt_shares = dt_shares;

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
//                let dt_training = decision_tree::train(&mut dt_ctx);
//            });

            dt_ctx.party0_port = current_p0_port;
            dt_ctx.party1_port = current_p1_port;
            reset_share_indices(&mut dt_ctx);
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
            // multi-thread could help
            for item in attr_values_bytes.iter() {
                let mut attr_data_item = Vec::new();
                let mut attr_data_bigint_item = Vec::new();
                for data_item in item.iter() {
                    attr_data_item.push(change_binary_to_decimal_field(data_item, &mut dt_ctx));
                    attr_data_bigint_item.push(change_binary_to_bigint_field(data_item, &mut dt_ctx));
                }
                attr_values.push(attr_data_item);
                attr_values_bigint.push(attr_data_bigint_item);
            }
            dt_ctx.dt_data.attr_values = attr_values;
            dt_ctx.dt_data.attr_values_big_integer = attr_values_bigint;

            let mut class_value_bytes = dt_ctx.dt_data.class_values_bytes.clone();
            for item in class_value_bytes.iter() {
                class_values.push(change_binary_to_decimal_field(item, &mut dt_ctx));
                class_values_bigint.push(change_binary_to_bigint_field(item, &mut dt_ctx));
            }

            dt_ctx.dt_data.class_values = class_values;
            dt_ctx.dt_data.class_values_big_integer = class_values_bigint;
            let max_depth = (&dt_ctx.dt_training).max_depth;
            let dt_training = decision_tree::train(&mut dt_ctx,max_depth);

            current_p0_port += 1;
            current_p1_port += 1;
        }

        thread_pool.join();
        ctx.thread_hierarchy.pop();
    }
}
pub mod random_forest {
    use crate::computing_party::computing_party::{ComputingParty, get_formatted_address, try_setup_socket, initialize_party_context, ti_receive};
    use crate::decision_tree::decision_tree;
    use std::sync::{Arc, Mutex};
    use threadpool::ThreadPool;

    pub struct RandomForest {}

    pub fn train(mut ctx: ComputingParty) {
        let thread_pool = ThreadPool::new(ctx.thread_count);
        let mut remainder = ctx.tree_count;
        let mut party0_port = Arc::new(Mutex::new(ctx.party0_port));
        let mut party1_port = Arc::new(Mutex::new(ctx.party1_port));
        while remainder > 0 {
            let dt_shares = ti_receive(
                ctx.ti_stream.try_clone().expect("failed to clone ti recvr"));
            let mut dt_ctx = ctx.clone();
            let mut p0_port = Arc::clone(&party0_port);
            let mut p1_port = Arc::clone(&party1_port);
            thread_pool.execute(move || {
                let mut p0_port = p0_port.lock().unwrap();
                *p0_port += 1;
                dt_ctx.party0_port = *p0_port;

                let mut p1_port = p1_port.lock().unwrap();
                *p1_port+=1;
                dt_ctx.party1_port = *p1_port;

                let (internal_addr, external_addr) = get_formatted_address(dt_ctx.party_id, &dt_ctx.party0_ip, dt_ctx.party0_port, &dt_ctx.party1_ip, dt_ctx.party1_port);
                let (in_stream, o_stream) = try_setup_socket(&internal_addr, &external_addr);
                dt_ctx.in_stream = in_stream;
                dt_ctx.o_stream = o_stream;
                let dt_training = decision_tree::train(dt_ctx,dt_shares);
            });
            remainder -= 1;
        }
        thread_pool.join();
    }
}
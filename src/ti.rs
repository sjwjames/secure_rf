pub mod ti {
    //author Davis, email:daviscrailsback@gmail.com
    extern crate rand;
    extern crate num;
    extern crate serde;

    use rand::Rng;
    use std::time::SystemTime;
    use std::thread;
    use std::net::{TcpStream, TcpListener, SocketAddr, Shutdown};
    use std::io::{Read, Write, BufWriter};
    use std::num::Wrapping;
    use std::io;
    use crate::constants::constants;
    use std::sync::{Arc, Mutex, Barrier};
    use num::bigint::{BigUint, ToBigUint, ToBigInt, RandBigInt};
    use num::integer::*;
    use self::num::{One, Zero, BigInt, checked_pow, Num};
    use std::ops::{Add, Sub, Mul};
    use crate::constants::constants::BINARY_PRIME;
    use crate::decision_tree::decision_tree::{DecisionTreeShares, DecisionTreeTIShareMessage};
    use serde::{Serialize, Deserialize, Serializer};
    use std::str::FromStr;
    use threadpool::ThreadPool;
    use std::collections::HashMap;
    use crate::utils::utils::{big_uint_subtract, send_u8_messages, mod_subtraction, mod_subtraction_u8};
    use std::f64::consts::E;
    use std::fs::File;


    pub struct TI {
        pub ti_ip: String,
        pub ti_port0: u16,
        pub ti_port1: u16,
        pub test_mode: bool,
        pub add_shares_per_tree: usize,
        pub add_shares_bigint_per_tree: usize,
        pub equality_shares_per_tree: usize,
        pub binary_shares_per_tree: usize,
        pub ohe_add_shares: u64,
        pub ohe_binary_shares: u64,
        pub ohe_equality_shares: u64,
        pub tree_count: usize,
        pub batch_size: usize,
        pub tree_training_batch_size: usize,
        pub thread_count: usize,
        pub big_int_prime: BigUint,
        pub prime: u64,
        pub bigint_bit_size: usize,
        pub attr_value_cnt: u64,
        pub class_value_cnt: u64,
        pub feature_selected: u64,
        pub instance_selected: u64,
        pub rfs_field: u64,
        pub bagging_field: u64,
        pub dataset_size_prime: u64,
        pub instance_cnt: u64,
        pub feature_cnt: u64,
        pub output_path: String,
        pub fs_selection_file: File,
        pub sampling_file: File,
    }

    const TI_BATCH_SIZE: usize = constants::TI_BATCH_SIZE;
    const U64S_PER_TX: usize = constants::U64S_PER_TX;
    const U8S_PER_TX: usize = constants::U8S_PER_TX;

    union Xbuffer {
        u64_buf: [u64; U64S_PER_TX],
        u8_buf: [u8; U8S_PER_TX],
    }

    impl Clone for TI {
        fn clone(&self) -> Self {
            TI {
                ti_ip: self.ti_ip.clone(),
                ti_port0: self.ti_port0,
                ti_port1: self.ti_port1,
                test_mode: self.test_mode,
                add_shares_per_tree: self.add_shares_per_tree,
                add_shares_bigint_per_tree: self.add_shares_bigint_per_tree,
                equality_shares_per_tree: self.equality_shares_per_tree,
                binary_shares_per_tree: self.equality_shares_per_tree,
                ohe_add_shares: self.ohe_add_shares,
                ohe_binary_shares: self.ohe_binary_shares,
                ohe_equality_shares: self.ohe_equality_shares,
                tree_count: self.tree_count,
                batch_size: self.batch_size,
                tree_training_batch_size: self.tree_training_batch_size,
                big_int_prime: self.big_int_prime.clone(),
                prime: self.prime,
                thread_count: self.thread_count,
                bigint_bit_size: self.bigint_bit_size,
                attr_value_cnt: self.attr_value_cnt,
                feature_selected: self.feature_selected,
                instance_selected: self.instance_selected,
                rfs_field: self.rfs_field,
                class_value_cnt: self.class_value_cnt,
                bagging_field: self.bagging_field,
                dataset_size_prime: self.dataset_size_prime,
                instance_cnt: self.instance_cnt,
                feature_cnt: self.feature_cnt,
                output_path: self.output_path.clone(),
                fs_selection_file: self.fs_selection_file.try_clone().unwrap(),
                sampling_file: self.sampling_file.try_clone().unwrap(),
            }
        }
    }


    pub fn initialize_ti_context(settings_file: String) -> TI {
        let mut settings = config::Config::default();
        settings
            .merge(config::File::with_name(settings_file.as_str())).unwrap()
            .merge(config::Environment::with_prefix("APP")).unwrap();

        let ti_ip = match settings.get_str("ti_ip") {
            Ok(num) => num as String,
            Err(error) => {
                panic!("Encountered a problem while parsing ti_ip: {:?} ", error)
            }
        };

        let ti_port0 = match settings.get_int("ti_port0") {
            Ok(num) => num as u16,
            Err(error) => {
                panic!("Encountered a problem while parsing ti_port0: {:?} ", error)
            }
        };

        let ti_port1 = match settings.get_int("ti_port1") {
            Ok(num) => num as u16,
            Err(error) => {
                panic!("Encountered a problem while parsing ti_port1: {:?} ", error)
            }
        };


        let add_shares_per_tree = match settings.get_int("add_shares_per_tree") {
            Ok(num) => num as usize,
            Err(error) => {
                panic!("Encountered a problem while parsing add_shares_per_tree: {:?}", error)
            }
        };

        let add_shares_bigint_per_tree = match settings.get_int("add_shares_bigint_per_tree") {
            Ok(num) => num as usize,
            Err(error) => {
                panic!("Encountered a problem while parsing add_shares_bigint_per_tree: {:?}", error)
            }
        };

        let equality_shares_per_tree = match settings.get_int("equality_shares_per_tree") {
            Ok(num) => num as usize,
            Err(error) => {
                panic!("Encountered a problem while parsing equality_shares_per_tree: {:?}", error)
            }
        };

        let binary_shares_per_tree = match settings.get_int("binary_shares_per_tree") {
            Ok(num) => num as usize,
            Err(error) => {
                panic!("Encountered a problem while parsing binary_shares_per_tree: {:?}", error)
            }
        };

        let tree_count = match settings.get_int("tree_count") {
            Ok(num) => num as usize,
            Err(error) => {
                panic!("Encountered a problem while parsing tree_count: {:?}", error)
            }
        };

        let batch_size = match settings.get_int("batch_size") {
            Ok(num) => num as usize,
            Err(error) => {
                panic!("Encountered a problem while parsing batch_size: {:?}", error)
            }
        };

        let tree_training_batch_size = match settings.get_int("tree_training_batch_size") {
            Ok(num) => num as usize,
            Err(error) => {
                panic!("Encountered a problem while parsing tree_training_batch_size: {:?}", error)
            }
        };

        let big_int_prime_str = match settings.get_str("big_int_prime") {
            Ok(num) => num as String,
            Err(error) => {
                panic!("Encountered a problem while parsing big_int_prime: {:?}", error)
            }
        };

        let big_int_prime = BigUint::from_str_radix(&big_int_prime_str, 10).unwrap();


        let prime = match settings.get_int("prime") {
            Ok(num) => num as u64,
            Err(error) => {
                panic!("Encountered a problem while parsing prime: {:?}", error)
            }
        };

        let thread_count = match settings.get_int("thread_count") {
            Ok(num) => num as usize,
            Err(error) => {
                panic!("Encountered a problem while parsing thread_count: {:?}", error)
            }
        };

        let bigint_bit_size = match settings.get_int("bigint_bit_size") {
            Ok(num) => num as usize,
            Err(error) => {
                panic!("Encountered a problem while parsing bigint_bit_size: {:?}", error)
            }
        };

        let attr_value_cnt = match settings.get_int("attr_value_cnt") {
            Ok(num) => num as u64,
            Err(error) => {
                panic!("Encountered a problem while parsing attr_value_cnt: {:?}", error)
            }
        };

        let instance_selected = match settings.get_int("instance_selected") {
            Ok(num) => num as u64,
            Err(error) => {
                panic!("Encountered a problem while parsing instance_selected: {:?}", error)
            }
        };

        let feature_selected = match settings.get_int("feature_selected") {
            Ok(num) => num as u64,
            Err(error) => {
                panic!("Encountered a problem while parsing feature_selected: {:?}", error)
            }
        };

        let instance_cnt = match settings.get_int("instance_cnt") {
            Ok(num) => num as u64,
            Err(error) => {
                panic!("Encountered a problem while parsing instance_cnt: {:?}", error)
            }
        };

        let feature_cnt = match settings.get_int("feature_cnt") {
            Ok(num) => num as u64,
            Err(error) => {
                panic!("Encountered a problem while parsing feature_cnt: {:?}", error)
            }
        };

        let class_value_cnt = match settings.get_int("class_value_cnt") {
            Ok(num) => num as u64,
            Err(error) => {
                panic!("Encountered a problem while parsing class_value_cnt: {:?}", error)
            }
        };

        let ohe_add_shares = match settings.get_int("ohe_add_shares") {
            Ok(num) => num as u64,
            Err(error) => {
                panic!("Encountered a problem while parsing ohe_add_shares: {:?}", error)
            }
        };

        let ohe_binary_shares = match settings.get_int("ohe_binary_shares") {
            Ok(num) => num as u64,
            Err(error) => {
                panic!("Encountered a problem while parsing ohe_binary_shares: {:?}", error)
            }
        };

        let ohe_equality_shares = match settings.get_int("ohe_equality_shares") {
            Ok(num) => num as u64,
            Err(error) => {
                panic!("Encountered a problem while parsing ohe_equality_shares: {:?}", error)
            }
        };

        let output_path = match settings.get_str("output_path") {
            Ok(string) => string,
            Err(error) => {
                panic!("Encountered a problem while parsring weights_output_path: {:?}", error)
            }
        };

        let test_mode = match settings.get_bool("test_mode") {
            Ok(ans) => ans as bool,
            Err(error) => {
                panic!("Encountered a problem while parsing test_mode: {:?}", error)
            }
        };

        let mut fs_file = File::create(output_path.clone() + &format!("{}_", tree_count).to_string() + "fs_selection.csv").unwrap();
        let mut sampling_file = File::create(output_path.clone() + &format!("{}_", tree_count).to_string() + "sampling_selection.csv").unwrap();

        let rfs_field = 2.0_f64.powf((attr_value_cnt as f64).log2().ceil()) as u64;

        let bagging_field = 2.0_f64.powf((instance_selected as f64).log2().ceil()) as u64;

        let dataset_size_prime = 2.0_f64.powf((instance_selected as f64).log2().ceil()) as u64;

        TI {
            ti_ip,
            ti_port0,
            ti_port1,
            test_mode,
            add_shares_per_tree,
            add_shares_bigint_per_tree,
            equality_shares_per_tree,
            binary_shares_per_tree,
            ohe_add_shares,
            ohe_binary_shares,
            ohe_equality_shares,
            tree_count,
            batch_size,
            tree_training_batch_size,
            big_int_prime,
            prime,
            thread_count,
            bigint_bit_size,
            attr_value_cnt,
            class_value_cnt,
            feature_selected,
            instance_selected,
            rfs_field,
            bagging_field,
            dataset_size_prime,
            instance_cnt,
            feature_cnt,
            output_path,
            fs_selection_file: fs_file,
            sampling_file,
        }
    }

    fn send_u64_shares(shares: Vec<Wrapping<u64>>, stream: &mut TcpStream) {
        let mut stream = stream.try_clone().unwrap();
        let mut batches: usize = 0;
        let mut data_len = shares.len();
        let mut current_batch = 0;
        let mut push_buf = Xbuffer { u64_buf: [0u64; U64S_PER_TX] };
        batches = (data_len as f64 / U64S_PER_TX as f64).ceil() as usize;
        while current_batch < batches {
            for i in 0..U64S_PER_TX {
                unsafe {
                    if current_batch * U64S_PER_TX + i < data_len {
                        push_buf.u64_buf[i] = shares[current_batch * U64S_PER_TX + i].0;
                    } else {
                        break;
                    }
                }
            }
            let mut bytes_written = 0;
            while bytes_written < U8S_PER_TX {
                let current_bytes = unsafe {
                    stream.write(&push_buf.u8_buf[bytes_written..])
                };
                bytes_written += current_bytes.unwrap();
            }
            current_batch += 1;
        }
    }

    fn send_u8_shares(shares: Vec<u8>, stream: &mut TcpStream) {
        let mut stream = stream.try_clone().unwrap();
        let mut batches: usize = 0;
        let mut data_len = shares.len();
        let mut current_batch = 0;
        let mut push_buf = Xbuffer { u8_buf: [0u8; U8S_PER_TX] };
        batches = (data_len as f64 / U8S_PER_TX as f64).ceil() as usize;
        while current_batch < batches {
            for i in 0..U8S_PER_TX {
                unsafe {
                    if current_batch * U8S_PER_TX + i < data_len {
                        push_buf.u8_buf[i] = shares[current_batch * U8S_PER_TX + i];
                    } else {
                        break;
                    }
                }
            }
            let mut bytes_written = 0;
            while bytes_written < U8S_PER_TX {
                let current_bytes = unsafe {
                    stream.write(&push_buf.u8_buf[bytes_written..])
                };
                bytes_written += current_bytes.unwrap();
            }
            current_batch += 1;
        }
    }

    fn send_preprocessing_shares(additive_shares: (Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)>), binary_shares: (Vec<(u8, u8, u8)>), equality_shares: Vec<Wrapping<u64>>, stream: &mut TcpStream) {
        let mut additive_shares_sent = Vec::new();
        for item in &additive_shares {
            additive_shares_sent.push(item.0.clone());
            additive_shares_sent.push(item.1.clone());
            additive_shares_sent.push(item.2.clone());
        }
        send_u64_shares(additive_shares_sent, stream);

        let mut binary_shares_sent = Vec::new();
        for item in &binary_shares {
            binary_shares_sent.push(item.0);
            binary_shares_sent.push(item.1);
            binary_shares_sent.push(item.2);
        }
        send_u8_shares(binary_shares_sent, stream);

        send_u64_shares(equality_shares, stream);
    }


    pub fn generate_preprocessing_shares(ctx: &mut TI, thread_pool: &ThreadPool, streams: Vec<&TcpStream>) {
        let mut add_amount = ctx.ohe_add_shares as usize;
        let mut eq_amount = ctx.ohe_equality_shares as usize;
        let mut binary_amount = ctx.ohe_binary_shares as usize;
        let prime = ctx.prime;
//        let u64_prime = (2.0f64.powf(64.0) as u64);

        // OHE shares of attrs
        print!("preprocessing- generating additive shares...      ");
        let now = SystemTime::now();
        let (add_triples0, add_triples1) = additive_share_helper(ctx, thread_pool, BINARY_PRIME as u64, add_amount);
        println!("complete -- work time = {:5} (ms)", now.elapsed().unwrap().as_millis());


        print!("preprocessing- generating binary shares...           ");
        let now = SystemTime::now();
        let (binary_triples0, binary_triples1) = generate_binary_shares(&ctx, thread_pool, binary_amount);
        println!("complete -- work time = {:5} (ms)",
                 now.elapsed().unwrap().as_millis());

        print!("preprocessing- generating equality integer shares...           ");
        let now = SystemTime::now();
        let (equality_share0, equality_share1) = equality_integer_shares_helper(ctx, thread_pool, prime, eq_amount);
        println!("complete -- work time = {:5} (ms)",
                 now.elapsed().unwrap().as_millis());

//        println!("add_triples0:{:?}", add_triples0[0]);
//        println!("add_triples1:{:?}", add_triples1[0]);
//        println!("binary_triples0:{:?}", binary_triples0[0]);
//        println!("binary_triples1:{:?}", binary_triples1[0]);
//        println!("equality_share0:{}", equality_share0[0]);
//        println!("equality_share1:{}", equality_share1[0]);

        let mut stream0 = streams[0].try_clone().unwrap();
        stream0.set_ttl(std::u32::MAX).expect("set_ttl call failed");
        stream0.set_write_timeout(None).expect("set_write_timeout call failed");
        stream0.set_read_timeout(None).expect("set_read_timeout call failed");
        thread_pool.execute(move || {
            send_preprocessing_shares(add_triples0, binary_triples0, equality_share0, &mut stream0);
        });

        let mut stream1 = streams[1].try_clone().unwrap();
        stream1.set_ttl(std::u32::MAX).expect("set_ttl call failed");
        stream1.set_write_timeout(None).expect("set_write_timeout call failed");
        stream1.set_read_timeout(None).expect("set_read_timeout call failed");
        thread_pool.execute(move || {
            send_preprocessing_shares(add_triples1, binary_triples1, equality_share1, &mut stream1);
        });
        thread_pool.join();

        // OHE shares of classes
//        if class_val_prime != ctx.rfs_field {
//            print!("preprocessing- generating additive shares...      ");
//            let now = SystemTime::now();
//            let (add_triples0, add_triples1) = additive_share_helper(ctx, thread_pool, class_val_prime, add_amount);
//            println!("complete -- work time = {:5} (ms)", now.elapsed().unwrap().as_millis());
//
//            print!("preprocessing- generating equality integer shares...           ");
//            let now = SystemTime::now();
//            let (equality_share0, equality_share1) = equality_integer_shares_helper(ctx, thread_pool, class_val_prime, eq_amount);
//            println!("complete -- work time = {:5} (ms)",
//                     now.elapsed().unwrap().as_millis());
//
//            let mut stream0 = streams[0].try_clone().unwrap();
//            stream0.set_ttl(std::u32::MAX).expect("set_ttl call failed");
//            stream0.set_write_timeout(None).expect("set_write_timeout call failed");
//            stream0.set_read_timeout(None).expect("set_read_timeout call failed");
//            thread_pool.execute(move || {
//                send_preprocessing_shares(add_triples0, vec![], equality_share0, &mut stream0);
//            });
//
//            let mut stream1 = streams[1].try_clone().unwrap();
//            stream1.set_ttl(std::u32::MAX).expect("set_ttl call failed");
//            stream1.set_write_timeout(None).expect("set_write_timeout call failed");
//            stream1.set_read_timeout(None).expect("set_read_timeout call failed");
//            thread_pool.execute(move || {
//                send_preprocessing_shares(add_triples1, vec![], equality_share1, &mut stream1);
//            });
//            thread_pool.join();
//        }
    }

    pub fn run_ti_module(ctx: &mut TI) {
        // TODO log module
        let prefix = "main:      ";
        let s0_pfx = "server 0:  ";
        let s1_pfx = "server 1:  ";

       let socket0: SocketAddr = format!("{}:{}", &ctx.ti_ip, ctx.ti_port0)
           .parse()
           .expect("unable to parse internal socket address");

       let socket1: SocketAddr = format!("{}:{}", &ctx.ti_ip, ctx.ti_port1)
           .parse()
           .expect("unable to parse external socket address");

       let listener0 = TcpListener::bind(&socket0)
           .expect("unable to establish Tcp Listener");

       let listener1 = TcpListener::bind(&socket1)
           .expect("unable to establish Tcp Listener");

       println!("{} listening on port {}", &s0_pfx, listener0.local_addr().unwrap());
       println!("{} listening on port {}", &s1_pfx, listener1.local_addr().unwrap());

       let in_stream0 = match listener0.accept() {
           Ok((stream, _addr)) => stream,
           Err(_) => panic!("server 0: failed to accept connection"),
       };

       let in_stream1 = match listener1.accept() {
           Ok((stream, _addr)) => stream,
           Err(_) => panic!("server 1: failed to accept connection"),
       };
       println!("{} accepted connection from {}", &s0_pfx, in_stream0.peer_addr().unwrap());
       println!("{} accepted connection from {}", &s1_pfx, in_stream1.peer_addr().unwrap());


        let thread_pool = ThreadPool::new(ctx.thread_count);

        //preprocess shares, discretization and OHE
       generate_preprocessing_shares(ctx, &thread_pool, [&in_stream0, &in_stream1].to_vec());

        for i in 0..ctx.tree_count {
            print!("{} [{}] generating additive shares...      ", &prefix, i);
            let now = SystemTime::now();
            let (add_triples0, add_triples1) = generate_additive_shares(&ctx, &thread_pool);
            println!("complete -- work time = {:5} (ms)", now.elapsed().unwrap().as_millis());


            print!("{} [{}] generating binary shares...           ", &prefix, i);
            let now = SystemTime::now();
            let (binary_triples0, binary_triples1) = generate_binary_shares(&ctx, &thread_pool, ctx.binary_shares_per_tree);
            println!("complete -- work time = {:5} (ms)",
                     now.elapsed().unwrap().as_millis());

            print!("{} [{}] generating equality bigint shares...           ", &prefix, i);
            let now = SystemTime::now();
            let (equality_bigint_share0, equality_bigint_share1) = generate_equality_bigint_shares(&ctx, &thread_pool);
            println!("complete -- work time = {:5} (ms)",
                     now.elapsed().unwrap().as_millis());
            print!("{} [{}] generating additive bigint shares...           ", &prefix, i);
            let now = SystemTime::now();
            let (additive_bigint_share0, additive_bigint_share1) = generate_additive_bigint_shares(&ctx, &thread_pool);
            println!("complete -- work time = {:5} (ms)",
                     now.elapsed().unwrap().as_millis());


            let now = SystemTime::now();
            let (rfs_share0, rfs_share1) = generate_rfs_share(&ctx);
            println!("complete -- work time = {:5} (ms)",
                     now.elapsed().unwrap().as_millis());

            let now = SystemTime::now();
            let (bagging_share0, bagging_share1) = generate_bagging_share(&ctx);
            println!("complete -- work time = {:5} (ms)",
                     now.elapsed().unwrap().as_millis());

            let now = SystemTime::now();
            let (matrix_mul_share0, matrix_mul_share1) = generate_matrix_shares(&ctx, (ctx.attr_value_cnt * ctx.feature_selected) as usize, (ctx.attr_value_cnt * ctx.feature_cnt) as usize, ctx.instance_cnt as usize, BINARY_PRIME as u64);
            println!("complete -- work time = {:5} (ms)",
                     now.elapsed().unwrap().as_millis());

            print!("{} [{}] generating equality integer shares...           ", &prefix, i);
            let now = SystemTime::now();
            let (equality_share0, equality_share1) = generate_equality_integer_shares(&ctx, &thread_pool);
            println!("complete -- work time = {:5} (ms)",
                     now.elapsed().unwrap().as_millis());

            let now = SystemTime::now();
            let (bagging_matrix_mul_share0, bagging_matrix_mul_share1) = generate_matrix_shares(&ctx, (ctx.feature_selected * ctx.attr_value_cnt + ctx.class_value_cnt) as usize, ctx.instance_cnt as usize, ctx.instance_selected as usize, BINARY_PRIME as u64);
            println!("complete -- work time = {:5} (ms)",
                     now.elapsed().unwrap().as_millis());

            let mut share0 = DecisionTreeShares {
                additive_triples: add_triples0,
                additive_bigint_triples: additive_bigint_share0,
                rfs_shares: rfs_share0,
                bagging_shares: bagging_share0,
                binary_triples: binary_triples0,
                equality_shares: equality_bigint_share0,
                equality_integer_shares: equality_share0,
                current_additive_index: Arc::new(Mutex::new(0)),
                current_additive_bigint_index: Arc::new(Mutex::new(0)),
                current_equality_index: Arc::new(Mutex::new(0)),
                current_binary_index: Arc::new(Mutex::new(0)),
                sequential_additive_index: Default::default(),
                sequential_additive_bigint_index: 0,
                sequential_equality_index: 0,
                sequential_binary_index: 0,
                sequential_equality_integer_index: Default::default(),
                sequential_ohe_additive_index: 0,
                matrix_mul_shares: matrix_mul_share0,
                bagging_matrix_mul_shares: bagging_matrix_mul_share0,
            };

            let mut share1 = DecisionTreeShares {
                additive_triples: add_triples1,
                additive_bigint_triples: additive_bigint_share1,
                rfs_shares: rfs_share1,
                bagging_shares: bagging_share1,
                binary_triples: binary_triples1,
                equality_shares: equality_bigint_share1,
                equality_integer_shares: equality_share1,
                current_additive_index: Arc::new(Mutex::new(0)),
                current_additive_bigint_index: Arc::new(Mutex::new(0)),
                current_equality_index: Arc::new(Mutex::new(0)),
                current_binary_index: Arc::new(Mutex::new(0)),
                sequential_additive_index: Default::default(),
                sequential_additive_bigint_index: 0,
                sequential_equality_index: 0,
                sequential_binary_index: 0,
                sequential_equality_integer_index: Default::default(),
                sequential_ohe_additive_index: 0,
                matrix_mul_shares: matrix_mul_share1,
                bagging_matrix_mul_shares: bagging_matrix_mul_share1,
            };

            let stream0 = in_stream0.try_clone().expect("server 0: failed to clone stream");
            let sender_thread0 = thread::spawn(move || {
                match get_confirmation(stream0.try_clone()
                .expect("server 0: failed to clone stream")) {
                    Ok(_) => return send_dt_shares(stream0.try_clone()
                            .expect("server 0: failed to clone stream"), share0),
                        Err(e) => return Err(e),
                }
            });

            let stream1 = in_stream1.try_clone().expect("server 1: failed to clone stream");
            let sender_thread1 = thread::spawn(move || {
                match get_confirmation(stream1.try_clone()
                .expect("server 1: failed to clone stream")) {
                    Ok(_) => return send_dt_shares(stream1.try_clone()
                            .expect("server 1: failed to clone stream"), share1),
                        Err(e) => return Err(e),
                }
            });
//            let stream = in_stream0.try_clone().expect("server 0: failed to clone stream");
            // let mut file0 = File::create(format!("{}_{}shares_tree_{}_0.txt", &ctx.output_path, ctx.tree_count, i).as_str()).unwrap();
            // let mut file1 = File::create(format!("{}_{}shares_tree_{}_1.txt", &ctx.output_path, ctx.tree_count, i).as_str()).unwrap();

            // let sender_thread0 = thread::spawn(move || {
            //     write_shares_to_file(share0, &mut file0);
            // });

//            let stream = in_stream1.try_clone().expect("server 0: failed to clone stream");
            // let sender_thread1 = thread::spawn(move || {
            //     write_shares_to_file(share1, &mut file1)
            // });

            match sender_thread0.join() {
                Ok(_) => println!("{} [{}] correlated randomnness sent...     complete -- work time = {:5} (ms)",
                                  &s0_pfx, i, now.elapsed().unwrap().as_millis()),
                Err(_) => panic!("main: failed to join sender 0"),
            };//.expect("main: failed to rejoin server 0");

            match sender_thread1.join() {
                Ok(_) => println!("{} [{}] correlated randomnness sent...     complete -- work time = {:5} (ms)",
                                  &s1_pfx, i, now.elapsed().unwrap().as_millis()),
                Err(_) => panic!("main: failed to join sender 1"),
            };//.expect("main: failed to rejoin server 0");
        }
    }

    fn write_shares_to_file(mut shares: DecisionTreeShares, file: &mut File) {
        //////////////////////// SEND ADDITIVES ////////////////////////

        let mut additive_shares = shares.additive_triples;

        //////////////////////// SEND ADDITIVES ////////////////////////

        let mut binary_triples = shares.binary_triples;
        let mut binary_share_str_vec = Vec::new();
        for item in binary_triples.iter() {
            binary_share_str_vec.push(serde_json::to_string(&item).unwrap());
        }


        //////////////////////// ADDITIVE BIGINT ////////////////////////
        let mut additive_bigint_triples = shares.additive_bigint_triples;
        let mut additive_bigint_str_vec = Vec::new();
        for item in additive_bigint_triples.iter() {
            let mut tuples = Vec::new();
//            tuples.push(serde_json::to_string(&(item.0.to_bytes_le())).unwrap());
//            tuples.push(serde_json::to_string(&(item.1.to_bytes_le())).unwrap());
//            tuples.push(serde_json::to_string(&(item.2.to_bytes_le())).unwrap());

            tuples.push(item.0.to_string());
            tuples.push(item.1.to_string());
            tuples.push(item.2.to_string());

            additive_bigint_str_vec.push(tuples.join("&"));
        }

        //////////////////////// EQUALITY BIGINT ////////////////////////
        let mut equality_bigint_triples = shares.equality_shares;
        let mut equality_bigint_str_vec = Vec::new();
        for item in equality_bigint_triples.iter() {
            equality_bigint_str_vec.push(item.to_string());
        }

        //////////////////////// RFS Shares ////////////////////////
        let mut rfs_shares = shares.rfs_shares;
        let mut rfs_shares_str_vec = Vec::new();
        for item in rfs_shares.iter() {
            rfs_shares_str_vec.push(serde_json::to_string(&item).unwrap());
        }

        //////////////////////// Bagging Shares ////////////////////////
        let mut bagging_shares = shares.bagging_shares;
        let mut bagging_shares_str_vec = Vec::new();
        for item in bagging_shares.iter() {
            bagging_shares_str_vec.push(serde_json::to_string(&item).unwrap());
        }

        //////////////////////// Matrix Multiplication Shares ////////////////////////
        let mut matrix_mul_share = shares.matrix_mul_shares;
        let mut matrix_mul_share_str_vec = Vec::new();
        matrix_mul_share_str_vec.push(serde_json::to_string(&matrix_mul_share.0).unwrap());
        matrix_mul_share_str_vec.push(serde_json::to_string(&matrix_mul_share.1).unwrap());
        matrix_mul_share_str_vec.push(serde_json::to_string(&matrix_mul_share.2).unwrap());

        //////////////////////// EQUALITY INTEGER ////////////////////////
        let mut equality_integer_shares = shares.equality_integer_shares;


        let mut bagging_matrix_mul_share = shares.bagging_matrix_mul_shares;
        let mut bagging_matrix_mul_share_str_vec = Vec::new();
        bagging_matrix_mul_share_str_vec.push(serde_json::to_string(&bagging_matrix_mul_share.0).unwrap());
        bagging_matrix_mul_share_str_vec.push(serde_json::to_string(&bagging_matrix_mul_share.1).unwrap());
        bagging_matrix_mul_share_str_vec.push(serde_json::to_string(&bagging_matrix_mul_share.2).unwrap());

        let dt_share_message = DecisionTreeTIShareMessage {
            additive_triples: serde_json::to_string(&additive_shares).unwrap(),
            additive_bigint_triples: additive_bigint_str_vec.join(";"),
            binary_triples: binary_share_str_vec.join(";"),
            equality_shares: equality_bigint_str_vec.join(";"),
            rfs_shares: rfs_shares_str_vec.join(";"),
            bagging_shares: bagging_shares_str_vec.join(";"),
            matrix_mul_shares: matrix_mul_share_str_vec.join("&"),
            bagging_matrix_mul_shares: bagging_matrix_mul_share_str_vec.join("&"),
            equality_integer_shares: serde_json::to_string(&equality_integer_shares).unwrap(),
        };
        file.write_all(serde_json::to_string(&dt_share_message).unwrap().as_bytes());
    }

    fn send_dt_shares(mut stream: TcpStream, mut shares: DecisionTreeShares) -> io::Result<()> {
        stream.set_ttl(std::u32::MAX).expect("set_ttl call failed");
        stream.set_write_timeout(None).expect("set_write_timeout call failed");
        stream.set_read_timeout(None).expect("set_read_timeout call failed");
        //////////////////////// SEND ADDITIVES ////////////////////////

        let mut additive_shares = shares.additive_triples;

        //////////////////////// SEND ADDITIVES ////////////////////////

        let mut binary_triples = shares.binary_triples;
        let mut binary_share_str_vec = Vec::new();
        for item in binary_triples.iter() {
            binary_share_str_vec.push(serde_json::to_string(&item).unwrap());
        }


        //////////////////////// ADDITIVE BIGINT ////////////////////////
        let mut additive_bigint_triples = shares.additive_bigint_triples;
        let mut additive_bigint_str_vec = Vec::new();
        for item in additive_bigint_triples.iter() {
            let mut tuples = Vec::new();
//            tuples.push(serde_json::to_string(&(item.0.to_bytes_le())).unwrap());
//            tuples.push(serde_json::to_string(&(item.1.to_bytes_le())).unwrap());
//            tuples.push(serde_json::to_string(&(item.2.to_bytes_le())).unwrap());

            tuples.push(item.0.to_string());
            tuples.push(item.1.to_string());
            tuples.push(item.2.to_string());

            additive_bigint_str_vec.push(tuples.join("&"));
        }

        //////////////////////// EQUALITY BIGINT ////////////////////////
        let mut equality_bigint_triples = shares.equality_shares;
        let mut equality_bigint_str_vec = Vec::new();
        for item in equality_bigint_triples.iter() {
            equality_bigint_str_vec.push(item.to_string());
        }

        //////////////////////// RFS Shares ////////////////////////
        let mut rfs_shares = shares.rfs_shares;
        let mut rfs_shares_str_vec = Vec::new();
        for item in rfs_shares.iter() {
            rfs_shares_str_vec.push(serde_json::to_string(&item).unwrap());
        }

        //////////////////////// Bagging Shares ////////////////////////
        let mut bagging_shares = shares.bagging_shares;
        let mut bagging_shares_str_vec = Vec::new();
        for item in bagging_shares.iter() {
            bagging_shares_str_vec.push(serde_json::to_string(&item).unwrap());
        }

        //////////////////////// Matrix Multiplication Shares ////////////////////////
        let mut matrix_mul_share = shares.matrix_mul_shares;
        let mut matrix_mul_share_str_vec = Vec::new();
        matrix_mul_share_str_vec.push(serde_json::to_string(&matrix_mul_share.0).unwrap());
        matrix_mul_share_str_vec.push(serde_json::to_string(&matrix_mul_share.1).unwrap());
        matrix_mul_share_str_vec.push(serde_json::to_string(&matrix_mul_share.2).unwrap());

        //////////////////////// EQUALITY INTEGER ////////////////////////
        let mut equality_integer_shares = shares.equality_integer_shares;


        let mut bagging_matrix_mul_share = shares.bagging_matrix_mul_shares;
        let mut bagging_matrix_mul_share_str_vec = Vec::new();
        bagging_matrix_mul_share_str_vec.push(serde_json::to_string(&bagging_matrix_mul_share.0).unwrap());
        bagging_matrix_mul_share_str_vec.push(serde_json::to_string(&bagging_matrix_mul_share.1).unwrap());
        bagging_matrix_mul_share_str_vec.push(serde_json::to_string(&bagging_matrix_mul_share.2).unwrap());

        let dt_share_message = DecisionTreeTIShareMessage {
            additive_triples: serde_json::to_string(&additive_shares).unwrap(),
            additive_bigint_triples: additive_bigint_str_vec.join(";"),
            binary_triples: binary_share_str_vec.join(";"),
            equality_shares: equality_bigint_str_vec.join(";"),
            rfs_shares: rfs_shares_str_vec.join(";"),
            bagging_shares: bagging_shares_str_vec.join(";"),
            matrix_mul_shares: matrix_mul_share_str_vec.join("&"),
            bagging_matrix_mul_shares: bagging_matrix_mul_share_str_vec.join("&"),
            equality_integer_shares: serde_json::to_string(&equality_integer_shares).unwrap(),
        };

        let mut message_str = serde_json::to_string(&dt_share_message).unwrap() + "\n";
        stream.write(message_str.as_bytes());
        Ok(())
    }

    fn generate_binary_shares(ctx: &TI, thread_pool: &ThreadPool, amount: usize) -> (Vec<(u8, u8, u8)>, Vec<(u8, u8, u8)>) {
        let mut share0_arc = Arc::new(Mutex::new(HashMap::new()));
        let mut share1_arc = Arc::new(Mutex::new(HashMap::new()));
        let new_amount = if ctx.test_mode {1} else {amount};

        for i in 0..new_amount {
            let mut share0_arc_copy = Arc::clone(&share0_arc);
            let mut share1_arc_copy = Arc::clone(&share1_arc);
            let mut ctx = ctx.clone();

            thread_pool.execute(move || {
                let mut rng = rand::thread_rng();
                let (share0_item, share1_item) = new_binary_shares(&mut rng);
                let mut share0_arc_copy = share0_arc_copy.lock().unwrap();
                (*share0_arc_copy).insert(i, share0_item);

                let mut share1_arc_copy = share1_arc_copy.lock().unwrap();
                (*share1_arc_copy).insert(i, share1_item);
            })
        }
        thread_pool.join();
        let mut share0_map = &(*(share0_arc.lock().unwrap())).clone();
        let mut share1_map = &(*(share1_arc.lock().unwrap())).clone();
        let mut share0 = Vec::new();
        let mut share1 = Vec::new();

        for i in 0..amount {
            let share0_item = share0_map.get(&i).unwrap().clone();
            share0.push((share0_item.0, share0_item.1, share0_item.2));
            let share1_item = share0_map.get(&i).unwrap().clone();
            share1.push((share1_item.0, share1_item.1, share1_item.2));
        }
        (share0, share1)
    }

    fn additive_share_helper(ctx: &TI, thread_pool: &ThreadPool, prime: u64, amount: usize) -> (Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)>, Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)>) {
        let mut share0_arc = Arc::new(Mutex::new(HashMap::new()));
        let mut share1_arc = Arc::new(Mutex::new(HashMap::new()));
        let mut share0 = Vec::new();
        let mut share1 = Vec::new();

        for i in 0..amount {
            let mut share0_arc_copy = Arc::clone(&share0_arc);
            let mut share1_arc_copy = Arc::clone(&share1_arc);
            let mut ctx = ctx.clone();
            thread_pool.execute(move || {
                let mut rng = rand::thread_rng();
                let (share0_item, share1_item) = new_add_triple(&mut rng, prime);
                let mut share0_arc_copy = share0_arc_copy.lock().unwrap();
                (*share0_arc_copy).insert(i, share0_item);

                let mut share1_arc_copy = share1_arc_copy.lock().unwrap();
                (*share1_arc_copy).insert(i, share1_item);
            })
        }
        thread_pool.join();
        let mut share0_map = &(*(share0_arc.lock().unwrap())).clone();
        let mut share1_map = &(*(share1_arc.lock().unwrap())).clone();


        for i in 0..amount {
            let share0_item = share0_map.get(&i).unwrap().clone();
            share0.push((Wrapping(share0_item.0), Wrapping(share0_item.1), Wrapping(share0_item.2)));
            let share1_item = share1_map.get(&i).unwrap().clone();
            share1.push((Wrapping(share1_item.0), Wrapping(share1_item.1), Wrapping(share1_item.2)));
        }
        (share0, share1)
    }

    fn generate_additive_shares(ctx: &TI, thread_pool: &ThreadPool) -> (HashMap<u64, Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)>>, HashMap<u64, Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)>>) {
        let mut result0 = HashMap::new();
        let mut result1 = HashMap::new();
        let amount = if ctx.test_mode {1} else {ctx.add_shares_per_tree};
        let (mut general0, mut general1) = additive_share_helper(ctx, thread_pool, ctx.dataset_size_prime, amount);
//        let (mut feature0, mut feature1) = additive_share_helper(ctx, thread_pool, ctx.rfs_field);
//        let (mut class0, mut class1) = additive_share_helper(ctx, thread_pool, ctx.bagging_field);
        result0.insert(ctx.dataset_size_prime, general0);
        result1.insert(ctx.dataset_size_prime, general1);
//        if ctx.dataset_size_prime == ctx.rfs_field {
//            result0.get_mut(&ctx.dataset_size_prime).unwrap().append(&mut feature0);
//            result1.get_mut(&ctx.dataset_size_prime).unwrap().append(&mut feature1);
//        } else {
//            result0.insert(ctx.rfs_field, feature0);
//            result1.insert(ctx.rfs_field, feature1);
//        }
//        if ctx.dataset_size_prime == ctx.bagging_field {
//            result0.get_mut(&ctx.dataset_size_prime).unwrap().append(&mut class0);
//            result1.get_mut(&ctx.dataset_size_prime).unwrap().append(&mut class1);
//        } else {
//            if ctx.rfs_field == ctx.bagging_field {
//                result0.get_mut(&ctx.rfs_field).unwrap().append(&mut class0);
//                result1.get_mut(&ctx.rfs_field).unwrap().append(&mut class1);
//            } else {
//                result0.insert(ctx.bagging_field, class0);
//                result1.insert(ctx.bagging_field, class1);
//            }
//        }

        (result0, result1)
    }

    fn equality_integer_shares_helper(ctx: &TI, thread_pool: &ThreadPool, prime: u64, amount: usize) -> (Vec<Wrapping<u64>>, Vec<Wrapping<u64>>) {
        let mut share0_arc = Arc::new(Mutex::new(HashMap::new()));
        let mut share1_arc = Arc::new(Mutex::new(HashMap::new()));

        for i in 0..amount {
            let mut share0_arc_copy = Arc::clone(&share0_arc);
            let mut share1_arc_copy = Arc::clone(&share1_arc);
            let mut ctx = ctx.clone();

            thread_pool.execute(move || {
                let mut rng = rand::thread_rng();
                let (share0_item, share1_item) = new_equality_int_shares(&mut rng, prime);
                let mut share0_arc_copy = share0_arc_copy.lock().unwrap();
                (*share0_arc_copy).insert(i, share0_item);

                let mut share1_arc_copy = share1_arc_copy.lock().unwrap();
                (*share1_arc_copy).insert(i, share1_item);
            })
        }
        thread_pool.join();
        let mut share0 = Vec::new();
        let mut share1 = Vec::new();
        let share0_map = &(*(share0_arc.lock().unwrap()));
        let share1_map = &(*(share1_arc.lock().unwrap()));
        for i in 0..amount {
            let mut share0_item = share0_map.get(&i).unwrap().clone();
            share0.push(share0_item);
            let mut share1_item = share1_map.get(&i).unwrap().clone();
            share1.push(share1_item);
        }
        (share0, share1)
    }

    fn generate_equality_integer_shares(ctx: &TI, thread_pool: &ThreadPool) -> (HashMap<u64, Vec<Wrapping<u64>>>, HashMap<u64, Vec<Wrapping<u64>>>) {
        let mut result0 = HashMap::new();
        let mut result1 = HashMap::new();
        let new_amount = if ctx.test_mode {1} else {ctx.ohe_equality_shares as usize};
        let (mut feature_eq0, mut feature_eq1) = equality_integer_shares_helper(ctx, thread_pool, ctx.rfs_field, new_amount);
        let (mut class_eq0, mut class_eq1) = equality_integer_shares_helper(ctx, thread_pool, ctx.bagging_field, new_amount);
        result0.insert(ctx.rfs_field, feature_eq0);
        result1.insert(ctx.rfs_field, feature_eq1);

        if ctx.rfs_field == ctx.bagging_field {
            result0.get_mut(&ctx.rfs_field).unwrap().append(&mut class_eq0);
            result1.get_mut(&ctx.rfs_field).unwrap().append(&mut class_eq1);
        } else {
            result0.insert(ctx.bagging_field, class_eq0);
            result1.insert(ctx.bagging_field, class_eq1);
        }
        (result0, result1)
    }

    fn generate_rfs_share(ctx: &TI) -> (Vec<Vec<Wrapping<u64>>>, Vec<Vec<Wrapping<u64>>>) {
        //generate selection vector
        let mut feature_selected_remain = ctx.feature_selected;
        let mut feature_bit_vec = vec![0u8; ctx.feature_cnt as usize];
        let mut rng = rand::thread_rng();
        let mut vec_to_record = Vec::new();
        while feature_selected_remain != 0 {
            let index: usize = rng.gen_range(0, ctx.feature_cnt as usize);
            if feature_bit_vec[index] == 0 {
                vec_to_record.push(format!("{}", index));
                feature_bit_vec[index] = 1;
                feature_selected_remain -= 1;
            }
        }

        // vec_to_record.sort();
        // let mut file = ctx.fs_selection_file.try_clone().unwrap();
        // file.write_all(format!("{}\n", vec_to_record.join(",")).as_bytes());
//
//        for item in [11, 13, 17, 23, 6].to_vec() {
//            feature_bit_vec[item] = 1;
//            vec_to_record.push(format!("{}", item));
//        }
//        vec_to_record.sort();
//        let mut file = ctx.fs_selection_file.try_clone().unwrap();
//        file.write_all(format!("{}\n", vec_to_record.join(",")).as_bytes());

        let mut share0 = Vec::new();
        let mut share1 = Vec::new();


        for i in 0..feature_bit_vec.len() {
            let item = feature_bit_vec[i];
            if item == 1 {
                for j in 0..ctx.attr_value_cnt {
                    let mut row0 = Vec::new();
                    let mut row1 = Vec::new();
                    for k in 0..ctx.feature_cnt * ctx.attr_value_cnt {
                        let current_item = if k == (i as u64 * ctx.attr_value_cnt + j) { 1 } else { 0 };
                        let share0_item: u64 = rng.gen_range(0, 2);
                        let share1_item: u64 = mod_subtraction(Wrapping(current_item as u64), Wrapping(share0_item), BINARY_PRIME as u64).0;
                        row0.push(Wrapping(share0_item));
                        row1.push(Wrapping(share1_item));
                    }
                    share0.push(row0);
                    share1.push(row1);
                }
            }
        }

        (share0, share1)
    }

    fn generate_bagging_share(ctx: &TI) -> (Vec<Vec<Wrapping<u64>>>, Vec<Vec<Wrapping<u64>>>) {
        //generate selection vector
        let mut instance_selected_remain = ctx.instance_selected;
        let mut instance_selected_vec = vec![0u32; ctx.instance_cnt as usize];
        let mut rng = rand::thread_rng();
        let mut vec_to_record = Vec::new();
        while instance_selected_remain != 0 {
            let index: usize = rng.gen_range(0, ctx.instance_cnt as usize);
            instance_selected_vec[index] += 1;
            vec_to_record.push(format!("{}", index));
            instance_selected_remain -= 1;
        }

        vec_to_record.sort();
        let mut file = ctx.sampling_file.try_clone().unwrap();
        file.write_all(format!("{}\n", vec_to_record.join(",")).as_bytes());

//        for item in [112, 124, 137, 150, 213, 217, 225, 240, 266, 28, 348, 363, 371, 398, 4, 404, 415, 444, 55, 64, 89].to_vec() {
//            instance_selected_vec[item] = 1;
//            vec_to_record.push(format!("{}", item));
//        }
//        vec_to_record.sort();
//        let mut file = ctx.sampling_file.try_clone().unwrap();
//        file.write_all(format!("{}\n", vec_to_record.join(",")).as_bytes());
        let mut share0 = vec![vec![Wrapping(0u64); ctx.instance_selected as usize]; ctx.instance_cnt as usize];
        let mut share1 = vec![vec![Wrapping(0u64); ctx.instance_selected as usize]; ctx.instance_cnt as usize];
        let mut share = vec![vec![Wrapping(0u64); ctx.instance_selected as usize]; ctx.instance_cnt as usize];
        let mut count = 0;
        for i in 0..instance_selected_vec.len() {
            let item = instance_selected_vec[i];
            for k in 0..item {
                for j in 0..ctx.instance_cnt {
                    let current_item = if j as usize == i { 1 } else { 0 };
                    let share0_item: u8 = rng.gen_range(0, BINARY_PRIME as u8);
                    let share1_item: u8 = mod_subtraction_u8(current_item, share0_item, BINARY_PRIME as u8);
                    share0[j as usize][count] = Wrapping(share0_item as u64);
                    share1[j as usize][count] = Wrapping(share1_item as u64);
                    share[j as usize][count] = Wrapping(current_item as u64);
                }
                count += 1;
            }
        }
        (share0, share1)
    }

    fn generate_matrix_shares(ctx: &TI, first_row: usize, second_row: usize, second_col: usize, prime: u64) -> ((Vec<Vec<Wrapping<u64>>>, Vec<Vec<Wrapping<u64>>>, Vec<Vec<Wrapping<u64>>>), (Vec<Vec<Wrapping<u64>>>, Vec<Vec<Wrapping<u64>>>, Vec<Vec<Wrapping<u64>>>)) {
        let mut rng = rand::thread_rng();
        let mut u = Vec::new();
        let mut v = Vec::new();
        let mut w = Vec::new();
        let mut u_share0 = Vec::new();
        let mut v_share0 = Vec::new();
        let mut w_share0 = Vec::new();
        let mut u_share1 = Vec::new();
        let mut v_share1 = Vec::new();
        let mut w_share1 = Vec::new();

        for i in 0..first_row {
            let mut share0_row = Vec::new();
            let mut share1_row = Vec::new();
            let mut row = Vec::new();
            for j in 0..second_row {
                let u_item = rng.gen_range(0, prime);
                //hard_coded for two parties
                let u_item0 = rng.gen_range(0, prime);
                let u_item1 = mod_subtraction(Wrapping(u_item), Wrapping(u_item0), prime).0;
                share0_row.push(Wrapping(u_item0));
                share1_row.push(Wrapping(u_item1));
                row.push(Wrapping(u_item));
            }
            u_share0.push(share0_row);
            u_share1.push(share1_row);
            u.push(row);
        }

        for i in 0..second_row {
            let mut share0_row = Vec::new();
            let mut share1_row = Vec::new();
            let mut row = Vec::new();
            for j in 0..second_col {
                let v_item = rng.gen_range(0, prime);
                //hard_coded for two parties
                let v_item0 = rng.gen_range(0, prime);
                let v_item1 = mod_subtraction(Wrapping(v_item), Wrapping(v_item0), prime).0;
                share0_row.push(Wrapping(v_item0));
                share1_row.push(Wrapping(v_item1));
                row.push(Wrapping(v_item));
            }
            v_share0.push(share0_row);
            v_share1.push(share1_row);
            v.push(row);
        }


        for i in 0..first_row {
            let mut share0_row = Vec::new();
            let mut share1_row = Vec::new();
            let mut row = Vec::new();
            for j in 0..second_col {
                let mut w_item = Wrapping(0);
                for k in 0..second_row {
                    let w_temp = u[i][k] * v[k][j];
                    w_item += w_temp;
                }
                w_item = Wrapping(w_item.0.mod_floor(&prime));
                //hard_coded for two parties
                let w_item0 = rng.gen_range(0, prime);
                let w_item1 = mod_subtraction(w_item, Wrapping(w_item0), prime).0;
                share0_row.push(Wrapping(w_item0));
                share1_row.push(Wrapping(w_item1));
                row.push(w_item);
            }
            w_share0.push(share0_row);
            w_share1.push(share1_row);
            w.push(row);
        }

        ((u_share0, v_share0, w_share0), (u_share1, v_share1, w_share1))
    }

    fn generate_equality_bigint_shares(ctx: &TI, thread_pool: &ThreadPool) -> (Vec<BigUint>, Vec<BigUint>) {
        let mut share0_arc = Arc::new(Mutex::new(HashMap::new()));
        let mut share1_arc = Arc::new(Mutex::new(HashMap::new()));
        let new_amount = if ctx.test_mode {1} else {ctx.equality_shares_per_tree};

        for i in 0..new_amount {
            let mut share0_arc_copy = Arc::clone(&share0_arc);
            let mut share1_arc_copy = Arc::clone(&share1_arc);
            let mut ctx = ctx.clone();

            thread_pool.execute(move || {
                let mut rng = rand::thread_rng();
                let (share0_item, share1_item) = new_equality_bigint_shares(&mut rng, &ctx.big_int_prime, ctx.bigint_bit_size);
                let mut share0_arc_copy = share0_arc_copy.lock().unwrap();
                (*share0_arc_copy).insert(i, share0_item);

                let mut share1_arc_copy = share1_arc_copy.lock().unwrap();
                (*share1_arc_copy).insert(i, share1_item);
            })
        }
        thread_pool.join();
        let mut share0 = Vec::new();
        let mut share1 = Vec::new();
        let share0_map = &(*(share0_arc.lock().unwrap()));
        let share1_map = &(*(share1_arc.lock().unwrap()));
        for i in 0..ctx.equality_shares_per_tree {
            let mut share0_item = share0_map.get(&i).unwrap().clone();
            share0.push(share0_item);
            let mut share1_item = share1_map.get(&i).unwrap().clone();
            share1.push(share1_item);
        }

        (share0, share1)
    }


    fn generate_additive_bigint_shares(ctx: &TI, thread_pool: &ThreadPool) -> (Vec<(BigUint, BigUint, BigUint)>, Vec<(BigUint, BigUint, BigUint)>) {
        let mut share0_arc = Arc::new(Mutex::new(HashMap::new()));
        let mut share1_arc = Arc::new(Mutex::new(HashMap::new()));

        let new_amount = if ctx.test_mode {1} else {ctx.add_shares_bigint_per_tree};

        for i in 0..new_amount {
            let mut share0_arc_copy = Arc::clone(&share0_arc);
            let mut share1_arc_copy = Arc::clone(&share1_arc);
            let mut ctx = ctx.clone();

            thread_pool.execute(move || {
                let mut rng = rand::thread_rng();
                let (share0_item, share1_item) = new_bigint_add_triple(&mut rng, &ctx.big_int_prime, ctx.bigint_bit_size);
                let mut share0_arc_copy = share0_arc_copy.lock().unwrap();
                (*share0_arc_copy).insert(i, share0_item);

                let mut share1_arc_copy = share1_arc_copy.lock().unwrap();
                (*share1_arc_copy).insert(i, share1_item);
            })
        }
        thread_pool.join();
        let share0_map = &(*(share0_arc.lock().unwrap()));
        let share1_map = &(*(share1_arc.lock().unwrap()));
        let mut share0 = Vec::new();
        let mut share1 = Vec::new();
        for i in 0..ctx.add_shares_bigint_per_tree {
            let share0_item = (share0_map.get(&i).unwrap()).clone();
            share0.push((share0_item.0, share0_item.1, share0_item.2));
            let share1_item = (share1_map.get(&i).unwrap()).clone();
            share1.push((share1_item.0, share1_item.1, share1_item.2));
        }
        (share0, share1)
    }

    fn get_confirmation(stream: TcpStream) -> io::Result<()> {
        stream.set_ttl(std::u32::MAX).expect("set_ttl call failed");
        stream.set_write_timeout(None).expect("set_write_timeout call failed");
        stream.set_read_timeout(None).expect("set_read_timeout call failed");

        let mut stream = stream;
        let mut recv_buf = [0u8; 11];
        let mut bytes_read = 0;

        while bytes_read < recv_buf.len() {
            let current_bytes = stream.read(&mut recv_buf[bytes_read..])?;
            bytes_read += current_bytes;
        }

        assert_eq!(b"send shares", &recv_buf);
        //println!("confirmation received");

        let mut bytes_written = 0;
        while bytes_written < recv_buf.len() {
            let current_bytes = stream.write(&recv_buf[bytes_written..]);
            bytes_written += current_bytes.unwrap();
        }

        //println!("confirmation echoed");


        Ok(())
    }


    /* generate group of 64 Beaver triples over Z_2 */
    fn new_xor_triple(rng: &mut rand::ThreadRng) -> ((u64, u64, u64), (u64, u64, u64)) {
        let u: u64 = rng.gen();
        let v: u64 = rng.gen();
        let w = u & v;
        let u0: u64 = rng.gen();
        let v0: u64 = rng.gen();
        let w0: u64 = rng.gen();
        let u1 = u ^ u0;
        let v1 = v ^ v0;
        let w1 = w ^ w0;

        ((u0, v0, w0), (u1, v1, w1))
    }

    /* generate Beaver triples over Z_2^64 */
    fn new_add_triple(rng: &mut rand::ThreadRng, prime: u64) -> ((u64, u64, u64), (u64, u64, u64)) {
        let u: u64 = rng.gen_range(0, prime);
        let v: u64 = rng.gen_range(0, prime);
        let w = mod_floor((Wrapping(u) * Wrapping(v)).0, prime);
        let u0: u64 = rng.gen_range(0, prime);
        let v0: u64 = rng.gen_range(0, prime);
        let w0: u64 = rng.gen_range(0, prime);
        let u1 = mod_subtraction(Wrapping(u), Wrapping(u0), prime).0;
        let v1 = mod_subtraction(Wrapping(v), Wrapping(v0), prime).0;
        let w1 = mod_subtraction(Wrapping(w), Wrapping(w0), prime).0;

        ((u0, v0, w0), (u1, v1, w1))
    }

    fn new_bigint_add_triple(rng: &mut rand::ThreadRng, big_int_prime: &BigUint, bigint_bit_size: usize) -> ((BigUint, BigUint, BigUint), (BigUint, BigUint, BigUint)) {
        let u: BigUint = rng.gen_biguint(bigint_bit_size);
        let v: BigUint = rng.gen_biguint(bigint_bit_size);
        let w = BigUint::mod_floor(&(&u * &v), big_int_prime);
        let u0: BigUint = rng.gen_biguint(bigint_bit_size);
        let v0: BigUint = rng.gen_biguint(bigint_bit_size);
        let w0: BigUint = rng.gen_biguint(bigint_bit_size);
        let u1 = big_uint_subtract(&u, &u0, big_int_prime);
        let v1 = big_uint_subtract(&v, &v0, big_int_prime);
        let w1 = big_uint_subtract(&w, &w0, big_int_prime);

        let computed = u0.clone().add(&u1).mod_floor(&big_int_prime).mul(v0.clone().add(&v1).mod_floor(&big_int_prime)).mod_floor(&big_int_prime);
        let computed_w = w0.clone().add(&w1).mod_floor(&big_int_prime);
        assert_eq!(computed_w, computed);
        assert_eq!(computed_w, w.clone());

        ((u0, v0, w0), (u1, v1, w1))
    }

    fn new_equality_int_shares(rng: &mut rand::ThreadRng, prime: u64) -> (Wrapping<u64>, Wrapping<u64>) {
        let mut r = rng.gen_range(0, prime - 1) + 1;
        let mut rsum = 0;
        //hard-coded for two parties
        let r0 = rng.gen_range(0, prime);
        rsum = rsum + r0;
        let r1 = mod_subtraction(Wrapping(r), Wrapping(rsum), prime).0;
        (Wrapping(r0), Wrapping(r1))
    }

    fn new_equality_bigint_shares(rng: &mut rand::ThreadRng, big_int_prime: &BigUint, bigint_bit_size: usize) -> (BigUint, BigUint) {
        let mut r = rng.gen_biguint(bigint_bit_size).add(BigUint::one());
        let mut rsum = BigUint::zero();
        //hard-coded for two parties
        let r0 = rng.gen_biguint(bigint_bit_size);
        rsum = rsum.add(&r0);
        let r1 = big_uint_subtract(&r, &rsum, big_int_prime);
        (r0, r1)
    }

    fn new_binary_shares(rng: &mut rand::ThreadRng) -> ((u8, u8, u8), (u8, u8, u8)) {
        let u: u8 = rng.gen_range(0, BINARY_PRIME as u8);
        let v: u8 = rng.gen_range(0, BINARY_PRIME as u8);
        let w: u8 = (Wrapping(u) * Wrapping(v)).0;
        let mut usum = 0;
        let mut vsum = 0;
        let mut wsum = 0;
        let u0: u8 = rng.gen_range(0, BINARY_PRIME as u8);
        let v0: u8 = rng.gen_range(0, BINARY_PRIME as u8);
        let w0: u8 = rng.gen_range(0, BINARY_PRIME as u8);
        usum += u0;
        vsum += v0;
        wsum += w0;

        let u1 = mod_subtraction_u8(u, usum, BINARY_PRIME as u8);
        let v1 = mod_subtraction_u8(v, vsum, BINARY_PRIME as u8);
        let w1 = mod_subtraction_u8(w, wsum, BINARY_PRIME as u8);

        ((u0, v0, w0), (u1, v1, w1))
    }
}
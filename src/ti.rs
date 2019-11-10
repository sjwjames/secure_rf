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
    use self::num::{One, Zero, BigInt};
    use std::ops::{Add, Sub};
    use crate::constants::constants::BINARY_PRIME;
    use crate::decision_tree::decision_tree::{DecisionTreeShares, DecisionTreeTIShareMessage};
    use serde::{Serialize, Deserialize, Serializer};
    use std::str::FromStr;
    use threadpool::ThreadPool;
    use std::collections::HashMap;
    use crate::utils::utils::big_uint_subtract;

    pub struct TI {
        pub ti_ip: String,
        pub ti_port0: u16,
        pub ti_port1: u16,
        pub add_shares_per_tree: usize,
        pub add_shares_bigint_per_tree: usize,
        pub equality_shares_per_tree: usize,
        pub binary_shares_per_tree: usize,
        pub tree_count: usize,
        pub batch_size: usize,
        pub tree_training_batch_size: usize,
        pub thread_count: usize,
        pub big_int_prime: BigUint,
        pub prime: u64,
        pub bigint_bit_size: usize,
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
                add_shares_per_tree: self.add_shares_per_tree,
                add_shares_bigint_per_tree: self.add_shares_bigint_per_tree,
                equality_shares_per_tree: self.equality_shares_per_tree,
                binary_shares_per_tree: self.equality_shares_per_tree,
                tree_count: self.tree_count,
                batch_size: self.batch_size,
                tree_training_batch_size: self.tree_training_batch_size,
                big_int_prime: self.big_int_prime.clone(),
                prime: self.prime,
                thread_count: self.thread_count,
                bigint_bit_size: self.bigint_bit_size,
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

        let big_int_prime = match settings.get_str("big_int_prime") {
            Ok(num) => num as String,
            Err(error) => {
                panic!("Encountered a problem while parsing big_int_prime: {:?}", error)
            }
        };


        let big_int_prime = BigUint::from_str(&big_int_prime).unwrap();


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

        TI {
            ti_ip,
            ti_port0,
            ti_port1,
            add_shares_per_tree,
            add_shares_bigint_per_tree,
            equality_shares_per_tree,
            binary_shares_per_tree,
            tree_count,
            batch_size,
            tree_training_batch_size,
            big_int_prime,
            prime,
            thread_count,
            bigint_bit_size,
        }
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

        let mut trees_remaining = ctx.tree_count as isize;
        let mut batch_count = 0;
        let thread_pool = ThreadPool::new(ctx.thread_count);

        while trees_remaining > 0 {
            let current_batch_size = if trees_remaining >= ctx.tree_training_batch_size as isize { ctx.tree_training_batch_size } else { trees_remaining as usize };
            println!("current batch:{}", batch_count);


//                println!("{} [{}] sending correlated randomness...   ", &s0_pfx, i);
//                let shares0 = (add_triples0, xor_triples0);
//                let stream = in_stream0.try_clone().expect("server 0: failed to clone stream");
//                let sender_thread0 = thread::spawn(move || {
//                    match get_confirmation(stream.try_clone()
//                        .expect("server 0: failed to clone stream")) {
//                        Ok(_) => return send_shares(0, stream.try_clone()
//                            .expect("server 0: failed to clone stream"),
//                                                    shares0),
//                        Err(e) => return Err(e),// panic!("server 0: failed to recv confirmation"),
//                    };
//                });
//
//                println!("{} [{}] sending correlated randomness...   ", &s1_pfx, i);
//                let shares1 = (add_triples1, xor_triples1);
//                let stream = in_stream1.try_clone().expect("server 1: failed to clone stream");
//                let sender_thread1 = thread::spawn(move || {
//                    match get_confirmation(stream.try_clone()
//                        .expect("server 1: failed to clone stream")) {
//                        Ok(_) => return send_shares(1, stream.try_clone()
//                            .expect("server 1: failed to clone stream"),
//                                                    shares1),
//                        Err(e) => return Err(e),//::("server 1: failed to recv confirmation"),
//                    };
//                });
//
            for i in 0..current_batch_size {
                print!("{} [{}] generating additive shares...      ", &prefix, i);
                let now = SystemTime::now();
                let (add_triples0, add_triples1) = generate_additive_shares(&ctx, &thread_pool);
                println!("complete -- work time = {:5} (ms)", now.elapsed().unwrap().as_millis());


                print!("{} [{}] generating binary shares...           ", &prefix, i);
                let now = SystemTime::now();
                let (binary_triples0, binary_triples1) = generate_binary_shares(&ctx, &thread_pool);
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
                let mut share0 = DecisionTreeShares {
                    additive_triples: add_triples0,
                    additive_bigint_triples: additive_bigint_share0,
                    binary_triples: binary_triples0,
                    equality_shares: equality_bigint_share0,
                    current_additive_index: Arc::new(Mutex::new(0)),
                    current_additive_bigint_index: Arc::new(Mutex::new(0)),
                    current_equality_index: Arc::new(Mutex::new(0)),
                    current_binary_index: Arc::new(Mutex::new(0)),
                };

                let mut share1 = DecisionTreeShares {
                    additive_triples: add_triples1,
                    additive_bigint_triples: additive_bigint_share1,
                    binary_triples: binary_triples1,
                    equality_shares: equality_bigint_share1,
                    current_additive_index: Arc::new(Mutex::new(0)),
                    current_additive_bigint_index: Arc::new(Mutex::new(0)),
                    current_equality_index: Arc::new(Mutex::new(0)),
                    current_binary_index: Arc::new(Mutex::new(0)),
                };
                let stream = in_stream0.try_clone().expect("server 0: failed to clone stream");
                let sender_thread0 = thread::spawn(move || {
                    match get_confirmation(stream.try_clone()
                        .expect("server 0: failed to clone stream")) {
                        Ok(_) => return send_dt_shares(stream.try_clone()
                                                           .expect("server 0: failed to clone stream"),
                                                       share0),
                        Err(e) => return Err(e),// panic!("server 0: failed to recv confirmation"),
                    };
                });

                let stream = in_stream1.try_clone().expect("server 0: failed to clone stream");
                let sender_thread1 = thread::spawn(move || {
                    match get_confirmation(stream.try_clone()
                        .expect("server 0: failed to clone stream")) {
                        Ok(_) => return send_dt_shares(stream.try_clone()
                                                           .expect("server 0: failed to clone stream"),
                                                       share1),
                        Err(e) => return Err(e),// panic!("server 0: failed to recv confirmation"),
                    };
                });

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


            trees_remaining -= ctx.tree_training_batch_size as isize;
            batch_count += 1;
        }
    }

    fn send_dt_shares(mut stream: TcpStream, mut shares: DecisionTreeShares) -> io::Result<()> {
        stream.set_ttl(std::u32::MAX).expect("set_ttl call failed");
        stream.set_write_timeout(None).expect("set_write_timeout call failed");
        stream.set_read_timeout(None).expect("set_read_timeout call failed");
        //////////////////////// SEND ADDITIVES ////////////////////////

        let mut additive_shares = shares.additive_triples;
        let mut additive_share_str_vec = Vec::new();
        for item in additive_shares.iter() {
            additive_share_str_vec.push(serde_json::to_string(&item).unwrap());
        }

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
            tuples.push(serde_json::to_string(&(item.0.to_bytes_le())).unwrap());
            tuples.push(serde_json::to_string(&(item.1.to_bytes_le())).unwrap());
            tuples.push(serde_json::to_string(&(item.2.to_bytes_le())).unwrap());

            additive_bigint_str_vec.push(format!("({})", tuples.join(",")));
        }

        //////////////////////// EQUALITY BIGINT ////////////////////////
        let mut equality_bigint_triples = shares.equality_shares;
        let mut equality_bigint_str_vec = Vec::new();
        for item in equality_bigint_triples.iter() {
            equality_bigint_str_vec.push(serde_json::to_string(&(item.to_bytes_le())).unwrap());
        }

        let dt_share_message = DecisionTreeTIShareMessage {
            additive_triples: additive_share_str_vec.join(";"),
            additive_bigint_triples: additive_bigint_str_vec.join(";"),
            binary_triples: binary_share_str_vec.join(";"),
            equality_shares: equality_bigint_str_vec.join(";"),
        };

        let mut message_str = serde_json::to_string(&dt_share_message).unwrap() + "\n";
        stream.write(message_str.as_bytes());
        Ok(())
    }

    fn generate_binary_shares(ctx: &TI, thread_pool: &ThreadPool) -> (Vec<(u8, u8, u8)>, Vec<(u8, u8, u8)>) {
        let mut share0_arc = Arc::new(Mutex::new(HashMap::new()));
        let mut share1_arc = Arc::new(Mutex::new(HashMap::new()));

        for i in 0..ctx.binary_shares_per_tree {
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

        for i in 0..ctx.binary_shares_per_tree {
            let share0_item = share0_map.get(&i).unwrap().clone();
            share0.push((share0_item.0, share0_item.1, share0_item.2));
            let share1_item = share0_map.get(&i).unwrap().clone();
            share1.push((share1_item.0, share1_item.1, share1_item.2));
        }
        (share0, share1)
    }

    fn generate_additive_shares(ctx: &TI, thread_pool: &ThreadPool) -> (Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)>, Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)>) {
        let mut share0_arc = Arc::new(Mutex::new(HashMap::new()));
        let mut share1_arc = Arc::new(Mutex::new(HashMap::new()));

        for i in 0..ctx.add_shares_per_tree {
            let mut share0_arc_copy = Arc::clone(&share0_arc);
            let mut share1_arc_copy = Arc::clone(&share1_arc);
            let mut ctx = ctx.clone();
            thread_pool.execute(move || {
                let mut rng = rand::thread_rng();
                let (share0_item, share1_item) = new_add_triple(&mut rng, ctx.prime);
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

        for i in 0..ctx.add_shares_per_tree {
            let share0_item = share0_map.get(&i).unwrap().clone();
            share0.push((Wrapping(share0_item.0), Wrapping(share0_item.1), Wrapping(share0_item.2)));
            let share1_item = share0_map.get(&i).unwrap().clone();
            share1.push((Wrapping(share1_item.0), Wrapping(share1_item.1), Wrapping(share1_item.2)));
        }
        (share0, share1)
    }

    fn generate_equality_bigint_shares(ctx: &TI, thread_pool: &ThreadPool) -> (Vec<BigUint>, Vec<BigUint>) {
        let mut share0_arc = Arc::new(Mutex::new(HashMap::new()));
        let mut share1_arc = Arc::new(Mutex::new(HashMap::new()));

        for i in 0..ctx.equality_shares_per_tree {
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

        for i in 0..ctx.add_shares_bigint_per_tree {
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
        let u1 = mod_floor((Wrapping(u) - Wrapping(u0)).0, prime);
        let v1 = mod_floor((Wrapping(v) - Wrapping(v0)).0, prime);
        let w1 = mod_floor((Wrapping(w) - Wrapping(w0)).0, prime);

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
        ((u0, v0, w0), (u1, v1, w1))
    }

    fn new_equality_bigint_shares(rng: &mut rand::ThreadRng, big_int_prime: &BigUint, bigint_bit_size: usize) -> (BigUint, BigUint) {
        let mut r = rng.gen_biguint(bigint_bit_size);
        r = r + BigUint::one();
        let mut rsum = BigUint::zero();
        let r0 = rng.gen_biguint(bigint_bit_size);
        rsum = rsum + &r0;
        let r1 = BigInt::mod_floor(&(r.to_bigint().unwrap() - rsum.to_bigint().unwrap()), &(big_int_prime.to_bigint().unwrap())).to_biguint().unwrap();
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

        let u1 = mod_floor((Wrapping(u) - Wrapping(usum)).0, BINARY_PRIME as u8);
        let v1 = mod_floor((Wrapping(v) - Wrapping(vsum)).0, BINARY_PRIME as u8);
        let w1 = mod_floor((Wrapping(w) - Wrapping(wsum)).0, BINARY_PRIME as u8);

        ((u0, v0, w0), (u1, v1, w1))
    }
}
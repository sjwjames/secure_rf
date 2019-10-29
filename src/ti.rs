pub mod ti {
    //author Davis, email:daviscrailsback@gmail.com
    extern crate rand;
    extern crate num;

    use rand::Rng;
    use std::time::SystemTime;
    use std::thread;
    use std::net::{TcpStream, TcpListener, SocketAddr};
    use std::io::{Read, Write};
    use std::num::Wrapping;
    use std::io;
    use crate::constants::constants;
    use crate::thread_pool::thread_pool::ThreadPool;
    use std::sync::{Arc, Mutex};
    use num::bigint::{BigUint, ToBigUint, RandBigInt};
    use num::integer::*;
    use self::num::{One, Zero};
    use std::ops::Add;
    use crate::constants::constants::BINARY_PRIME;

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
        pub big_int_prime: BigUint,
        pub prime: u64,
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
                big_int_prime: self.big_int_prime.clone(),
                prime: self.prime,
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

        let big_int_prime = match settings.get_int("big_int_prime") {
            Ok(num) => num as u128,
            Err(error) => {
                panic!("Encountered a problem while parsing big_int_prime: {:?}", error)
            }
        };


        let big_int_prime = big_int_prime.to_biguint().unwrap();


        let prime = match settings.get_int("prime") {
            Ok(num) => num as u64,
            Err(error) => {
                panic!("Encountered a problem while parsing prime: {:?}", error)
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
            big_int_prime,
            prime,
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

        let mut trees_remaining = ctx.tree_count;
        let mut batch_count = 0;
        let thread_pool = ThreadPool::new(ctx.batch_size);
        while trees_remaining > 0 {
            let current_batch = if trees_remaining >= ctx.batch_size { ctx.batch_size } else { trees_remaining };
            println!("current batch:{}", batch_count);

            for i in 0..current_batch {
                println!("{} [{}] generating additive shares...      ", &prefix, i);
                let now = SystemTime::now();
                let (add_triples0, add_triples1) = generate_triples(&ctx, true);
                println!("{} [{}] additive shares                    complete -- work time = {:5} (ms)",
                         &prefix, i, now.elapsed().unwrap().as_millis());


                print!("{} [{}] generating additive big int shares...           ", &prefix, i);
                let now = SystemTime::now();
                let (xor_triples0, xor_triples1) = new_bigint_add_triple(&ctx);
                println!("complete -- work time = {:5} (ms)",
                         now.elapsed().unwrap().as_millis());


                println!("{} [{}] sending correlated randomness...   ", &s0_pfx, i);
                let shares0 = (add_triples0, xor_triples0);
                let stream = in_stream0.try_clone().expect("server 0: failed to clone stream");
                let sender_thread0 = thread::spawn(move || {
                    match get_confirmation(stream.try_clone()
                        .expect("server 0: failed to clone stream")) {
                        Ok(_) => return send_shares(0, stream.try_clone()
                            .expect("server 0: failed to clone stream"),
                                                    shares0),
                        Err(e) => return Err(e),// panic!("server 0: failed to recv confirmation"),
                    };
                });

                println!("{} [{}] sending correlated randomness...   ", &s1_pfx, i);
                let shares1 = (add_triples1, xor_triples1);
                let stream = in_stream1.try_clone().expect("server 1: failed to clone stream");
                let sender_thread1 = thread::spawn(move || {
                    match get_confirmation(stream.try_clone()
                        .expect("server 1: failed to clone stream")) {
                        Ok(_) => return send_shares(1, stream.try_clone()
                            .expect("server 1: failed to clone stream"),
                                                    shares1),
                        Err(e) => return Err(e),//::("server 1: failed to recv confirmation"),
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

            trees_remaining -= ctx.batch_size;
            batch_count += 1;
        }
    }

    fn send_shares(handle: usize,
                   mut stream: TcpStream,
                   mut shares: (Vec<(u64, u64, u64)>, Vec<(u64, u64, u64)>)) -> io::Result<()> {
        stream.set_ttl(std::u32::MAX).expect("set_ttl call failed");
        stream.set_write_timeout(None).expect("set_write_timeout call failed");
        stream.set_read_timeout(None).expect("set_read_timeout call failed");

        //println!("server {}: sending additive shares", handle);

        //////////////////////// SEND ADDITIVES ////////////////////////

        let mut remainder = shares.0.len();

        while remainder >= TI_BATCH_SIZE {
            let mut tx_buf = Xbuffer { u64_buf: [0u64; U64S_PER_TX] };

            for i in 0..TI_BATCH_SIZE {
                let (u, v, w) = shares.0.pop().unwrap();

                unsafe {
                    tx_buf.u64_buf[3 * i] = u;
                    tx_buf.u64_buf[3 * i + 1] = v;
                    tx_buf.u64_buf[3 * i + 2] = w;
                }
            }
            let mut bytes_written = 0;
            while bytes_written < U8S_PER_TX {
                let current_bytes = unsafe {
                    stream.write(&tx_buf.u8_buf[bytes_written..])
                };
                bytes_written += current_bytes.unwrap();
            }
            remainder -= TI_BATCH_SIZE;
        }

        let mut tx_buf = Xbuffer { u64_buf: [0u64; U64S_PER_TX] };

        for i in 0..remainder {
            let (u, v, w) = shares.0.pop().unwrap();

            unsafe {
                tx_buf.u64_buf[3 * i] = u;
                tx_buf.u64_buf[3 * i + 1] = v;
                tx_buf.u64_buf[3 * i + 2] = w;
            }
        }
        let mut bytes_written = 0;
        while bytes_written < U8S_PER_TX {
            let current_bytes = unsafe {
                stream.write(&tx_buf.u8_buf[bytes_written..])
            };
            bytes_written += current_bytes.unwrap();
        }
        //println!("server {}: additive shares sent. sending xor shares", handle);

        /////////////////////////// SEND XOR SHARES //////////////////////////

        let mut remainder = shares.1.len();
        while remainder >= TI_BATCH_SIZE {
            let mut tx_buf = Xbuffer { u64_buf: [0u64; U64S_PER_TX] };

            for i in 0..TI_BATCH_SIZE {
                let (u, v, w) = shares.1.pop().unwrap();

                unsafe {
                    tx_buf.u64_buf[3 * i] = u;
                    tx_buf.u64_buf[3 * i + 1] = v;
                    tx_buf.u64_buf[3 * i + 2] = w;
                }
            }
            let mut bytes_written = 0;
            while bytes_written < U8S_PER_TX {
                let current_bytes = unsafe {
                    stream.write(&tx_buf.u8_buf[bytes_written..])
                };
                bytes_written += current_bytes.unwrap();
            }
            remainder -= TI_BATCH_SIZE;
        }

        let mut tx_buf = Xbuffer { u64_buf: [0u64; U64S_PER_TX] };

        for i in 0..remainder {
            let (u, v, w) = shares.1.pop().unwrap();

            unsafe {
                tx_buf.u64_buf[3 * i] = u;
                tx_buf.u64_buf[3 * i + 1] = v;
                tx_buf.u64_buf[3 * i + 2] = w;
            }
        }
        let mut bytes_written = 0;

        while bytes_written < U8S_PER_TX {
            let current_bytes = unsafe {
                stream.write(&tx_buf.u8_buf[bytes_written..])
            };
            bytes_written += current_bytes.unwrap();
        }


        //println!("server {}: xor shares sent", handle);

        Ok(())
    }

    fn generate_triples(ctx: &TI, additive: bool) -> (Vec<(u64, u64, u64)>, Vec<(u64, u64, u64)>) {
        let triple_count = if additive { ctx.add_shares_per_iter } else { ctx.xor_shares_per_iter };

        let shares =

            if additive {
                let triple_count0 = triple_count;
                let first_half = thread::spawn(move || {
                    let mut rng = rand::thread_rng();
                    let mut shares = (Vec::new(), Vec::new());
                    let to_idx = triple_count0 / 2;

                    println!(" |--> worker thread 0: generating {} additive shares", to_idx);
                    let now = SystemTime::now();
                    for i in 0..to_idx {
                        let (p0_share, p1_share) = new_add_triple(&mut rng);
                        shares.0.push(p0_share);
                        shares.1.push(p1_share);
                    }
                    println!(" |--> worker thread 0: complete -- work time {} (ms)",
                             now.elapsed().unwrap().as_millis());

                    shares
                });

                let triple_count1 = triple_count;
                let second_half = thread::spawn(move || {
                    let mut rng = rand::thread_rng();
                    let mut shares = (Vec::new(), Vec::new());
                    let to_idx = triple_count1 / 2 + (triple_count1 % 2);

                    println!(" |--> worker thread 1: generating {} additive shares", to_idx);
                    let now = SystemTime::now();
                    for i in 0..to_idx {
                        let (p0_share, p1_share) = new_add_triple(&mut rng);
                        shares.0.push(p0_share);
                        shares.1.push(p1_share);
                    }
                    println!(" |--> worker thread 1: complete -- work time {} (ms)",
                             now.elapsed().unwrap().as_millis());

                    shares
                });

                let mut first = first_half.join().unwrap();
                let mut second = second_half.join().unwrap();

                first.0.append(&mut second.0);
                first.1.append(&mut second.1);

                first
            } else {
                let mut rng = rand::thread_rng();
                let mut shares = (Vec::new(), Vec::new());

                for i in 0..triple_count {
                    let (p0_share, p1_share) = new_xor_triple(&mut rng);

                    shares.0.push(p0_share);
                    shares.1.push(p1_share);
                }
                shares
            };

        shares
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
    fn new_add_triple(rng: &mut rand::ThreadRng) -> ((u64, u64, u64), (u64, u64, u64)) {
        let u: u64 = rng.gen();
        let v: u64 = rng.gen();
        let w = (Wrapping(u) * Wrapping(v)).0;
        let u0: u64 = rng.gen();
        let v0: u64 = rng.gen();
        let w0: u64 = rng.gen();
        let u1 = (Wrapping(u) - Wrapping(u0)).0;
        let v1 = (Wrapping(v) - Wrapping(v0)).0;
        let w1 = (Wrapping(w) - Wrapping(w0)).0;

        ((u0, v0, w0), (u1, v1, w1))
    }

    fn new_bigint_add_triple(rng: &mut rand::ThreadRng, big_int_prime: &BigUint, bigint_bit_size: usize) -> ((BigUint, BigUint, BigUint), (BigUint, BigUint, BigUint)) {
        let u: BigUint = rng.gen_biguint(bigint_bit_size);
        let v: BigUint = rng.gen_biguint(bigint_bit_size);
        let w = BigUint::mod_floor(&(&u * &v), big_int_prime);
        let u0: BigUint = rng.gen_biguint(bigint_bit_size);
        let v0: BigUint = rng.gen_biguint(bigint_bit_size);
        let w0: BigUint = rng.gen_biguint(bigint_bit_size);
        let u1 = (u - &u0).mod_floor(big_int_prime);
        let v1 = (v - &v0).mod_floor(big_int_prime);
        let w1 = (w - &v0).mod_floor(big_int_prime);
        ((u0, v0, w0), (u1, v1, w1))
    }

    fn new_equality_bigint_shares(rng: &mut rand::ThreadRng, big_int_prime: &BigUint, bigint_bit_size: usize) -> (BigUint, BigUint) {
        let r = rng.gen_biguint(bigint_bit_size) + BigUint::one();
        let mut rsum = BigUint::zero();
        let r0 = rng.gen_biguint(bigint_bit_size);
        rsum = rsum + &r0;
        let r1 = BigUint::mod_floor(&(&r - &rsum), big_int_prime);
        (r0, r1)
    }

    fn new_binary_shares(rng: &mut rand::ThreadRng) -> ((u8, u8, u8), (u8, u8, u8)) {
        let u: u8 = rng.gen(BINARY_PRIME);
        let v: u8 = rng.gen(BINARY_PRIME);
        let w: u8 = (Wrapping(u) * Wrapping(v)).0;
        let mut usum = 0;
        let mut vsum = 0;
        let mut wsum = 0;
        let u0: u8 = rng.gen(BINARY_PRIME);
        let v0: u8 = rng.gen(BINARY_PRIME);
        let w0: u8 = rng.gen(BINARY_PRIME);
        usum += u0;
        vsum += v0;
        wsum += w0;

        let u1 = mod_floor(u - usum, BINARY_PRIME as u8);
        let v1 = mod_floor(v - vsum, BINARY_PRIME as u8);
        let w1 = mod_floor(w - wsum, BINARY_PRIME as u8);

        ((u0, v0, w0), (u1, v1, w1))
    }
}
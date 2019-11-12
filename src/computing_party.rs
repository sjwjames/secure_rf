pub mod computing_party {
    extern crate num;

    use std::num::Wrapping;
    use std::net::{TcpStream, SocketAddr, TcpListener};
    use std::fs::File;
    use std::string::ToString;
    use std::io::{Write, Read, BufReader, BufRead};
    use crate::constants::constants::{TI_BATCH_SIZE, U64S_PER_TX, U8S_PER_TX};
    use crate::decision_tree::decision_tree::{DecisionTreeData, DecisionTreeTraining, DecisionTreeShares, DecisionTreeTIShareMessage};
    use num::bigint::{BigUint, BigInt, ToBigUint, ToBigInt};
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};
    use crate::message::message::MessageManager;
    use std::collections::HashMap;

    union Xbuffer {
        u64_buf: [u64; U64S_PER_TX],
        u8_buf: [u8; U8S_PER_TX],
    }

    //author Davis, email:daviscrailsback@gmail.com
    pub struct ComputingParty {
        /* options */
        pub debug_output: bool,
        pub decimal_precision: u32,

        /* network */
        pub party_id: u8,
        pub ti_ip: String,
        pub ti_port0: u16,
        pub ti_port1: u16,
        pub party0_ip: String,
        pub party0_port: u16,
        pub party1_ip: String,
        pub party1_port: u16,
        pub in_stream: TcpStream,
        pub o_stream: TcpStream,
        pub ti_stream: TcpStream,

        /* mpc */
        pub asymmetric_bit: u8,

        /* input output */
        pub output_path: String,

        /* DT training Data*/
        pub dt_data: DecisionTreeData,

        /* DT training*/
        pub dt_training: DecisionTreeTraining,
        pub dt_shares: DecisionTreeShares,

        /* random forest */
        pub thread_count: usize,
        pub tree_count: usize,
        pub batch_size: usize,
        pub tree_training_batch_size: usize,

        //multi_thread
        pub thread_hierarchy:Vec<String>,
        pub message_manager:MessageManager
    }

    impl Clone for ComputingParty {
        fn clone(&self) -> Self {
            ComputingParty {
                debug_output: self.debug_output,
                decimal_precision: self.decimal_precision,
                party_id: self.party_id,
                ti_ip: self.ti_ip.clone(),
                ti_port0: self.ti_port0,
                ti_port1: self.ti_port1,
                party0_ip: self.party0_ip.clone(),
                party0_port: self.party0_port,
                party1_ip: self.party1_ip.clone(),
                party1_port: self.party1_port,
                in_stream: self.in_stream.try_clone().expect("failed to clone in_stream"),
                o_stream: self.o_stream.try_clone().expect("failed to clone o_stream"),
                ti_stream: self.ti_stream.try_clone().expect("failed to clone ti_stream"),
                asymmetric_bit: self.asymmetric_bit,
                output_path: self.output_path.clone(),

                dt_data: self.dt_data.clone(),
                dt_training: self.dt_training.clone(),
                dt_shares: self.dt_shares.clone(),

                thread_count: self.thread_count,
                tree_count: self.tree_count,
                batch_size: self.batch_size,

                tree_training_batch_size: self.tree_training_batch_size,
                thread_hierarchy: self.thread_hierarchy.clone(),
                message_manager: self.message_manager.clone()
            }
        }
    }

    fn load_dt_training_file(file_path: &String) -> (usize, usize, usize, usize, Vec<Vec<u8>>) {
        let mut one_hot_encoding: Vec<Vec<u8>> = Vec::new();
        let file = File::open(file_path).expect("input file not found");
        let reader = BufReader::new(file);
        let mut count = 0;
        let mut class_value_count = 0;
        let mut attr_value_count = 0;
        let mut attr_count = 0;
        let mut instance_count = 0;
        for line in reader.lines() {
            let line = line.unwrap();
            match count {
                0 => {
                    class_value_count = line.parse().unwrap();
                }
                1 => {
                    attr_count = line.parse().unwrap();
                }
                2 => {
                    attr_value_count = line.parse().unwrap();
                }
                3 => {
                    instance_count = line.parse().unwrap();
                }
                _ => {
                    let item: Vec<&str> = line.split(",").collect();
                    let item = item.into_iter().map(|x| { x.parse().unwrap() }).collect();
                    one_hot_encoding.push(item);
                }
            }
            count += 1;
        }

        (class_value_count, attr_count, attr_value_count, instance_count, one_hot_encoding)
    }

    fn load_u64_matrix(file_path: &String, instances: usize, add_dummy: bool, one: u64) -> Vec<Vec<Wrapping<u64>>> {
        let mut matrix: Vec<Vec<Wrapping<u64>>> = vec![Vec::new(); instances];

        let file = File::open(file_path);
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(file.unwrap());

        let mut index = 0;
        for result in rdr.deserialize() {
            if index == instances {
                break;
            }

            matrix[index] = result.unwrap();
            if add_dummy {
                matrix[index].push(Wrapping(one)); // TODO: Ensure this is enough
            }
            index += 1;
        }
        matrix
    }


    fn try_connect(socket: &SocketAddr, prefix: &str) -> TcpStream {
        loop {
            match TcpStream::connect(socket) {
                Ok(stream) => return stream,
                Err(_) => println!("{} connection refused by {}", prefix, socket),
            };
        }
    }

    fn produce_dt_data(one_hot_encoding_data: Vec<Vec<u8>>, class_value_count: usize, attr_value_count: usize, attribute_count: usize, instance_count: usize, asymmetric_bit: u8) -> DecisionTreeData {
        let mut attr_values_bytes = Vec::new();
        let mut class_values_bytes = Vec::new();
        for i in 0..attribute_count {
            let mut attr_data = Vec::new();
            for j in 0..attr_value_count {
                let item_copied = one_hot_encoding_data[i * attr_value_count + j].clone();
                attr_data.push(item_copied);
            }
            attr_values_bytes.push(attr_data);
        }

        for i in 0..class_value_count {
            let item_copied = one_hot_encoding_data[attr_value_count * attribute_count + i].clone();
            class_values_bytes.push(item_copied);
        }

        DecisionTreeData {
            attr_value_count,
            class_value_count,
            attribute_count,
            instance_count,
            attr_values: vec![],
            class_values: vec![],
            attr_values_bytes,
            class_values_bytes,
            attr_values_big_integer: vec![],
            class_values_big_integer: vec![],
        }
    }


    pub fn initialize_party_context(settings_file: String) -> ComputingParty {
        let mut settings = config::Config::default();
        settings
            .merge(config::File::with_name(&settings_file.as_str())).unwrap()
            .merge(config::Environment::with_prefix("APP")).unwrap();


        let debug_output = match settings.get_bool("debug_output") {
            Ok(num) => num as bool,
            Err(error) => {
                panic!("Encountered a problem while parsing debug_output: {:?}", error)
            }
        };

        let party_id = match settings.get_int("party_id") {
            Ok(num) => num as u8,
            Err(error) => {
                panic!("Encountered a problem while parsing party_id: {:?}", error)
            }
        };

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

        let party0_ip = match settings.get_str("party0_ip") {
            Ok(num) => num as String,
            Err(error) => {
                panic!("Encountered a problem while parsing party0_ip: {:?} ", error)
            }
        };

        let party1_ip = match settings.get_str("party1_ip") {
            Ok(num) => num as String,
            Err(error) => {
                panic!("Encountered a problem while parsing party1_ip: {:?} ", error)
            }
        };

        let party0_port = match settings.get_int("party0_port") {
            Ok(num) => num as u16,
            Err(error) => {
                panic!("Encountered a problem while parsing party0_port: {:?} ", error)
            }
        };

        let party1_port = match settings.get_int("party1_port") {
            Ok(num) => num as u16,
            Err(error) => {
                panic!("Encountered a problem while parsing party1_port: {:?} ", error)
            }
        };

        let x_input_path = match settings.get_str("x_input_path") {
            Ok(string) => string,
            Err(error) => {
                panic!("Encountered a problem while parsing x_input_path: {:?}", error)
            }
        };

        let y_input_path = match settings.get_str("y_input_path") {
            Ok(string) => string,
            Err(error) => {
                panic!("Encountered a problem while parsing y_input_path: {:?}", error)
            }
        };

        let output_path = match settings.get_str("output_path") {
            Ok(string) => string,
            Err(error) => {
                panic!("Encountered a problem while parsring weights_output_path: {:?}", error)
            }
        };
        let decimal_precision = match settings.get_int("decimal_precision") {
            Ok(num) => num as u32,
            Err(error) => {
                panic!("Encountered a problem while parsing instance: {:?}", error)
            }
        };

        let integer_precision = match settings.get_int("integer_precision") {
            Ok(num) => num as u32,
            Err(error) => {
                panic!("Encountered a problem while parsing instance: {:?}", error)
            }
        };

        let thread_count = match settings.get_int("thread_count") {
            Ok(num) => num as usize,
            Err(error) => {
                panic!("Encountered a problem while parsing thread_count: {:?}", error)
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


        let max_depth = match settings.get_int("max_depth") {
            Ok(num) => num as usize,
            Err(error) => {
                panic!("Encountered a problem while parsing max_depth: {:?}", error)
            }
        };

        let alpha = match settings.get_int("alpha") {
            Ok(num) => num as usize,
            Err(error) => {
                panic!("Encountered a problem while parsing alpha: {:?}", error)
            }
        };
        let alpha = alpha.to_bigint().unwrap();


        let epsilon = match settings.get_float("epsilon") {
            Ok(num) => num as f64,
            Err(error) => {
                panic!("Encountered a problem while parsing epsilon: {:?}", error)
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


        let bit_length = match settings.get_int("bit_length") {
            Ok(num) => num as u64,
            Err(error) => {
                panic!("Encountered a problem while parsing bit_length: {:?}", error)
            }
        };


        let (class_value_count, attribute_count, attr_value_count, instance_count, one_hot_encoding_matrix) = load_dt_training_file(&x_input_path);

        let dataset_size_prime = (instance_count as f64).log2().ceil() as u64;

        let dataset_size_bit_length = (dataset_size_prime as f64).powf(2.0) as u64;

        let mut internal_addr; //= String::new();
        let mut external_addr; //= String::new();
        let mut ti_addr;       //= String::new();

        if party_id == 0 {
            internal_addr = format!("{}:{}", &party0_ip, party0_port);
            external_addr = format!("{}:{}", &party1_ip, party1_port);
            ti_addr = format!("{}:{}", &ti_ip, ti_port0);
        } else {
            internal_addr = format!("{}:{}", &party1_ip, party1_port);
            external_addr = format!("{}:{}", &party0_ip, party0_port);
            ti_addr = format!("{}:{}", &ti_ip, ti_port1);
        }

        let server_socket: SocketAddr = internal_addr
            .parse()
            .expect("unable to parse internal socket address");

        let client_socket: SocketAddr = external_addr
            .parse()
            .expect("unable to parse external socket address");

        let listener = TcpListener::bind(&server_socket)
            .expect("unable to establish Tcp Listener");


        let s_pfx = "server:    ";
        let c_pfx = "client:    ";
        let t_pfx = "ti client: ";

        println!("{} listening on port {}", &s_pfx, &internal_addr);

        let o_stream = try_connect(&client_socket, &c_pfx);

        println!("{} successfully connected to server on port {}",
                 &c_pfx, &external_addr);

        let in_stream = match listener.accept() {
            Ok((stream, _addr)) => stream,
            Err(_) => panic!("failed to accept connection"),
        };

        // TI connection
        let ti_socket: SocketAddr = ti_addr
            .parse()
            .expect("unable to parse ti socket address");

        let ti_stream = try_connect(&ti_socket, &t_pfx);

        println!("{} successfully connected to ti server on port {}",
                 &t_pfx, &external_addr);

        o_stream.set_ttl(std::u32::MAX).expect("set_ttl call failed");
        o_stream.set_write_timeout(None).expect("set_write_timeout call failed");
        o_stream.set_read_timeout(None).expect("set_read_timeout call failed");

        in_stream.set_ttl(std::u32::MAX).expect("set_ttl call failed");
        in_stream.set_write_timeout(None).expect("set_write_timeout call failed");
        in_stream.set_read_timeout(None).expect("set_read_timeout call failed");

        ti_stream.set_ttl(std::u32::MAX).expect("set_ttl call failed");
        ti_stream.set_write_timeout(None).expect("set_write_timeout call failed");
        ti_stream.set_read_timeout(None).expect("set_read_timeout call failed");


        let dt_data = produce_dt_data(one_hot_encoding_matrix, class_value_count, attr_value_count, attribute_count, instance_count, party_id);


        let subset_transaction_bit_vector = vec![party_id as u8; instance_count];
        let cutoff_transaction_set_size = (epsilon * instance_count as f64) as usize;
        let attribute_bit_vector = vec![1u8; attribute_count];
        let dt_training = DecisionTreeTraining {
            max_depth,
            alpha,
            epsilon,
            cutoff_transaction_set_size,
            subset_transaction_bit_vector,
            attribute_bit_vector,
            prime,
            big_int_prime,
            dataset_size_prime,
            dataset_size_bit_length,
            bit_length,
            big_int_ti_index: 0,
        };

        ComputingParty {
            debug_output,
            decimal_precision,
            party_id,
            ti_ip,
            ti_port0,
            ti_port1,
            party0_ip,
            party0_port,
            party1_ip,
            party1_port,
            asymmetric_bit: party_id,
            output_path,
            ti_stream,
            in_stream,
            o_stream,
            thread_count,
            tree_count,
            batch_size,
            tree_training_batch_size,
            dt_data,
            dt_training,
            dt_shares: DecisionTreeShares {
                additive_triples: vec![],
                additive_bigint_triples: vec![],
                binary_triples: vec![],
                equality_shares: vec![],
                current_additive_index: Arc::new(Mutex::new(0 as usize)),
                current_additive_bigint_index: Arc::new(Mutex::new(0 as usize)),
                current_equality_index: Arc::new(Mutex::new(0 as usize)),
                current_binary_index: Arc::new(Mutex::new(0 as usize)),
            },
            thread_hierarchy:vec![format!("{}","main")],
            message_manager:MessageManager{
                map: HashMap::new()
            }
        }
    }

    pub fn get_formatted_address(party_id: u8, party0_ip: &str, party0_port: u16, party1_ip: &str, party1_port: u16) -> (String, String) {
        let mut internal_addr; //= String::new();
        let mut external_addr; //= String::new();

        if party_id == 0 {
            internal_addr = format!("{}:{}", &party0_ip, party0_port);
            external_addr = format!("{}:{}", &party1_ip, party1_port);
        } else {
            internal_addr = format!("{}:{}", &party1_ip, party1_port);
            external_addr = format!("{}:{}", &party0_ip, party0_port);
        }

        (internal_addr, external_addr)
    }

    pub fn try_setup_socket(internal_addr: &str, external_addr: &str) -> (TcpStream, TcpStream) {
        let server_socket: SocketAddr = internal_addr
            .parse()
            .expect("unable to parse internal socket address");

        let client_socket: SocketAddr = external_addr
            .parse()
            .expect("unable to parse external socket address");

        let listener = TcpListener::bind(&server_socket)
            .expect("unable to establish Tcp Listener");


        let s_pfx = "server:    ";
        let c_pfx = "client:    ";

        println!("{} listening on port {}", &s_pfx, &internal_addr);

        let o_stream = try_connect(&client_socket, &c_pfx);

        println!("{} successfully connected to server on port {}",
                 &c_pfx, &external_addr);

        let in_stream = match listener.accept() {
            Ok((stream, _addr)) => stream,
            Err(_) => panic!("failed to accept connection"),
        };

        o_stream.set_ttl(std::u32::MAX).expect("set_ttl call failed");
        o_stream.set_write_timeout(None).expect("set_write_timeout call failed");
        o_stream.set_read_timeout(None).expect("set_read_timeout call failed");

        in_stream.set_ttl(std::u32::MAX).expect("set_ttl call failed");
        in_stream.set_write_timeout(None).expect("set_write_timeout call failed");
        in_stream.set_read_timeout(None).expect("set_read_timeout call failed");

        (in_stream, o_stream)
    }

    pub fn ti_receive(mut stream: TcpStream) -> DecisionTreeShares {
        stream.set_ttl(std::u32::MAX).expect("set_ttl call failed");
        stream.set_write_timeout(None).expect("set_write_timeout call failed");
        stream.set_read_timeout(None).expect("set_read_timeout call failed");


//        let mut xor_shares: Vec<(u64, u64, u64)> = Vec::new();
//        let mut add_shares: Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)> = Vec::new();
//
        println!("sending ready msg to ti");

        let mut recv_buf = [0u8; 11];
        let msg = b"send shares";
        let mut bytes_written = 0;
        while bytes_written < msg.len() {
            let current_bytes = stream.write(&msg[bytes_written..]);
            bytes_written += current_bytes.unwrap();
        }
        println!("ready msg sent. awaiting confimation");
//
//
        let mut bytes_read = 0;
        while bytes_read < recv_buf.len() {
            let current_bytes = stream.read(&mut recv_buf[bytes_read..]).unwrap();
            bytes_read += current_bytes;
        }

        assert_eq!(msg, &recv_buf);

        let mut reader = BufReader::new(stream);
        let mut share_message = String::new();
        reader.read_line(&mut share_message).expect("fail to read share message str");
        let ti_shares_message: DecisionTreeTIShareMessage = serde_json::from_str(&share_message).unwrap();
        let additive_triple_str_vec: Vec<&str> = ti_shares_message.additive_triples.split(";").collect();
        let mut additive_triples = Vec::new();
        for item in additive_triple_str_vec {
            additive_triples.push(serde_json::from_str(item).unwrap());
        }

        let mut additive_bigint_triples = Vec::new();
        let additive_bigint_triple_str_vec: Vec<&str> = ti_shares_message.additive_bigint_triples.split(";").collect();
        for item in additive_bigint_triple_str_vec {
            let temp_str = &item[1..item.len()];
            let str_vec: Vec<&str> = temp_str.split(",").collect();
            additive_bigint_triples.push(
                (
                    BigUint::from_bytes_le(str_vec[0].as_bytes()),
                    BigUint::from_bytes_le(str_vec[1].as_bytes()),
                    BigUint::from_bytes_le(str_vec[2].as_bytes())
                )
            );
        }

        let mut binary_triples = Vec::new();
        let binary_triples_vec: Vec<&str> = ti_shares_message.binary_triples.split(";").collect();
        for item in binary_triples_vec {
            binary_triples.push(serde_json::from_str(item).unwrap());
        }

        let mut equality_shares = Vec::new();
        let equality_share_vec: Vec<&str> = ti_shares_message.equality_shares.split(";").collect();
        for item in equality_share_vec {
            equality_shares.push(
                BigUint::from_bytes_le(item.as_bytes())
            );
        }

        DecisionTreeShares {
            additive_triples,
            additive_bigint_triples,
            binary_triples,
            equality_shares,
            current_additive_index: Arc::new(Mutex::new(0 as usize)),
            current_additive_bigint_index: Arc::new(Mutex::new(0 as usize)),
            current_equality_index: Arc::new(Mutex::new(0 as usize)),
            current_binary_index: Arc::new(Mutex::new(0 as usize)),
        }
//
//
//        //println!("got confirmation ; recving additive shares");
//
//        //////////////////////// SEND ADDITIVES ////////////////////////
//        let mut remainder = add_share_cnt;
//        while remainder >= TI_BATCH_SIZE {
//            let mut rx_buf = Xbuffer { u64_buf: [0u64; U64S_PER_TX] };
//
//            let mut bytes_read = 0;
//            while bytes_read < U8S_PER_TX {
//                let current_bytes = unsafe {
//                    stream.read(&mut rx_buf.u8_buf[bytes_read..])
//                };
//                bytes_read += current_bytes.unwrap();
//            }
//
//            for i in 0..TI_BATCH_SIZE {
//                let u = unsafe { rx_buf.u64_buf[3 * i] };
//                let v = unsafe { rx_buf.u64_buf[3 * i + 1] };
//                let w = unsafe { rx_buf.u64_buf[3 * i + 2] };
//
//                //println!("recvd: ({:X},{:X},{:X})", u, v, w);
//                add_shares.push((Wrapping(u), Wrapping(v), Wrapping(w)));
//            }
//
//            remainder -= TI_BATCH_SIZE;
//        }
//
//        let mut rx_buf = Xbuffer { u64_buf: [0u64; U64S_PER_TX] };
//
//        let mut bytes_read = 0;
//        while bytes_read < U8S_PER_TX {
//            let current_bytes = unsafe {
//                stream.read(&mut rx_buf.u8_buf[bytes_read..])
//            };
//            bytes_read += current_bytes.unwrap();
//        }
//
//        for i in 0..remainder {
//            let u = unsafe { rx_buf.u64_buf[3 * i] };
//            let v = unsafe { rx_buf.u64_buf[3 * i + 1] };
//            let w = unsafe { rx_buf.u64_buf[3 * i + 2] };
//
//            //println!("recvd: ({:X},{:X},{:X})", u, v, w);
//            add_shares.push((Wrapping(u), Wrapping(v), Wrapping(w)));
//        }
//
//        //println!("additive shares recv'd. recving xor shares");
//
//        ///////////////// RECV XOR SHARES
//
//        let mut remainder = xor_share_cnt;
//        while remainder >= TI_BATCH_SIZE {
//            let mut rx_buf = Xbuffer { u64_buf: [0u64; U64S_PER_TX] };
//
//            let mut bytes_read = 0;
//            while bytes_read < U8S_PER_TX {
//                let current_bytes = unsafe {
//                    stream.read(&mut rx_buf.u8_buf[bytes_read..])
//                };
//                bytes_read += current_bytes.unwrap();
//            }
//
//            for i in 0..TI_BATCH_SIZE {
//                let u = unsafe { rx_buf.u64_buf[3 * i] };
//                let v = unsafe { rx_buf.u64_buf[3 * i + 1] };
//                let w = unsafe { rx_buf.u64_buf[3 * i + 2] };
//
//                //println!("recvd: ({:X},{:X},{:X})", u, v, w);
//                xor_shares.push((u, v, w));
//            }
//
//            remainder -= TI_BATCH_SIZE;
//        }
//
//        let mut rx_buf = Xbuffer { u64_buf: [0u64; U64S_PER_TX] };
//
//        let mut bytes_read = 0;
//        while bytes_read < U8S_PER_TX {
//            let current_bytes = unsafe {
//                stream.read(&mut rx_buf.u8_buf[bytes_read..])
//            };
//            bytes_read += current_bytes.unwrap();
//        }
//
//        for i in 0..remainder {
//            let u = unsafe { rx_buf.u64_buf[3 * i] };
//            let v = unsafe { rx_buf.u64_buf[3 * i + 1] };
//            let w = unsafe { rx_buf.u64_buf[3 * i + 2] };
//
//            //println!("recvd: ({:X},{:X},{:X})", u, v, w);
//            xor_shares.push((u, v, w));
//        }
//
//        //println!("xor shares recv'd");
//
//        assert_eq!(add_share_cnt, add_shares.len());
//        assert_eq!(xor_share_cnt, xor_shares.len());
//
//        (add_shares, xor_shares)
    }

    pub fn reset_share_indices(ctx:&mut ComputingParty){
        ctx.dt_shares.current_binary_index=Arc::new(Mutex::new(0));
        ctx.dt_shares.current_additive_index=Arc::new(Mutex::new(0));
        ctx.dt_shares.current_additive_bigint_index=Arc::new(Mutex::new(0));
        ctx.dt_shares.current_equality_index=Arc::new(Mutex::new(0));

    }
}
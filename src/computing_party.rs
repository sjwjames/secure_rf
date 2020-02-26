pub mod computing_party {
    extern crate num;

    use std::num::Wrapping;
    use std::net::{TcpStream, SocketAddr, TcpListener};
    use std::fs::File;
    use std::string::ToString;
    use std::io::{Write, Read, BufReader, BufRead};
    use crate::constants::constants::{TI_BATCH_SIZE, U64S_PER_TX, U8S_PER_TX};
    use crate::decision_tree::decision_tree::{DecisionTreeData, DecisionTreeTraining, DecisionTreeShares, DecisionTreeTIShareMessage, DecisionTreeResult};
    use num::bigint::{BigUint, BigInt, ToBigUint, ToBigInt};
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};
    use crate::message::message::{MessageManager, MQMetaMaps};
    use std::collections::HashMap;
    use std::thread;
    use amiquip::{Connection, Channel, Exchange};
    use threadpool::ThreadPool;
    use std::f64::consts::E;
    use self::num::Num;
    use crate::utils::utils::{receive_u64_triple_shares, receive_u8_triple_shares, receive_u64_shares};


    union Xbuffer {
        u64_buf: [u64; U64S_PER_TX],
        u8_buf: [u8; U8S_PER_TX],
    }

    //author Davis, email:daviscrailsback@gmail.com
    pub struct ComputingParty {
        /* options */
        pub debug_output: bool,
        pub decimal_precision: u32,
        pub raw_tcp_communication: bool,
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

        /* mq */
        pub local_mq_ip: String,
        pub local_mq_port: u16,
        pub remote_mq_ip: String,
        pub remote_mq_port: u16,
        pub local_mq_address: String,
        pub remote_mq_address: String,

        /* mpc*/
        pub asymmetric_bit: u8,

        /* input output */
        pub output_path: String,

        /* DT training Data*/
        pub dt_data: DecisionTreeData,
        pub x_input_path: String,
        pub y_input_path: String,

        /* DT training*/
        pub dt_training: DecisionTreeTraining,
        pub dt_shares: DecisionTreeShares,
        pub dt_results: DecisionTreeResult,
        pub result_file: File,

        /* random forest */
        pub thread_count: usize,
        pub tree_count: usize,
        pub batch_size: usize,
        pub tree_training_batch_size: usize,
        pub instance_selected: u64,
        pub feature_selected: u64,
        pub ohe_add_shares: u64,
        pub ohe_binary_shares: u64,
        pub ohe_equality_shares: u64,
        //multi_thread
        pub thread_hierarchy: Vec<String>,
        pub message_manager: Arc<Mutex<MessageManager>>,
    }

    impl Clone for ComputingParty {
        fn clone(&self) -> Self {
            ComputingParty {
                debug_output: self.debug_output,
                raw_tcp_communication: self.raw_tcp_communication,
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
                local_mq_ip: self.local_mq_ip.clone(),
                local_mq_port: self.local_mq_port,
                remote_mq_ip: self.remote_mq_ip.clone(),
                remote_mq_port: self.remote_mq_port,
                local_mq_address: self.local_mq_address.clone(),
                remote_mq_address: self.remote_mq_address.clone(),
                asymmetric_bit: self.asymmetric_bit,
                output_path: self.output_path.clone(),

                dt_data: self.dt_data.clone(),
                x_input_path: self.x_input_path.clone(),
                y_input_path: self.y_input_path.clone(),
                dt_training: self.dt_training.clone(),
                dt_shares: self.dt_shares.clone(),
                dt_results: self.dt_results.clone(),

                result_file: self.result_file.try_clone().unwrap(),
                thread_count: self.thread_count,
                tree_count: self.tree_count,
                batch_size: self.batch_size,

                tree_training_batch_size: self.tree_training_batch_size,
                instance_selected: self.instance_selected,
                feature_selected: self.feature_selected,
                ohe_add_shares: self.ohe_add_shares,
                ohe_binary_shares: self.ohe_binary_shares,
                ohe_equality_shares: self.ohe_equality_shares,
                thread_hierarchy: self.thread_hierarchy.clone(),
                message_manager: Arc::new(Mutex::new(MessageManager {
                    map: HashMap::new()
                })),

            }
        }
    }

    pub fn load_dt_raw_data(x_path: &String) -> Vec<Vec<Wrapping<u64>>>{
        let x_file = File::open(x_path).expect("input file not found");
        let reader = BufReader::new(x_file);
        let mut x = Vec::new();

        for line in reader.lines() {
            let line = line.unwrap();
            let item: Vec<&str> = line.split(",").collect();
            let length = item.len();
            let mut x_row = Vec::new();
            for i in 0..length {
                let x_item: u64 = item[i].parse().unwrap();
                x_row.push(Wrapping(x_item));
            }
            x.push(x_row);
        }

        x
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
//                Err(_) => println!("{} connection refused by {}", prefix, socket),
                Err(_) => print!(""),
            };
        }
    }

    pub fn produce_dt_data(one_hot_encoding_data: Vec<Vec<u8>>, class_value_count: usize, attr_value_count: usize, attribute_count: usize, instance_count: usize, asymmetric_bit: u8) -> DecisionTreeData {
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
            discretized_x: vec![],
            discretized_y: vec![],
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

        let raw_tcp_communication = match settings.get_bool("raw_tcp_communication") {
            Ok(num) => num as bool,
            Err(error) => {
                panic!("Encountered a problem while parsing raw_tcp_communication: {:?}", error)
            }
        };

        let party_id = match settings.get_int("party_id") {
            Ok(num) => num as u8,
            Err(error) => {
                panic!("Encountered a problem while parsing party_id: {:?}", error)
            }
        };

        let asymmetric_bit = match settings.get_int("asymmetric_bit") {
            Ok(num) => num as u8,
            Err(error) => {
                panic!("Encountered a problem while parsing asymmetric_bit: {:?}", error)
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

        let local_mq_ip = match settings.get_str("local_mq_ip") {
            Ok(num) => num as String,
            Err(error) => {
                panic!("Encountered a problem while parsing local_mq_ip: {:?} ", error)
            }
        };

        let remote_mq_ip = match settings.get_str("remote_mq_ip") {
            Ok(num) => num as String,
            Err(error) => {
                panic!("Encountered a problem while parsing remote_mq_ip: {:?} ", error)
            }
        };

        let local_mq_port = match settings.get_int("local_mq_port") {
            Ok(num) => num as u16,
            Err(error) => {
                panic!("Encountered a problem while parsing local_mq_port: {:?} ", error)
            }
        };

        let remote_mq_port = match settings.get_int("remote_mq_port") {
            Ok(num) => num as u16,
            Err(error) => {
                panic!("Encountered a problem while parsing party1_mq_port: {:?} ", error)
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
        let alpha = alpha.to_biguint().unwrap();


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


        let big_int_prime = BigUint::from_str_radix(&big_int_prime, 10).unwrap();


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

        let class_value_count = match settings.get_int("class_value_count") {
            Ok(num) => num as usize,
            Err(error) => {
                panic!("Encountered a problem while parsing class_value_count: {:?}", error)
            }
        };

        let attr_value_count = match settings.get_int("attr_value_count") {
            Ok(num) => num as usize,
            Err(error) => {
                panic!("Encountered a problem while parsing attr_value_count: {:?}", error)
            }
        };

        let attribute_count = match settings.get_int("attribute_count") {
            Ok(num) => num as usize,
            Err(error) => {
                panic!("Encountered a problem while parsing attribute_count: {:?}", error)
            }
        };

        let instance_count = match settings.get_int("instance_count") {
            Ok(num) => num as usize,
            Err(error) => {
                panic!("Encountered a problem while parsing instance_count: {:?}", error)
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

//        let (class_value_count, attribute_count, attr_value_count, instance_count, one_hot_encoding_matrix) = load_dt_training_file(&x_input_path);

        let dataset_size_bit_length = (attr_value_count as f64).log2().ceil() as u64;

        let dataset_size_prime = 2.0_f64.powf(dataset_size_bit_length as f64) as u64;

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


//        let dt_data = produce_dt_data(one_hot_encoding_matrix, class_value_count, attr_value_count, attribute_count, instance_count, party_id);

        let rfs_field = 2.0_f64.powf((attr_value_count as f64).log2().ceil()) as u64;

        let bagging_field = 2.0_f64.powf((instance_selected as f64).log2().ceil()) as u64;

        let subset_transaction_bit_vector = vec![asymmetric_bit as u8; instance_count];
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
            rfs_field,
            bagging_field,
        };
        let local_mq_address = format!("amqp://guest:guest@{}:{}", local_mq_ip.clone(), local_mq_port);
        let remote_mq_address = format!("amqp://guest:guest@{}:{}", remote_mq_ip.clone(), remote_mq_port);
        let mut result_file = File::create(output_path.clone()).unwrap();
        ComputingParty {
            debug_output,
            raw_tcp_communication,
            decimal_precision,
            party_id,
            ti_ip,
            ti_port0,
            ti_port1,
            party0_ip,
            party0_port,
            party1_ip,
            party1_port,
            asymmetric_bit,
            output_path,
            ti_stream,
            local_mq_ip,
            local_mq_port,
            remote_mq_ip,
            remote_mq_port,
            local_mq_address,
            remote_mq_address,
            in_stream,
            o_stream,
            thread_count,
            tree_count,
            batch_size,
            tree_training_batch_size,
            instance_selected,
            dt_training,
            result_file,
            x_input_path,
            y_input_path,
            dt_shares: DecisionTreeShares {
                additive_triples: Default::default(),
                additive_bigint_triples: vec![],
                rfs_shares: vec![],
                bagging_shares: vec![],
                binary_triples: vec![],
                equality_shares: vec![],
                equality_integer_shares: Default::default(),
                current_additive_index: Arc::new(Mutex::new(0 as usize)),
                current_additive_bigint_index: Arc::new(Mutex::new(0 as usize)),
                current_equality_index: Arc::new(Mutex::new(0 as usize)),
                current_binary_index: Arc::new(Mutex::new(0 as usize)),
                sequential_additive_index: Default::default(),
                sequential_additive_bigint_index: 0,
                sequential_equality_index: 0,
                sequential_binary_index: 0,
                sequential_equality_integer_index: Default::default(),
                sequential_ohe_additive_index: 0,
                matrix_mul_shares: (vec![], vec![], vec![]),
                bagging_matrix_mul_shares: (vec![], vec![], vec![]),
            },
            dt_results: DecisionTreeResult {
                result_list: vec![]
            },
            thread_hierarchy: vec![format!("{}", "main")],
            message_manager: Arc::new(Mutex::new(MessageManager {
                map: HashMap::new()
            })),
            dt_data: DecisionTreeData {
                attr_value_count,
                class_value_count,
                attribute_count,
                instance_count,
                attr_values: vec![],
                class_values: vec![],
                attr_values_bytes: vec![],
                class_values_bytes: vec![],
                attr_values_big_integer: vec![],
                class_values_big_integer: vec![],
                discretized_x: vec![],
                discretized_y: vec![],
            },
            feature_selected,
            ohe_add_shares,
            ohe_binary_shares,
            ohe_equality_shares,
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

    pub fn try_setup_socket(internal_addr: &str, external_addr: &str, message_manager: &Arc<Mutex<MessageManager>>) -> (TcpStream, TcpStream) {
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

    pub fn receive_preprocessing_shares(ctx: &mut ComputingParty){
        let mut stream = ctx.ti_stream.try_clone().unwrap();
        let mut additive_shares = receive_u64_triple_shares(&mut stream, ctx.ohe_add_shares);
        let mut binary_shares = receive_u8_triple_shares(&mut stream,ctx.ohe_binary_shares);
        let mut equality_shares = receive_u64_shares(&mut stream,ctx.ohe_equality_shares);

        let prime = if ctx.dt_training.rfs_field > ctx.dt_data.class_value_count as u64 { ctx.dt_training.rfs_field } else { ctx.dt_data.class_value_count as u64 };
        ctx.dt_shares.additive_triples.insert(prime,additive_shares);
        ctx.dt_shares.equality_integer_shares.insert(prime,equality_shares);
        ctx.dt_shares.binary_triples.append(&mut binary_shares);
        ctx.dt_shares.sequential_additive_index = HashMap::new();
        ctx.dt_shares.sequential_additive_index.insert(prime,0);
        ctx.dt_shares.sequential_equality_integer_index = HashMap::new();
        ctx.dt_shares.sequential_equality_integer_index.insert(prime,0);
        ctx.dt_shares.sequential_binary_index = 0;
    }

    pub fn ti_receive(mut stream: TcpStream, ctx: &mut ComputingParty) -> DecisionTreeShares {
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
        let additive_triples: HashMap<u64, Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)>> = serde_json::from_str(&ti_shares_message.additive_triples).unwrap();

        let mut additive_bigint_triples = Vec::new();
        let additive_bigint_triple_str_vec: Vec<&str> = ti_shares_message.additive_bigint_triples.split(";").collect();
        for item in additive_bigint_triple_str_vec {
            let str_vec: Vec<&str> = item.split("&").collect();

            additive_bigint_triples.push(
                (
                    BigUint::from_str(&str_vec[0]).unwrap(),
                    BigUint::from_str(&str_vec[1]).unwrap(),
                    BigUint::from_str(&str_vec[2]).unwrap()
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
                BigUint::from_str(item).unwrap()
            );
        }

        let mut rfs_shares = Vec::new();
        let rfs_share_vec: Vec<&str> = ti_shares_message.rfs_shares.split(";").collect();
        for item in rfs_share_vec {
            rfs_shares.push(serde_json::from_str(item).unwrap());
        }

        let mut bagging_shares = Vec::new();
        let bagging_vec: Vec<&str> = ti_shares_message.bagging_shares.split(";").collect();
        for item in bagging_vec {
            bagging_shares.push(serde_json::from_str(item).unwrap());
        }

        let mut matrix_mul_shares = (Vec::new(), Vec::new(), Vec::new());
        let matrix_mul_shares_str_vec: Vec<&str> = ti_shares_message.matrix_mul_shares.split("&").collect();
        matrix_mul_shares.0 = serde_json::from_str(&matrix_mul_shares_str_vec[0]).unwrap();
        matrix_mul_shares.1 = serde_json::from_str(&matrix_mul_shares_str_vec[1]).unwrap();
        matrix_mul_shares.2 = serde_json::from_str(&matrix_mul_shares_str_vec[2]).unwrap();

        let mut equality_integer_shares = serde_json::from_str(&ti_shares_message.equality_integer_shares).unwrap();

        let mut bagging_matrix_mul_shares = (Vec::new(), Vec::new(), Vec::new());
        let bagging_matrix_mul_shares_str_vec: Vec<&str> = ti_shares_message.bagging_matrix_mul_shares.split("&").collect();
        bagging_matrix_mul_shares.0 = serde_json::from_str(&bagging_matrix_mul_shares_str_vec[0]).unwrap();
        bagging_matrix_mul_shares.1 = serde_json::from_str(&bagging_matrix_mul_shares_str_vec[1]).unwrap();
        bagging_matrix_mul_shares.2 = serde_json::from_str(&bagging_matrix_mul_shares_str_vec[2]).unwrap();

        let mut sequential_equality_integer_index = HashMap::new();
        sequential_equality_integer_index.insert(ctx.dt_training.rfs_field, 0);
        sequential_equality_integer_index.insert(ctx.dt_training.bagging_field, 0);

        let mut sequential_additive_index = HashMap::new();
        sequential_additive_index.insert(ctx.dt_training.prime, 0);
        sequential_additive_index.insert(ctx.dt_training.rfs_field, 0);
        sequential_additive_index.insert(ctx.dt_training.bagging_field, 0);

        DecisionTreeShares {
            additive_triples,
            additive_bigint_triples,
            rfs_shares,
            bagging_shares,
            binary_triples,
            equality_shares,
            equality_integer_shares,
            current_additive_index: Arc::new(Mutex::new(0 as usize)),
            current_additive_bigint_index: Arc::new(Mutex::new(0 as usize)),
            current_equality_index: Arc::new(Mutex::new(0 as usize)),
            current_binary_index: Arc::new(Mutex::new(0 as usize)),
            sequential_additive_index,
            sequential_additive_bigint_index: 0,
            sequential_equality_index: 0,
            sequential_binary_index: 0,
            sequential_equality_integer_index,
            sequential_ohe_additive_index: 0,
            matrix_mul_shares,
            bagging_matrix_mul_shares,
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
}
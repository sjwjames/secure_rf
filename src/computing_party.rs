pub mod computing_party {
    use std::num::Wrapping;
    use std::net::{TcpStream, SocketAddr, TcpListener};
    use std::fs::File;
    use crate::thread_pool::thread_pool::ThreadPool;
    use std::io::{Write, Read};
    use crate::constants::constants::{TI_BATCH_SIZE, U64S_PER_TX, U8S_PER_TX};
    use num_bigint::{BigUint, BigInt};

    union Xbuffer {
        u64_buf: [u64; U64S_PER_TX],
        u8_buf: [u8; U8S_PER_TX],
    }

    //author Davis, email:daviscrailsback@gmail.com
    pub struct ComputingParty {
        /* options */
        pub debug_output: bool,

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
        pub xor_shares_per_iter: usize,
        pub add_shares_per_iter: usize,

        /* input output */
        pub output_path: String,
        pub x_matrix: Vec<Vec<Wrapping<u64>>>,
        pub y_matrix: Vec<Vec<Wrapping<u64>>>,

        /* DT training*/

        /* random forest */
        pub thread_count: usize,
        pub tree_count: usize,
        pub batch_size: usize,
        pub attribute_count: usize,
        pub instance_count: usize,

        pub corr_rand: Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)>,
        pub corr_rand_xor: Vec<(u64, u64, u64)>,
        pub big_int_ti_shares:Vec<(BigInt, BigInt, BigInt)>,
        pub equality_ti_shares:Vec<BigInt>,


    }

    impl Clone for ComputingParty {
        fn clone(&self) -> Self {
            ComputingParty {
                debug_output: self.debug_output,
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
                xor_shares_per_iter: self.xor_shares_per_iter,
                add_shares_per_iter: self.add_shares_per_iter,
                output_path: self.output_path.clone(),
                x_matrix: self.x_matrix.clone(),
                y_matrix: self.y_matrix.clone(),
                thread_count: self.thread_count,
                tree_count: self.tree_count,
                batch_size: self.batch_size,
                attribute_count: self.attribute_count,
                instance_count: self.instance_count,
                corr_rand: Vec::new(),
                corr_rand_xor: Vec::new(),
                big_int_ti_shares: vec![],
                equality_ti_shares: vec![]
            }
        }
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

        let xor_shares_per_iter = match settings.get_int("xor_shares_per_iter") {
            Ok(num) => num as usize,
            Err(error) => {
                panic!("Encountered a problem while parsing xor_shares_per_iter: {:?}", error)
            }
        };

        let add_shares_per_iter = match settings.get_int("add_shares_per_iter") {
            Ok(num) => num as usize,
            Err(error) => {
                panic!("Encountered a problem while parsing add_shares_per_iter: {:?}", error)
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


        let x_matrix = load_u64_matrix(&x_input_path, instance_count as usize, false, (party_id as u64) << decimal_precision as u64);
        let y_matrix = load_u64_matrix(&y_input_path, instance_count as usize, false, (party_id as u64) << decimal_precision as u64);
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


        ComputingParty {
            debug_output,
            party_id,
            ti_ip,
            ti_port0,
            ti_port1,
            party0_ip,
            party0_port,
            party1_ip,
            party1_port,
            asymmetric_bit: party_id,
            xor_shares_per_iter,
            add_shares_per_iter,
            output_path,
            x_matrix,
            y_matrix,
            ti_stream,
            in_stream,
            o_stream,
            thread_count,
            tree_count,
            batch_size,
            instance_count,
            attribute_count,
            corr_rand: Vec::new(),
            corr_rand_xor: Vec::new(),
            big_int_ti_shares: vec![],
            equality_ti_shares: vec![]
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

    pub fn ti_receive(mut stream: TcpStream, add_share_cnt: usize, xor_share_cnt: usize) ->
    (Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)>, Vec<(u64, u64, u64)>) {
        stream.set_ttl(std::u32::MAX).expect("set_ttl call failed");
        stream.set_write_timeout(None).expect("set_write_timeout call failed");
        stream.set_read_timeout(None).expect("set_read_timeout call failed");

        let mut xor_shares: Vec<(u64, u64, u64)> = Vec::new();
        let mut add_shares: Vec<(Wrapping<u64>, Wrapping<u64>, Wrapping<u64>)> = Vec::new();

        //println!("sending ready msg to ti");

        let mut recv_buf = [0u8; 11];
        let msg = b"send shares";
        let mut bytes_written = 0;
        while bytes_written < msg.len() {
            let current_bytes = stream.write(&msg[bytes_written..]);
            bytes_written += current_bytes.unwrap();
        }
        //println!("ready msg sent. awaiting confimation");


        let mut bytes_read = 0;
        while bytes_read < recv_buf.len() {
            let current_bytes = stream.read(&mut recv_buf[bytes_read..]).unwrap();
            bytes_read += current_bytes;
        }

        assert_eq!(msg, &recv_buf);


        //println!("got confirmation ; recving additive shares");

        //////////////////////// SEND ADDITIVES ////////////////////////
        let mut remainder = add_share_cnt;
        while remainder >= TI_BATCH_SIZE {
            let mut rx_buf = Xbuffer { u64_buf: [0u64; U64S_PER_TX] };

            let mut bytes_read = 0;
            while bytes_read < U8S_PER_TX {
                let current_bytes = unsafe {
                    stream.read(&mut rx_buf.u8_buf[bytes_read..])
                };
                bytes_read += current_bytes.unwrap();
            }

            for i in 0..TI_BATCH_SIZE {
                let u = unsafe { rx_buf.u64_buf[3 * i] };
                let v = unsafe { rx_buf.u64_buf[3 * i + 1] };
                let w = unsafe { rx_buf.u64_buf[3 * i + 2] };

                //println!("recvd: ({:X},{:X},{:X})", u, v, w);
                add_shares.push((Wrapping(u), Wrapping(v), Wrapping(w)));
            }

            remainder -= TI_BATCH_SIZE;
        }

        let mut rx_buf = Xbuffer { u64_buf: [0u64; U64S_PER_TX] };

        let mut bytes_read = 0;
        while bytes_read < U8S_PER_TX {
            let current_bytes = unsafe {
                stream.read(&mut rx_buf.u8_buf[bytes_read..])
            };
            bytes_read += current_bytes.unwrap();
        }

        for i in 0..remainder {
            let u = unsafe { rx_buf.u64_buf[3 * i] };
            let v = unsafe { rx_buf.u64_buf[3 * i + 1] };
            let w = unsafe { rx_buf.u64_buf[3 * i + 2] };

            //println!("recvd: ({:X},{:X},{:X})", u, v, w);
            add_shares.push((Wrapping(u), Wrapping(v), Wrapping(w)));
        }

        //println!("additive shares recv'd. recving xor shares");

        ///////////////// RECV XOR SHARES

        let mut remainder = xor_share_cnt;
        while remainder >= TI_BATCH_SIZE {
            let mut rx_buf = Xbuffer { u64_buf: [0u64; U64S_PER_TX] };

            let mut bytes_read = 0;
            while bytes_read < U8S_PER_TX {
                let current_bytes = unsafe {
                    stream.read(&mut rx_buf.u8_buf[bytes_read..])
                };
                bytes_read += current_bytes.unwrap();
            }

            for i in 0..TI_BATCH_SIZE {
                let u = unsafe { rx_buf.u64_buf[3 * i] };
                let v = unsafe { rx_buf.u64_buf[3 * i + 1] };
                let w = unsafe { rx_buf.u64_buf[3 * i + 2] };

                //println!("recvd: ({:X},{:X},{:X})", u, v, w);
                xor_shares.push((u, v, w));
            }

            remainder -= TI_BATCH_SIZE;
        }

        let mut rx_buf = Xbuffer { u64_buf: [0u64; U64S_PER_TX] };

        let mut bytes_read = 0;
        while bytes_read < U8S_PER_TX {
            let current_bytes = unsafe {
                stream.read(&mut rx_buf.u8_buf[bytes_read..])
            };
            bytes_read += current_bytes.unwrap();
        }

        for i in 0..remainder {
            let u = unsafe { rx_buf.u64_buf[3 * i] };
            let v = unsafe { rx_buf.u64_buf[3 * i + 1] };
            let w = unsafe { rx_buf.u64_buf[3 * i + 2] };

            //println!("recvd: ({:X},{:X},{:X})", u, v, w);
            xor_shares.push((u, v, w));
        }

        //println!("xor shares recv'd");

        assert_eq!(add_share_cnt, add_shares.len());
        assert_eq!(xor_share_cnt, xor_shares.len());

        (add_shares, xor_shares)
    }
}
pub mod computing_party {
    use std::num::Wrapping;
    use crate::PartyLifeCycle;
    use std::net::{TcpStream, SocketAddr, TcpListener};
    use utils::thread_pool::thread_pool::ThreadPool;
    // author:Davis
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

        /* mpc */
        pub asymmetric_bit: u8,
        pub xor_shares_per_iter: usize,
        pub add_shares_per_iter: usize,

        /* input output */
        pub output_path: String,
        pub x_matrix: Vec<Vec<Wrapping<u64>>>,
        pub y_matrix: Vec<Vec<Wrapping<u64>>>,
        pub in_stream: TcpStream,
        pub o_stream: TcpStream,
        pub ti_stream: TcpStream,
        pub thread_pool: ThreadPool,
    }

    pub fn initiate(settings_file: String) -> ComputingParty {
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

        let thread_pool = ThreadPool::new(thread_count);

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
            thread_pool
        }
    }

    fn try_connect(socket: &SocketAddr, prefix: &str) -> TcpStream {
        loop {
            match TcpStream::connect(socket) {
                Ok(stream) => return stream,
                Err(_) => println!("{} connection refused by {}", prefix, socket),
            };
        }
    }
}
pub mod message {
    use std::collections::HashMap;
    use serde::{Serialize, Deserialize, Serializer};
    use std::sync::{Arc, Mutex};
    use std::net::{TcpStream, TcpListener};
    use std::io::{BufReader, BufRead, Read};
    use std::time::SystemTime;
    use threadpool::ThreadPool;
    use std::thread;
    use crate::computing_party::computing_party::ComputingParty;
    use amiquip::{Channel, Connection, Exchange};

    pub const MAX_SEARCH_TIMES: u128 = 1000;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct RFMessage {
        pub message_id: String,
        pub message_content: String,
    }

    pub struct MessageManager {
        pub map: HashMap<String, RFMessage>,
    }

    pub struct MQMetaMaps<'a>{
        pub connection_map:HashMap<String,Connection>,
        pub exchange_map:HashMap<String,Exchange<'a>>
    }

    impl Clone for RFMessage {
        fn clone(&self) -> Self {
            RFMessage {
                message_id: self.message_id.clone(),
                message_content: self.message_content.clone(),
            }
        }
    }

    impl Clone for MessageManager {
        fn clone(&self) -> Self {
            MessageManager {
                map: self.map.clone()
            }
        }
    }


    impl MessageManager {
        fn add_message(&mut self, message: &RFMessage) -> Result<&'static str, &'static str> {
            if self.map.contains_key(&message.message_id) {
                Err("Unable to add the message since the id exists already")
            } else {
                self.map.insert(message.message_id.clone(), message.clone());
                Ok("Successfully add the message")
            }
        }

//        pub fn search_pop_message(&mut self, message_id: String, in_stream: &TcpStream) -> Result<RFMessage, &'static str> {
//            let mut remainder = MAX_SEARCH_TIMEOUT;
//            if self.map.contains_key(&message_id) {
//                let mut message = self.map.remove(&message_id);
//                return Ok(message.unwrap());
//            }
//            while remainder > 0 {
//                let mut now = SystemTime::now();
//                let mut reader = BufReader::new(in_stream);
//                let mut message_string = String::new();
//                reader.read_line(&mut message_string);
//                let message: RFMessage = serde_json::from_str(&message_string).unwrap();
//                if message.message_id==message_id{
//                    return Ok(message);
//                }else{
//                    self.add_message(&message);
//                }
//
//                remainder -= now.elapsed().unwrap().as_millis();
//            }
//            Err("Cannot find the message")
//        }
    }

    pub fn search_pop_message(ctx:&mut ComputingParty, message_id: String) -> Result<RFMessage, &'static str> {
        let in_stream = ctx.in_stream.try_clone().unwrap();
        let manager = Arc::clone(&ctx.message_manager);
        let mut manager_content = manager.lock().unwrap();
        if (*manager_content).map.contains_key(&message_id) {
            let mut message = (*manager_content).map.remove(&message_id);
            return Ok(message.unwrap());
        }
        let mut manager_arc = Arc::clone(&manager);
        loop{
            let in_stream=in_stream.try_clone().unwrap();
            let mut reader = BufReader::new(&in_stream);
            let mut line = String::new();
            reader.read_line(&mut line);
            let message: RFMessage = serde_json::from_str(&line).unwrap();
            if message.message_id.eq(&message_id) {
                return Ok(message);
            }
            let mut manager = manager_arc.lock().unwrap();
            (*manager).add_message(&message);
        }
        Err("Cannot find the message")
    }

//    pub fn setup_message_manager(in_stream: &TcpStream,manager:&Arc<Mutex<MessageManager>>) {
//        let mut manager_copied = Arc::clone(manager);
//        thread::spawn(move || {
//            let in_stream = in_stream.try_clone().unwrap();
//            let mut reader = BufReader::new(&stream);
//            for line in reader.lines(){
//                let message: RFMessage = serde_json::from_str(&line.unwrap()).unwrap();
//                println!("{} received", &message.message_id);
//                let mut manager = manager_copied.lock().unwrap();
//                (*manager).add_message(&message);
//            }
//
//        });
//    }

}
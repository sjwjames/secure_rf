pub mod message {
    use std::collections::HashMap;
    use serde::{Serialize, Deserialize, Serializer};
    use std::sync::{Arc, Mutex};
    use std::net::TcpStream;
    use std::io::{BufReader, BufRead, Read};
    use std::time::SystemTime;
    use threadpool::ThreadPool;
    use std::thread;
    use crate::computing_party::computing_party::ComputingParty;

    pub const MAX_SEARCH_TIMEOUT: u128 = 60 * 1000;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct RFMessage {
        pub message_id: String,
        pub message_content: String,
    }

    pub struct MessageManager {
        pub map: HashMap<String, RFMessage>,
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

    pub fn search_pop_message(ctx:&mut ComputingParty)->Result<RFMessage, &'static str> {
        let manager = &mut ctx.message_manager;
        let message_id = ctx.thread_hierarchy.join(":");
        let mut in_stream = ctx.in_stream.try_clone().unwrap();
        let mut remainder = MAX_SEARCH_TIMEOUT;
        if manager.map.contains_key(&message_id) {
            let mut message = manager.map.remove(&message_id);
            return Ok(message.unwrap());
        }
        while remainder > 0 {
            let mut now = SystemTime::now();
            let mut reader = BufReader::new(&in_stream);
            let mut message_string = String::new();
            reader.read_line(&mut message_string);
            let message: RFMessage = serde_json::from_str(&message_string).unwrap();
            if message.message_id==message_id{
                return Ok(message);
            }else{
                manager.add_message(&message);
            }

            remainder -= now.elapsed().unwrap().as_millis();
        }
        Err("Cannot find the message")
    }
}
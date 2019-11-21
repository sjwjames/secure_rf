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

    pub const MAX_SEARCH_TIMES: u128 = 1000;

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


    impl MessageManager {
        pub fn add_message(&mut self, message: &RFMessage) -> Result<&'static str, &'static str> {
            self.map.insert(message.message_id.clone(), message.clone());
            Ok("Successfully add the message")
        }
    }

//    pub fn search_pop_message(ctx: &mut ComputingParty, message_id: String) -> Result<RFMessage, &'static str> {
//        println!("querying {}", &message_id);
//        let in_stream = ctx.in_stream.try_clone().unwrap();
//        let manager = Arc::clone(&ctx.message_manager);
//        let mut manager_content = manager.lock().unwrap();
//        if (*manager_content).map.contains_key(&message_id) {
//            let mut message = (*manager_content).map.remove(&message_id);
//            println!("found    {}", &message_id);
//            return Ok(message.unwrap());
//        }
//
//        let mut manager_arc = Arc::clone(&manager);
//        let in_stream = in_stream.try_clone().unwrap();
//        let mut reader = BufReader::new(&in_stream);
//        let mut line = String::new();
//        reader.read_line(&mut line);
//        let message: RFMessage = serde_json::from_str(&line).unwrap();
//        if message.message_id.eq(&message_id) {
//            println!("found    {}", &message_id);
//            return Ok(message);
//        }
//        let mut manager = manager_arc.lock().unwrap();
//        (*manager).add_message(&message);
//
//        search_pop_message(ctx, message_id);
//
//        Err("Cannot find the message")
//    }


    pub fn search_pop_message(ctx: &mut ComputingParty, message_id: String) -> Result<RFMessage, &'static str> {
        println!("querying {}", &message_id);
        let mut manager_content = ctx.message_manager.lock().unwrap();
        loop{
            if (*manager_content).map.contains_key(&message_id){
                break;
            }
        }
        let mut message = (*manager_content).map.remove(&message_id);
        println!("found    {}", &message_id);
        return Ok(message.unwrap());
//        if (*manager_content).map.contains_key(&message_id) {
//            let mut message = (*manager_content).map.remove(&message_id);
//            println!("found    {}", &message_id);
//            return Ok(message.unwrap());
//        }else{
//
//        }
        Err("asd")
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
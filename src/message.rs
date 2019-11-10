pub mod message {
    use std::collections::HashMap;
    use serde::{Serialize, Deserialize, Serializer};
    use std::sync::{Arc, Mutex};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Message {
        pub message_id: String,
        pub message_content:String,
    }

    pub struct MessageManager{
        pub map: HashMap<String,Message>,
        pub current_message_id:Arc<Mutex<String>>
    }

    impl Clone for Message{
        fn clone(&self) -> Self {
            Message{
                message_id: self.message_id.clone(),
                message_content: self.message_content.clone()
            }
        }
    }

    impl MessageManager{
        pub fn new(thread_prefix:&str)->Self{
            MessageManager{
                map:HashMap::new(),
                current_message_id:Arc::new(Mutex::new(format!("{}:0",thread_prefix)))
            }
        }
        pub fn add_message(&mut self,message:&Message)->Result<&'static str,&'static str>{
            if self.map.contains_key(&message.message_id){
                Err("Unable to add the message since the id exists already")
            }else{
                self.map.insert(message.message_id.clone(),message.clone());
                Ok("Successfully add the message")
            }
        }

        pub fn pop_message(&mut self,message_id:String)->Result<Message,&'static str>{
            if self.map.contains_key(&message_id){
                let mut message= self.map.remove(&message_id);
                Ok(message.unwrap())
            }else{
                Err("Cannot find the message")
            }
        }

        pub fn generate_new_message(&self,message_content:String)->Message{
            let mut current_message = &*(self.current_message_id.lock().unwrap());
            let message_in_array:Vec<&str> = current_message.split(":").collect();
            let mut current_id:u64 = message_in_array[message_in_array.len()-1].parse().unwrap();
            let prefix = message_in_array[0..message_in_array.len()-1].join(":");
            current_id+=1;
            Message{
                message_id: format!("{}:{}",prefix,current_id),
                message_content
            }
        }
    }
}
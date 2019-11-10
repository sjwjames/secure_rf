pub mod message {
    use std::collections::HashMap;
    use serde::{Serialize, Deserialize, Serializer};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Message {
        pub message_id: u64,
        pub message_content:String,
    }

    pub struct MessageMap{
        pub map: HashMap<u64,Message>
    }

    impl Clone for Message{
        fn clone(&self) -> Self {
            Message{
                message_id: self.message_id.clone(),
                message_content: self.message_content.clone()
            }
        }
    }

    impl MessageMap{
        pub fn new()->Self{
            MessageMap{
                map:HashMap::new()
            }
        }
        pub fn add_message(&mut self,message:&Message)->Result<&'static str,&'static str>{
            if self.map.contains_key(&message.message_id){
                Err("Unable to add the message since the id exists already")
            }else{
                self.map.insert(message.message_id,message.clone());
                Ok("Successfully add the message")
            }
        }

        pub fn pop_message(&mut self,message_id:u64)->Result<Message,&'static str>{
            if self.map.contains_key(&message_id){
                let mut message= self.map.remove(&message_id);
                Ok(message.unwrap())
            }else{
                Err("Cannot find the message")
            }
        }
    }
}
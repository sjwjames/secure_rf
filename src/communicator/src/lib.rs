use std::net::TcpStream;
use std::io::Read;

pub trait Communicator{
    fn handle_connection(mut stream:TcpStream,buffer_size:usize){
//        let mut buffer = [0;buffer_size];
//        stream.read(&mut buffer).unwrap();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

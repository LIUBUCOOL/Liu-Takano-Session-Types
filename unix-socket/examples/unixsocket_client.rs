extern crate unix_socket;
use libc as c;
use unix_socket::UnixSocket;
use std::io::Result;
use std::net::{Shutdown};
use std::path::Path;
extern crate tempdir;
use std::io::BufWriter;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

extern crate bincode;
extern crate serde;

fn main() {

    let socket_path = Path::new("unixsocket");
    let mut unixsocket_client = UnixSocket::new(c::SOCK_STREAM, 0).unwrap();
    let connect = unixsocket_client.connect(socket_path);
    if connect.is_ok(){
        println!("Connect to server succeed!");
    }
    
    let buffer_write = bincode::serialize(&123).unwrap();
    Write::write(&mut unixsocket_client,&buffer_write).unwrap();
    Write::flush(&mut unixsocket_client).unwrap();
    println!("send buffer: {:?}", buffer_write);

    let a = std::thread::spawn(move || {
    let mut buffer_read:Vec<u8> = [0;4].to_vec();
    unixsocket_client.read(&mut buffer_read).unwrap();
    println!("recv buffer: {:?}", buffer_read);
    let message:bool = bincode::deserialize(&buffer_read[..]).unwrap();
    println!("{}", message);
    if message {
        println!("123 is even");
    }else{
        println!("123 is odd");
    }
    });
    a.join().unwrap();
}




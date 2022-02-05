use libc as c;
extern crate unix_socket;
use unix_socket::{Socket};
use std::net::{Shutdown};
use std::string::{String};


fn main()-> std::io::Result<()>{
    let clientsock = Socket::new(c::AF_INET, c::SOCK_STREAM, 0).unwrap();
    let a = clientsock.connect("127.0.0.1:8080");
    if a.is_ok(){
        println!("connect address to socket fd succeed!")
    }
    let b = clientsock.write(b"Hello, server!");
    if b.is_ok(){
        println!("send message to server succeed!")
    }
    
    let mut buffer = [0; 1024]; 
    let message = clientsock.read(&mut buffer);
    if message.is_ok(){
        println!("Got message from server succeed!");
    }
    println!("Message is:{}",String::from_utf8_lossy(&buffer[..]));
    clientsock.shut_down(Shutdown::Both).unwrap();
    
    
    Ok(())
}

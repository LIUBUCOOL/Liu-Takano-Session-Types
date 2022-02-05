extern crate unix_socket;
use serde::de::DeserializeOwned;
use unix_socket::*;
use std::io::BufWriter;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
extern crate bincode;
extern crate serde;
use std::marker; 



unsafe fn writetcp<A: marker::Send + 'static + serde::Serialize + DeserializeOwned>(socket: Socket, x: A) {
    let buffer: Vec<u8> = bincode::serialize(&x).unwrap();
    let mut writer = BufWriter::new(socket);
    Write::write_all(&mut writer,&buffer).unwrap();
    Write::flush(&mut writer).unwrap();
    
}
unsafe fn readtcp< A: DeserializeOwned + marker::Send + 'static>(socket: Socket) -> A {
    let mut buffer:Vec<u8>  = [0u8;1024].to_vec();
    let mut reader = BufReader::new(socket);
    Read::read(&mut reader ,&mut buffer).unwrap();
    let message:A = bincode::deserialize(&mut buffer).unwrap();
    message
}
fn client(socket:Socket){
    let id = 1;
    let number = 66;
    unsafe{writetcp(socket,id);}
    let result:bool = unsafe{readtcp(socket)};
    if result {
        unsafe{writetcp(socket, number)};
    }else{
        println!("id error");
        drop(socket);
        return;
    }
    let result2:bool = unsafe{readtcp(socket)};
    if result2 {
        println!("{} is even", number);
        drop(socket);
    }else{
        println!("{} is add", number);
        drop(socket);
    }
    

}
fn server(socket:Socket){
    loop{
    let id:i32 = unsafe{readtcp(socket)};
    if !id == 1{
        unsafe{writetcp(socket, false)};
        drop(socket);
    }else{
        unsafe{writetcp(socket, true)};
    }
    let number:i32 = unsafe{readtcp(socket)};
    if number % 2 == 0 {
        unsafe{writetcp(socket, true)};
    }else{
        unsafe{writetcp(socket, false)};
    }
    drop(socket);
    }
}

fn main(){
    let a = std::thread::spawn(move || {
        let socket = tcp_server_8080();
    for stream in socket.incoming(){
        match stream{
            Ok(stream) => {
                std::thread::spawn(move || {
                    server(stream);
                });
                
            },
            Err(e) => eprintln!("socket err{}",e),
        }
    }
    });
    let b = std::thread::spawn(move || {
        let socket1 = tcp_client_8080();
        client(socket1);
    });

    let _ = (a.join(),b.join());

}

extern crate unix_socket;
use libc as c;
use unix_socket::{UnixSocket};
use std::io::Result;
use std::thread;
use std::time::Duration;
extern crate tempdir;
use std::path::Path;
use std::io::BufWriter;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::fs;
extern crate bincode;
extern crate serde;
use serde::{Serialize,Deserialize};


//server
fn main() -> Result<()> {
    let socket_path = Path::new("unixsocket");
    if socket_path.exists(){
        fs::remove_file(socket_path).unwrap();
    }
    let unixsocket_serv = UnixSocket::new(c::SOCK_STREAM, 0).unwrap();
    let bind = unixsocket_serv.bind(&socket_path);
    if bind.is_ok(){
        println!("Unix server socket bound path OK!")
    }

    let listen = unixsocket_serv.listen(128);
    if listen.is_ok(){
        println!("listening......");
    }
    for client in unixsocket_serv.incoming(){
        match client{
            Ok(client) => {
                thread::spawn(move || handle_client(client));
            }
            Err(e) => {
                eprintln!("failed:{}",e);
            }
        }
    }
    Ok(())
}

fn handle_client(mut client: UnixSocket) {
    let mut buffer_read = [0;4];
    client.read(&mut buffer_read).unwrap();
    println!("recv buffer: {:?}", buffer_read);
    let message:i32 = bincode::deserialize(&buffer_read).unwrap();
    println!("server receive: {}",message); 

    let a = std::thread::spawn(move || {
        if message % 2 == 0 {
            client.write(&bincode::serialize(&true).unwrap()).unwrap();
            client.flush().unwrap();
        } else {
            client.write(&bincode::serialize(&false).unwrap()).unwrap();
            client.flush().unwrap();
        }
    });
    a.join().unwrap();

}




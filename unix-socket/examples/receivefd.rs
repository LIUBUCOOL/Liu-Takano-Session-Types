extern crate unix_socket;
use std::io::Read;
use std::os::unix::io::FromRawFd;
use libc as c;
use unix_socket::{UnixSocket};
use std::io::Result;
use std::thread;
use std::time::Duration;
extern crate tempdir;
use std::path::Path;
use std::fs::{File};
use std::io::{Seek as _, SeekFrom};

//server
fn main() -> Result<()> {

    let socket_path = Path::new("unixsocket");
    let unixsocket_serv = UnixSocket::new(c::SOCK_STREAM, 0).unwrap();
    let bind = unixsocket_serv.bind(&socket_path);
    if bind.is_ok(){
        println!("Unix server socket bound path OK!")
    }
    let listen = unixsocket_serv.listen(128);
    if listen.is_ok(){
        println!("listening......");
    }
    for client in unixsocket_serv.incoming() { //イテレータ
        match client{
            Ok(client) => {
                thread::spawn(move || handle_client(client));
            }
            Err(e) => {
                eprintln!("failed:{}",e);
            }
        }
    }
    //fs::remove_file("unixsocket").expect("could not remove file");
    Ok(())
    
}
fn handle_client(client: UnixSocket) {
    let fd = client.recv_fd().unwrap();
    let mut file = unsafe{File::from_raw_fd(fd)};
    file.seek(SeekFrom::Start(0)).expect("Failed to seek");
    println!("Got File Descriptor:\n{:?}",file);
    let mut buffer = String::new();
    let result = file.read_to_string(&mut buffer).unwrap();
    println!("Number of bytes read:\n{}",result);
    println!("Message is :\n{}",&buffer);
    thread::sleep(Duration::from_secs(1));  
}


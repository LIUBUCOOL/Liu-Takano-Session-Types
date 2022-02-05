extern crate unix_socket;
use std::fs::OpenOptions;
use std::io::Write;
use libc as c;
use unix_socket::UnixSocket;
use std::io::{Result};
use std::net::{Shutdown};
use std::path::Path;
use std::os::unix::io::AsRawFd;
extern crate tempdir;


fn main() -> Result<()>{
    let socket_path = Path::new("unixsocket");
    let mut file = OpenOptions::new().read(true).write(true).append(true).create(true).open("file.txt").unwrap();
    let unixsocket_client = UnixSocket::new(c::SOCK_STREAM, 0).unwrap();
    let connect = unixsocket_client.connect(socket_path);
    if connect.is_ok(){
        println!("Connect to server succeed!");
    }
    let message = b"hello,world!";
    file.write_all(message).unwrap();
    let write = unixsocket_client.send_fd(file.as_raw_fd());
    if write.is_ok(){
        println!("Send file descriptor succeed!");
    } 
    unixsocket_client.shutdown(Shutdown::Both).unwrap();
    Ok(())
}

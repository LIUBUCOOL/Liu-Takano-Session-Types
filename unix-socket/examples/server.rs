use libc as c;
extern crate unix_socket;
use unix_socket::{Socket};
use std::thread;
use std::time;


fn main() -> std::io::Result<()> {
    let fd = Socket::new(c::AF_INET, c::SOCK_STREAM, 0).unwrap();
    let a = fd.bind("127.0.0.1:8080");
    if a.is_ok(){
        println!("bind address succeed!")
    }
    let b = fd.listen(128);
    if b.is_ok(){
        println!("[Server]: listening......")
    }
    for stream in fd.incoming(){
        match stream {
            Err(e) => eprintln!("failed: {}", e),
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream)
                });
            }
        }
        }
    Ok(())

}
fn handle_client(stream: Socket) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    println!("Got message:\n{}",String::from_utf8_lossy(&buffer[..]));
    stream.write(b"hello, client. Got your message!").unwrap();
    thread::sleep(time::Duration::from_secs(1));  
}
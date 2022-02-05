
use std::time::Duration;
use std::os::unix::io::FromRawFd;
use std::os::unix::io::AsRawFd;
extern crate unix_socket;
use unix_socket::*;
use std::thread;
use std::time::Instant; 


fn judgment(id:i32)->bool{
    id == 1
}
fn id_verif(tcpserver8080: Socket, unixclient: UnixSocket) {
    let mut buffer = [0;1];
    //thread::sleep(Duration::from_millis(10));
    tcpserver8080.read(&mut buffer).unwrap();
    if judgment(buffer[0]) {
        let a = thread::spawn(move || {
            tcpserver8080.write(&[true]).unwrap();
            let tcpserver8081 = tcp_server_8081();
            for tcpatm in tcpserver8081.incoming(){
                match tcpatm{
                    Err(e) => eprintln!("{}",e),
                    Ok(tcpatm) => {
                        let b = thread::spawn(move || {
                            unixclient.send_fd(tcpatm.as_raw_fd()).unwrap();
                        }); 
                        b.join().unwrap();
                    }}}});
                        a.join().unwrap();
    }else{
        panic!("Error ID");
    }}


fn client(tcpclient8080: Socket){
    tcpclient8080.write(&[1]).unwrap();
    let mut buffer = [false;1];
    tcpclient8080.read(&mut buffer).unwrap();
    if buffer[0] == true {
        thread::sleep(Duration::from_millis(50));
        let tcpclient8181 = tcp_client_8081();
        let option = [2];
        let amount = [30];
        tcpclient8181.write(&option).unwrap();
        tcpclient8181.write(&amount).unwrap();
        let mut buffer2 = [0];
        tcpclient8181.read(&mut buffer2).unwrap();
        println!("Operation successd! Balance: {}",buffer2[0]);
    }
}
fn client2(tcpclient8080: Socket){
    tcpclient8080.write(&[1]).unwrap();
    let mut buffer = [false;1];
    tcpclient8080.read(&mut buffer).unwrap();
    if buffer[0] == true {
        thread::sleep(Duration::from_millis(50));
        let tcpclient8181 = tcp_client_8081();
        let option = [1];
        let amount = [30];
        tcpclient8181.write(&option).unwrap();
        tcpclient8181.write(&amount).unwrap();
        let mut buffer2 = [0];
        tcpclient8181.read(&mut buffer2).unwrap();
        println!("Operation successd! Balance: {}",buffer2[0]);
    }
}

fn atm(unixatm:UnixSocket){
    loop{
        let tcpatmsock = unixatm.recv_fd().unwrap();
        let socket = unsafe{Socket::from_raw_fd(tcpatmsock)};
        let mut option = [0];
        let mut amount = [0];
        socket.read(&mut option).unwrap();
        socket.read(&mut amount).unwrap();
        println!("option:{:?},amount:{:?}", option, amount);
        let mut balance = 200;
        match option[0]{
            1 => {
                balance += amount[0];
                socket.write(&[balance]).unwrap();
            },
            2 =>{
                if amount[0] > balance{
                    panic!("Insufficient balance");
                }else{
                balance -= amount[0];
                socket.write(&[balance]).unwrap();
                }
            },
            _ =>panic!("error selection"),
        } 
    }
    }


fn main() {
    let start = Instant::now();
    let a = std::thread::spawn(move || {
        let unixatm = unix_server();
        for stream in unixatm.incoming(){
            match stream{
                Err(e) => eprintln!("{}",e),
                Ok(stream) => {
                   thread::spawn(move || {
                        atm(stream);
                    });
                },
            }
        }
    });

    let b = std::thread::spawn(move || {
        let tcpserver8080 = tcp_server_8080();
        for stream in tcpserver8080.incoming(){
            match stream{
                Err(e) => eprintln!("{}",e),
                Ok(stream) =>{
                    thread::spawn(move || {
                        let unixclient = unix_client();
                        id_verif(stream, unixclient);
                     });
                }
            }
        }
    });

        let c = std::thread::spawn(move || {
            let mut count = 0;
            for i in 1..100{
                let tcpsock = tcp_client_8080();
                client(tcpsock);
                count += 1;
                println!("{}",count)
            }
            println!("time cost: {:?} ms", start.elapsed().as_millis());
        });



    let _ = (a.join(),b.join(),c.join());

    
}


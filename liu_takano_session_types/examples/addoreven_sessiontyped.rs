
extern crate liu_takano_session_types;
use std::time::Duration;
use std::marker::PhantomData;
use liu_takano_session_types::*;
extern crate serde_derive;
extern crate serde;
use std::thread;
use std::time::Instant; 

type Server = Recv<i32, Send<bool, End>>;
type Client = <Server as HasDual>::Dual;

type CliFirstStep = Send<i32,Recv<bool,End>>;
type Idverif = <CliFirstStep as HasDual>::Dual;

type SendEnd = Send<TcpEnd<(),Server>, End>;
type AtmFirstStep = <SendEnd as HasDual>::Dual;


fn server(end: UnixEnd<(), AtmFirstStep>){
    loop{
    let (end,atmend) = end.recv();
    end.close();
    let (atmend,number) = atmend.recv();
    if number % 2 == 0 {
        atmend.send(true).close()
    } else {
        atmend.send(false).close()
    }
}
}
    



fn id_verif(c1:TcpEnd<(),Idverif>, c2: UnixEnd<(), SendEnd>) {
    let (c1, id) = c1.recv();
    if !id == 1 {
        c1.send(false).close();
        return;
    }else{
    c1.send(true).close();
    }
    let socket = tcp_server_8081();
    for stream in socket.incoming(){
        match stream {
            Err(e) => eprintln!("{}",e),
            Ok(stream) => {
                std::thread::spawn(move || {
                    let atmend: TcpEnd<(), Server> = TcpEnd(stream,PhantomData);
                    c2.send(atmend).close();
                });

            }
        }
    }
}

fn client(c:TcpEnd<(), CliFirstStep>) {
    let c = c.send(1);
    let (c,result) = c.recv();
    c.close();
    if result{
        thread::sleep(Duration::from_millis(5));
        let cli:TcpEnd<(), Client> = st_tcp_client8081();
        let n = 8080;
        let cli = cli.send(n);
    
        let(c,b) = cli.recv();
        c.close();
        if b {
            println!("{} is even", n);
        } else {
            println!("{} is odd", n);
        }
    
    }
}

fn main() {
    let start = Instant::now();
    let a = thread::spawn(move ||  {
        let socket = unix_server();
        for stream in socket.incoming(){
            match stream {
                Err(e) => eprintln!("err: {}",e),
                Ok(stream) => {
                    thread::spawn(move || {
                        let unixsock1: UnixEnd<(), AtmFirstStep> = UnixEnd(stream, PhantomData);
                        server(unixsock1);
                    });

                }
            }
        }
    });

    let b = thread::spawn(move ||  {
        let socket = tcp_server_8080();
        for stream in socket.incoming(){
            match stream {
                Err(e) => eprintln!("err: {}",e),
                Ok(stream) => {
                    thread::spawn(move || {
                        let tcpsock2: TcpEnd<(), Idverif> = TcpEnd(stream, PhantomData);
                        let unixsock2: UnixEnd<(), SendEnd> = st_unix_client();
                        id_verif(tcpsock2,unixsock2);
                    });
                }
            }
        }
    });
let c = thread::spawn(move ||  {
    
    for i in 1..2{
    let tcpsock1: TcpEnd<(),CliFirstStep> = st_tcp_client8080();
     client(tcpsock1);
    }
     println!("time cost: {:?} ms", start.elapsed().as_millis());
    
});
    let _ = (c.join(), b.join(),a.join());
}



extern crate liu_takano_session_types;
use std::time::Duration;
use std::marker::PhantomData;
use liu_takano_session_types::*;
extern crate serde_derive;
extern crate serde;
use std::thread;
use std::time::Instant; 

type Id = String;
type Atm = Choose<Offer<AtmDeposit, Offer<AtmWithdraw, End>>,End>;
type AtmDeposit = Recv<u64, Send<u64, End>>;
type AtmWithdraw = Recv<u64, Choose<End, End>>;

type Client = <Atm as HasDual>::Dual;

type CliFirstStep = Send<Id,Recv<bool,End>>;
type Idverif = <CliFirstStep as HasDual>::Dual;

type SendEnd = Send<TcpEnd<(),Atm>, End>;
type AtmFirstStep = <SendEnd as HasDual>::Dual;


fn approved(id: &Id) -> bool {
    !id.is_empty()
}

fn atm(end: UnixEnd<(), AtmFirstStep>){
    loop{
    let (end,atmend) = end.recv();
    end.close();
    let atmend= atmend.sel_l();
    let balance = 1000;
    loop{
        match atmend.offer(){
            Left(atmend) => {
                let (atmend, mut value) = atmend.recv();
                value += balance;
                atmend.send(value).close();
                println!("despoit successd!");
                break
            },
            Right(atmend) => {
                match atmend.offer(){
                    Left(atmend) => {
                        let (atmend, value) = atmend.recv();
                        if value > balance{
                            atmend.sel_l().close();
                            break
                        }else{
                            atmend.sel_r().close();
                            println!("withdraw successd!");
                            break
                        }
                    },
                    Right(atmend) => {
                        atmend.close();
                        break
                    },
                };
            },
        };
    }
    
    }
}
    


fn client(c:TcpEnd<(), CliFirstStep>) {
    let (c,result) = c.send("Deposit Client".to_string()).recv();
    c.close();
    if !result {
        println!("[Deposit Client]: Login Error");
    }
        println!("[Deposit Client]: Login Success");
        thread::sleep(Duration::from_millis(5));
        let client_end:TcpEnd<(), Client> = st_tcp_client8081();
         match client_end.offer(){
        Left(client_end) => {
            let (client_end,value) = client_end.sel_l().send(200).recv();
            client_end.close();
            println!("balance:{}",value);
            
        },
        Right(_) =>{
            panic!("depoist error!");
        },
    };
    
}

fn id_verif(c1:TcpEnd<(),Idverif>, c2: UnixEnd<(), SendEnd>) {
    let (c1, id) = c1.recv();
    if !approved(&id) {
        c1.send(false).close();   
    }
    c1.send(true).close();
    let socket = tcp_server_8081();
    for stream in socket.incoming(){
        match stream {
            Err(e) => eprintln!("{}",e),
            Ok(stream) => {
                std::thread::spawn(move || {
                    let atmend: TcpEnd<(), Atm> = TcpEnd(stream,PhantomData);
                    c2.send(atmend).close();
                });

            }
        }
    }
}


fn main() {
    let start = Instant::now();
    let socket = unix_server();
    let a = thread::spawn(move ||  {
        for stream in socket.incoming(){
            match stream {
                Err(e) => eprintln!("err: {}",e),
                Ok(stream) => {
                    thread::spawn(move || {
                        let unixsock1: UnixEnd<(), AtmFirstStep> = UnixEnd(stream, PhantomData);
                        atm(unixsock1);
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
    let tcpsock1: TcpEnd<(),CliFirstStep> = st_tcp_client8080();
     client(tcpsock1);
});


    let _ = (c.join(), b.join(),a.join());
}

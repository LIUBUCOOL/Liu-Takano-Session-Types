
extern crate liu_takano_session_types;
use std::time::Duration;
use std::marker::PhantomData;
use liu_takano_session_types::*;
extern crate serde_derive;
extern crate serde;
use std::thread;
 

type Id = String;
type Atm = Choose<Rec<Offer<AtmDeposit, Offer<AtmWithdraw, Offer<AtmBalance, End>>>>,End>;
type AtmDeposit = Recv<u64, Send<u64, Var<Z>>>;
type AtmWithdraw = Recv<u64, Choose<Var<Z>, Var<Z>>>;
type AtmBalance = Send<u64, Var<Z>>;
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
    let mut atmend= atmend.sel_l().enter();
    let mut balance = 1000;
    loop {
        atmend = offer! {
            atmend,
            Deposit => {
                let (atmend, amt) = atmend.recv();
                balance += amt;
                println!("Desposit succeed");
                atmend.send(balance).zero()
            },
            Withdraw => {
                let (atmend, amt) = atmend.recv();
                if amt > balance {
                    atmend.sel_r().zero()
                } else {
                    balance -= amt;
                    println!("Withdraw succeed");
                    atmend.sel_l().zero()
                }
            },
            Balance => {
                atmend.send(balance).zero()
            },
            Quit => {
                atmend.close();
                break
            }
        }
            }
        }}
    

fn client(c:TcpEnd<(), CliFirstStep>) {
    let (c,result) = c.send("Deposit Client".to_string()).recv();
    c.close();
    if result{
        println!("[Deposit Client]: Login Success");
        thread::sleep(Duration::from_millis(5));
    let client_end:TcpEnd<(), Client> = st_tcp_client8081();
    let client_end = match client_end.offer() {
        Left(client_end) => client_end.enter(),
        Right(_) => panic!("[Deposit_client]: expected to be approved"),
    };
    let (client_end, new_balance) = client_end.sel_l().send(200).recv();
    println!("[Deposit_client]: new balance: {}", new_balance);
    client_end.zero().skip3().close();
    
    }
}

fn id_verif(c1:TcpEnd<(),Idverif>, c2: UnixEnd<(), SendEnd>) {
    let (c1, id) = c1.recv();
    if !approved(&id) {
  //if id.is_empty() {
        c1.send(false).close();
        
    }
    println!("[Server]: Right ID, Welcome!");
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



// fn client2(c:TcpEnd<(), CliFirstStep>) {
//     let (c,result) = c.send("Withdraw Client".to_string()).recv();
//     c.close();
//     if result{
//         println!("[Withdraw Client]: Login Success");
//         thread::sleep(Duration::from_millis(5));
//         let client_end:TcpEnd<(), Client> = st_tcp_client8081();
//         let client_end = match client_end.offer() {
//             Left(client_end) => client_end.enter(),
//             Right(_) => panic!("[Client]: Withdraw_client: expected to be approved"),
//         };

//         match client_end.sel_r().sel_l().send(300).offer() {
//             Left(c) => {
//                 println!("[withdraw Client]: withdraw_client: Successfully withdrew 300");
//                 c.zero().skip3().close();
//             }
//             Right(c) => {
//                 println!("[withdraw Client]: withdraw_client: Could not withdraw. Depositing instead.");
//                 c.zero().sel_l().send(0).recv().0.zero().skip3().close();
//             }
//         }
//     }else{
//         println!("err id");
//     }
// }

fn main() {
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
        let unixsock2: UnixEnd<(), SendEnd> = st_unix_client();
        let socket = tcp_server_8080();
        for stream in socket.incoming(){
            match stream {
                Err(e) => eprintln!("err: {}",e),
                Ok(stream) => {
                    thread::spawn(move || {
                        let tcpsock2: TcpEnd<(), Idverif> = TcpEnd(stream, PhantomData);
                        
                        id_verif(tcpsock2,unixsock2);
                    });
                }
            }
        }
    });

    // let c = thread::spawn(move ||  {
    //     let tcpsock1: TcpEnd<(),CliFirstStep> = st_tcp_client8080();
    //     client(tcpsock1);
    // });
    
    // let d = thread::spawn(move ||  {
    //     let tcpsock1: TcpEnd<(),CliFirstStep> = st_tcp_client8080();
    //     client2(tcpsock1);
    // });

//     let c = thread::spawn(move ||  {
//     let mut count = 0;
//     for i in 1..100{
//     let tcpsock1: TcpEnd<(),CliFirstStep> = st_tcp_client8080();
//      client(tcpsock1);
//      count += 1;
//      println!("{}",count)
//     }
//      println!("time cost: {:?} ms", start.elapsed().as_millis());
    
// });
let c = thread::spawn(move ||  {
    let tcpsock1: TcpEnd<(),CliFirstStep> = st_tcp_client8080();
     client(tcpsock1);
    }
    
);


    let _ = (c.join(), b.join(),a.join());
}


extern crate liu_takano_session_types;
extern crate unix_socket;
use std::marker::PhantomData;
use unix_socket::UnixSocket;
use liu_takano_session_types::*;
use std::thread;
type Id = String;
type Atm = Recv<Id, Choose<Rec<AtmInner>, End>>;
type AtmInner = Offer<AtmDeposit, Offer<AtmWithdraw, Offer<AtmBalance, End>>>;
type AtmDeposit = Recv<u64, Send<u64, Var<Z>>>;
type AtmWithdraw = Recv<u64, Choose<Var<Z>, Var<Z>>>;
type AtmBalance = Send<u64, Var<Z>>;
type Client = <Atm as HasDual>::Dual;

fn approved(id: &Id) -> bool {
    !id.is_empty()
}

fn atm(server: UnixEnd<(), Atm>) {
    let mut server = {
        let (server, id) = server.recv();
        if !approved(&id) {
            server.sel_r().close();
            return;
        }
        println!("[ATM]: Right ID, Welcome!");
        server.sel_l().enter()
    };
    let mut balance = 1000;
    loop {
        server = offer! {
            server,
            Deposit => {
                let (server, amt) = server.recv();
                balance += amt;
                println!("[ATM]: Desposit succeed");
                server.send(balance).zero()
            },
            Withdraw => {
                let (server, amt) = server.recv();
                if amt > balance {
                    server.sel_r().zero()
                } else {
                    balance -= amt;
                    println!("[ATM]: Withdraw succeed");
                    server.sel_l().zero()
                }
            },
            Balance => {
                server.send(balance).zero()
            },
            Quit => {
                server.close();
                break
            }
        }
    }
}
fn main() {
    let socket:UnixSocket= unix_server();
    println!("[ATM]: Listening...");
    for stream in socket.incoming() {
        match stream{
            Err(e) => eprintln!("stream err: {}",e),
            Ok(stream) => {
                std::thread::spawn(move || {
                    let server:UnixEnd<(),Atm> = UnixEnd(stream, PhantomData);
                    let srv_t = thread::spawn(move || atm(server));
                    let _ = srv_t.join();
                });
            }
        }
    }
}
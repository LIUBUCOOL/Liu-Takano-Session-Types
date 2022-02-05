
extern crate liu_takano_session_types;
extern crate unix_socket;
use liu_takano_session_types::*;
use std::thread;
type Id = String;
type Atm = Recv<Id, Choose<Rec<AtmInner>, End>>;
type AtmInner = Offer<AtmDeposit, Offer<AtmWithdraw, Offer<AtmBalance, End>>>;
type AtmDeposit = Recv<u64, Send<u64, Var<Z>>>;
type AtmWithdraw = Recv<u64, Choose<Var<Z>, Var<Z>>>;
type AtmBalance = Send<u64, Var<Z>>;
type Client = <Atm as HasDual>::Dual;


fn deposit_client(c: TcpEnd<(), Client>) {
    let c = match c.send("Deposit Client".to_string()).offer() {
        Left(c) => c.enter(),
        Right(_) => panic!("deposit_client: expected to be approved"),
    };

    let (c, new_balance) = c.sel_l().send(200).recv();
    println!("deposit_client: new balance: {}", new_balance);
    c.zero().skip3().close();
}

fn withdraw_client(c: TcpEnd<(), Client>) {
    let c = match c.send("Withdraw Client".to_string()).offer() {
        Left(c) => c.enter(),
        Right(_) => panic!("withdraw_client: expected to be approved"),
    };

    match c.sel_r().sel_l().send(300).offer() {
        Left(c) => {
            println!("withdraw_client: Successfully withdrew 300");
            c.zero().skip3().close();
        }
        Right(c) => {
            println!("withdraw_client: Could not withdraw. Depositing instead.");
            c.zero().sel_l().send(0).recv().0.zero().skip3().close();
        }
    }
}
fn main() {
    
    // let cli = st_tcp_client8080();
    // let client = thread::spawn(move || {deposit_client(cli);});
    // let _ = client.join();
    let cli2 = st_tcp_client8080();
    let client2 = thread::spawn(move || {withdraw_client(cli2);});
    let _ = client2.join();

}
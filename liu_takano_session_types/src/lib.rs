extern crate bincode;
extern crate serde;
extern crate serde_derive;

use libc as c;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use std::path::Path;
use std::io::Read;
use std::io::Write;
use std::io::BufWriter;
use std::io::BufReader;
use unix_socket::*;
use std::marker::PhantomData;
use std::marker;
use std::fs;
pub use Branch::{Left, Right};


#[must_use]
#[derive(Serialize, Deserialize, Debug,Copy,Clone)]
pub struct UnixEnd<E, P>(pub UnixSocket, pub PhantomData<(E, P)>);
#[must_use]
#[derive(Serialize, Deserialize, Debug,Copy,Clone)]
pub struct TcpEnd<E, P>(pub Socket, pub PhantomData<(E, P)>);



unsafe impl<E: marker::Send, P: marker::Send> marker::Send for UnixEnd<E, P> {}
unsafe impl<E: marker::Send, P: marker::Send> marker::Send for TcpEnd<E, P> {}


pub fn st_unix_client<P>()-> UnixEnd<(),P> {
    let socket = UnixSocket::new(c::SOCK_STREAM, 0).unwrap();
    let socket_path = Path::new("unixsocket");
    socket.connect(socket_path).unwrap();
    let c1 = UnixEnd(socket,PhantomData);
    c1
}

pub fn st_tcp_client8080<P>()-> TcpEnd<(),P> {
    let socket = Socket::new(c::AF_INET,c::SOCK_STREAM, 0).unwrap();
    socket.connect("127.0.0.1:8080").unwrap();
    let c1 = TcpEnd(socket,PhantomData);
    c1
}
pub fn st_tcp_client8081<P>()-> TcpEnd<(),P> {
    let socket = Socket::new(c::AF_INET,c::SOCK_STREAM, 0).unwrap();
    socket.connect("127.0.0.1:8081").unwrap();
    let c1 = TcpEnd(socket,PhantomData);
    c1
}


pub fn tcp_server_8080() -> Socket{
    let socket = Socket::new(c::AF_INET, c::SOCK_STREAM, 0).unwrap();
    socket.bind("127.0.0.1:8080").unwrap();
    socket.listen(128).unwrap();
    socket
} 
pub fn tcp_client_8080() -> Socket{
    let socket = Socket::new(c::AF_INET, c::SOCK_STREAM, 0).unwrap();
    socket.connect("127.0.0.1:8080").unwrap();
    socket
}
pub fn tcp_server_8081() -> Socket{
    let socket = Socket::new(c::AF_INET, c::SOCK_STREAM, 0).unwrap();
    socket.bind("127.0.0.1:8081").unwrap();
    socket.listen(128).unwrap();
    socket
} 
pub fn tcp_client_8081() -> Socket{
    let socket = Socket::new(c::AF_INET, c::SOCK_STREAM, 0).unwrap();
    socket.connect("127.0.0.1:8081").unwrap();
    socket
}

pub fn unix_server() -> UnixSocket{
    let socket = UnixSocket::new(c::SOCK_STREAM, 0).unwrap();
    let socket_path = Path::new("unixsocket");
    if socket_path.exists(){
        fs::remove_file(socket_path).unwrap();
    }
    socket.bind(socket_path).unwrap();
    socket.listen(128).unwrap();
    socket
} 
pub fn unix_client() -> UnixSocket{
    let socket = UnixSocket::new(c::SOCK_STREAM, 0).unwrap();
    let socket_path = Path::new("unixsocket");
    socket.connect(socket_path).unwrap();
    socket
}

//unix
unsafe fn write_unix_chan<A: marker::Send + 'static + serde::Serialize + DeserializeOwned, E, P>(&UnixEnd(sender, _): &UnixEnd<E, P>, x: A) {
    let buffer: Vec<u8> = bincode::serialize(&x).unwrap();
    let mut writer = BufWriter::new(sender);
    Write::write_all(&mut writer,&buffer).unwrap();
    Write::flush(&mut writer).unwrap();
   
}

unsafe fn read_unix_chan< A: DeserializeOwned + marker::Send + 'static, E, P>(&UnixEnd(receiver, _): &UnixEnd<E, P>) -> A {
    let mut buffer:Vec<u8>  = [0u8;1024].to_vec();
    let mut reader = BufReader::new(receiver);
    Read::read(&mut reader ,&mut buffer).unwrap();
    let message:A = bincode::deserialize(&mut buffer).unwrap();
    message
}


//tcp
unsafe fn write_tcp_chan<A: marker::Send + 'static + serde::Serialize + std::fmt::Display+DeserializeOwned , E, P>(&TcpEnd(sender, _): &TcpEnd<E, P>, x: A) {
    let buffer: Vec<u8> = bincode::serialize(&x).unwrap();
    let mut writer = BufWriter::new(sender);
    Write::write_all( &mut writer,&buffer).unwrap();
    Write::flush(&mut writer).unwrap();
}

unsafe fn read_tcp_chan< A: DeserializeOwned + marker::Send + 'static + std::fmt::Debug , E, P>(&TcpEnd(receiver, _): &TcpEnd<E, P>) -> A {
    let mut buffer:Vec<u8>  = [0u8;1024].to_vec();
    let mut reader = BufReader::new(receiver);
    Read::read(&mut reader ,&mut buffer).unwrap();
    let message:A = bincode::deserialize(&mut buffer).unwrap();
    message
}





/// Peano numbers: Zero
#[allow(missing_copy_implementations)]

#[derive(Serialize, Deserialize, Debug, Copy ,Clone)]
pub struct Z;

/// Peano numbers: Increment
#[derive(Serialize, Deserialize, Debug, Copy ,Clone)]
pub struct S<N>(PhantomData<N>);

/// End of communication session (epsilon)
#[allow(missing_copy_implementations)]  
#[derive(Serialize, Deserialize, Debug, Copy ,Clone)]
pub struct End;

/// Receive `A`, then `P`
#[derive(Serialize, Deserialize, Debug, Copy ,Clone)]
pub struct Recv<A, P>(PhantomData<(A, P)>);

/// Send `A`, then `P`
#[derive(Serialize, Deserialize, Debug, Copy ,Clone)]
pub struct Send<A, P>(pub PhantomData<(A, P)>);

/// Active choice between `P` and `Q`
#[derive(Serialize, Deserialize, Debug, Copy ,Clone)]
pub struct Choose<P, Q>(PhantomData<(P, Q)>);

/// Passive choice (offer) between `P` and `Q`
#[derive(Serialize, Deserialize, Debug, Copy ,Clone)]
pub struct Offer<P, Q>(PhantomData<(P, Q)>);

/// Enter a recursive environment
#[derive(Serialize, Deserialize, Debug, Copy ,Clone)]
pub struct Rec<P>(PhantomData<P>);

/// Recurse. N indicates how many layers of the recursive environment we recurse
/// out of.
#[derive(Serialize, Deserialize, Debug, Copy ,Clone)]
pub struct Var<N>(PhantomData<N>);

/// The HasDual trait defines the dual relationship between protocols.
///
/// Any valid protocol has a corresponding dual.
///
/// This trait is sealed and cannot be implemented outside of session-types
pub trait HasDual: private::Sealed {
    type Dual;
}

impl HasDual for End {
    type Dual = End;
}

impl<A, P: HasDual> HasDual for Send<A, P> {
    type Dual = Recv<A, P::Dual>;
}

impl<A, P: HasDual> HasDual for Recv<A, P> {
    type Dual = Send<A, P::Dual>;
}

impl<P: HasDual, Q: HasDual> HasDual for Choose<P, Q> {
    type Dual = Offer<P::Dual, Q::Dual>;
}

impl<P: HasDual, Q: HasDual> HasDual for Offer<P, Q> {
    type Dual = Choose<P::Dual, Q::Dual>;
}

impl HasDual for Var<Z> {
    type Dual = Var<Z>;
}

impl<N> HasDual for Var<S<N>> {
    type Dual = Var<S<N>>;
}

impl<P: HasDual> HasDual for Rec<P> {
    type Dual = Rec<P::Dual>;
}

pub enum Branch<L, R> {
    Left(L),
    Right(R),
}

// impl<E, P> Drop for UnixEnd<E, P> {
//     fn drop(&mut self) {
//         panic!("Unix Session channel prematurely dropped");
//     }
// }
// impl<E, P> Drop for TcpEnd<E, P> {
//     fn drop(&mut self) {
//         panic!("Tcp Session channel prematurely dropped");
//     }
// }

impl<E> UnixEnd<E, End> {
    pub fn close(self) {
        drop(self); // drop them
    }
}

impl<E> TcpEnd<E, End> {
    pub fn close(self) {
        drop(self);// drop them
    }
}

impl<E, P> UnixEnd<E, P> {
    unsafe fn cast<E2, P2>(self) -> UnixEnd<E2, P2> {
        UnixEnd(self.0,PhantomData)
    }
}

impl<E, P> TcpEnd<E, P> {
    unsafe fn cast<E2, P2>(self) -> TcpEnd<E2, P2> {
        TcpEnd(self.0, PhantomData)
    }
}

impl<E, P, A: marker::Send + 'static + serde::Serialize + DeserializeOwned > UnixEnd<E, Send<A, P>> {
    /// Send a value of type `A` over the channel. Returns a channel with
    /// protocol `P`
    #[must_use]
    pub fn send(self, v: A) -> UnixEnd<E, P> {
        unsafe {
            write_unix_chan(&self, v);
            self.cast()
        }
    }
}

impl<E, P, A: marker::Send + 'static + serde::Serialize + std::fmt::Display + DeserializeOwned > TcpEnd<E, Send<A, P>> {
    #[must_use]
    pub fn send(self, v: A) -> TcpEnd<E, P> {
        unsafe {
            write_tcp_chan(&self, v);
            self.cast()
        }
    }
}


impl<A: DeserializeOwned+marker::Send + 'static ,E, P> UnixEnd<E, Recv<A, P>> {
    /// Receives a value of type `A` from the channel. Returns a tuple
    /// containing the resulting channel and the received value.
    #[must_use]
    pub fn recv(self) -> (UnixEnd<E, P>, A) {
        unsafe {
            let v = read_unix_chan(&self);
            (self.cast(), v)
        }
    }
}

impl<A: DeserializeOwned+marker::Send + 'static + std::fmt::Debug,E, P> TcpEnd<E, Recv<A, P>> {
    /// Receives a value of type `A` from the channel. Returns a tuple
    /// containing the resulting channel and the received value.
    #[must_use]
    pub fn recv(self) -> (TcpEnd<E, P>, A) {
        unsafe {
            let v = read_tcp_chan(&self);
            (self.cast(), v)
        }
    }
}


impl<E, P, Q> UnixEnd<E, Choose<P, Q>> {
    /// Perform an active choice, selecting protocol `P`.
    #[must_use]
    pub fn sel_l(self) -> UnixEnd<E, P> {
        unsafe {
            write_unix_chan(&self, true);
            self.cast()
        }
    }

    /// Perform an active choice, selecting protocol `Q`.
    #[must_use]
    pub fn sel_r(self) -> UnixEnd<E, Q> {
        unsafe {
            write_unix_chan(&self, false);
            self.cast()
        }
    }
}

impl<E, P, Q> TcpEnd<E, Choose<P, Q>> {
    /// Perform an active choice, selecting protocol `P`.
    #[must_use]
    pub fn sel_l(self) -> TcpEnd<E, P> {
        unsafe {
            write_tcp_chan(&self, true);
            self.cast()
        }
    }

    /// Perform an active choice, selecting protocol `Q`.
    #[must_use]
    pub fn sel_r(self) -> TcpEnd<E, Q> {
        unsafe {
            write_tcp_chan(&self, false);
            self.cast()
        }
    }
}



/// Convenience function. This is identical to `.sel_r()`
impl<Z, A, B> UnixEnd<Z, Choose<A, B>> {
    #[must_use]
    pub fn skip(self) -> UnixEnd<Z, B> {
        self.sel_r()
    }
}

impl<Z, A, B> TcpEnd<Z, Choose<A, B>> {
    #[must_use]
    pub fn skip(self) -> TcpEnd<Z, B> {
        self.sel_r()
    }
}


/// Convenience function. This is identical to `.sel_r().sel_r()`
impl<Z, A, B, C> UnixEnd<Z, Choose<A, Choose<B, C>>> {
    #[must_use]
    pub fn skip2(self) -> UnixEnd<Z, C> {
        self.sel_r().sel_r()
    }
}

impl<Z, A, B, C> TcpEnd<Z, Choose<A, Choose<B, C>>> {
    #[must_use]
    pub fn skip2(self) -> TcpEnd<Z, C> {
        self.sel_r().sel_r()
    }
}

/// Convenience function. This is identical to `.sel_r().sel_r().sel_r()`
impl<Z, A, B, C, D> UnixEnd<Z, Choose<A, Choose<B, Choose<C, D>>>> {
    #[must_use]
    pub fn skip3(self) -> UnixEnd<Z, D> {
        self.sel_r().sel_r().sel_r()
    }
}

impl<Z, A, B, C, D> TcpEnd<Z, Choose<A, Choose<B, Choose<C, D>>>> {
    #[must_use]
    pub fn skip3(self) -> TcpEnd<Z, D> {
        self.sel_r().sel_r().sel_r()
    }
}

impl<E, P, Q> UnixEnd<E, Offer<P, Q>> {
    /// Passive choice. This allows the other end of the channel to select one
    /// of two options for continuing the protocol: either `P` or `Q`.
    #[must_use]
    pub fn offer(self) -> Branch<UnixEnd<E, P>, UnixEnd<E, Q>> {
        unsafe {
            let b = read_unix_chan(&self);
            if b {
                Left(self.cast())
            } else {
                Right(self.cast())
            }
        }
    }
}

impl<E, P, Q> TcpEnd<E, Offer<P, Q>> {
    /// Passive choice. This allows the other end of the channel to select one
    /// of two options for continuing the protocol: either `P` or `Q`.
    #[must_use]
    pub fn offer(self) -> Branch<TcpEnd<E, P>, TcpEnd<E, Q>> {
        unsafe {
            let b = read_tcp_chan(&self);
            if b {
                Left(self.cast())
            } else {
                Right(self.cast())
            }
        }
    }
}


impl<E, P> UnixEnd<E, Rec<P>> {
    /// Enter a recursive environment, putting the current environment on the
    /// top of the environment stack.
    #[must_use]
    pub fn enter(self) -> UnixEnd<(P, E), P> {
        unsafe { self.cast() }
    }
}

impl<E, P> TcpEnd<E, Rec<P>> {
    /// Enter a recursive environment, putting the current environment on the
    /// top of the environment stack.
    #[must_use]
    pub fn enter(self) -> TcpEnd<(P, E), P> {
        unsafe { self.cast() }
    }
}


impl<E, P> UnixEnd<(P, E), Var<Z>> {
    /// Recurse to the environment on the top of the environment stack.
    #[must_use]
    pub fn zero(self) -> UnixEnd<(P, E), P> {
        unsafe { self.cast() }
    }
}

impl<E, P> TcpEnd<(P, E), Var<Z>> {
    /// Recurse to the environment on the top of the environment stack.
    #[must_use]
    pub fn zero(self) -> TcpEnd<(P, E), P> {
        unsafe { self.cast() }
    }
}


impl<E, P, N> UnixEnd<(P, E), Var<S<N>>> {
    /// Pop the top environment from the environment stack.
    #[must_use]
    pub fn succ(self) -> UnixEnd<E, Var<N>> {
        unsafe { self.cast() }
    }
}

impl<E, P, N> TcpEnd<(P, E), Var<S<N>>> {
    /// Pop the top environment from the environment stack.
    #[must_use]
    pub fn succ(self) -> TcpEnd<E, Var<N>> {
        unsafe { self.cast() }
    }
}



mod private {
    use super::*;
    pub trait Sealed {}

    // Impl for all exported protocol types
    impl Sealed for End {}
    impl<A, P> Sealed for Send<A, P> {}
    impl<A, P> Sealed for Recv<A, P> {}
    impl<P, Q> Sealed for Choose<P, Q> {}
    impl<P, Q> Sealed for Offer<P, Q> {}
    impl<Z> Sealed for Var<Z> {}
    impl<P> Sealed for Rec<P> {}
}


#[macro_export]
macro_rules! offer {
    (
        $id:ident, $branch:ident => $code:expr, $($t:tt)+
    ) => (
        match $id.offer() {
            $crate::Left($id) => $code,
            $crate::Right($id) => offer!{ $id, $($t)+ }
        }
    );
    (
        $id:ident, $branch:ident => $code:expr
    ) => (
        $code
    )
}

/// Returns the channel unchanged on `TryRecvError::Empty`.
#[macro_export]
macro_rules! try_offer {
    (
        $id:ident, $branch:ident => $code:expr, $($t:tt)+
    ) => (
        match $id.try_offer() {
            Ok($crate::Left($id)) => $code,
            Ok($crate::Right($id)) => try_offer!{ $id, $($t)+ },
            Err($id) => Err($id)
        }
    );
    (
        $id:ident, $branch:ident => $code:expr
    ) => (
        $code
    )
}


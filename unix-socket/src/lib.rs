use std::io::BufReader;
use std::io::BufWriter;
use std::time::Duration;
use serde::de::DeserializeOwned;
use libc as c;
extern crate serde;
extern crate serde_derive;
extern crate num;
use std::{ptr, cmp, mem};
use std::marker;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::io::{Result,Error,ErrorKind};
use std::cmp::Ordering;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::{RawFd, AsRawFd, FromRawFd, IntoRawFd};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4,Shutdown,ToSocketAddrs};
use std::iter::FromIterator;
use serde::*;
use std::thread;




///This function is used to judge the return value of  functions
/// If return value less than 0 -> Error
pub fn judgment(val:c::c_int) -> Result<c::c_int>{
    if val < 0 {
        Err(Error::last_os_error())
    }else{
        Ok(val)
    }
}
///Used to judge if the buffer length is exceeded when RD or WR
pub fn max_len() -> usize {
    <c::ssize_t>::max_value() as usize
}

#[derive(Debug,Clone,Copy,Serialize, Deserialize,)]

//Server socket:fd
pub struct Socket{
    socket:RawFd,
}

// impl Drop for Socket {
//      fn drop(&mut self){
//         unsafe{
//             c::close(self.socket)
//         };
//     }
// }


#[derive(Debug,Clone,Copy)]
///An iterator
pub struct Incoming<'a> {
    server: &'a Socket,
}
impl<'a> Iterator for Incoming<'a> {
    type Item = Result<Socket>;
    fn next(&mut self) -> Option<Result<Socket>> {
        Some(self.server.accept().map(|p| p.0))
    }
}

impl Socket {
    ///Creat a new socket,return new socket file descriptor
    pub fn new(sock_domain:i32, sock_types:i32, sock_protocol:i32) -> Result<Socket> {
        unsafe{
        let sock_fd = c::socket(sock_domain, sock_types, sock_protocol);
        judgment(sock_fd).unwrap();
        Ok(Socket{socket:sock_fd})
        }  
    }
    ///Shut down read or write or both at same time by shutdown function.
    /// fdへの読み取りまたは書き込みをオフにするために使用されます。
    pub fn shut_down(&self, mode: Shutdown) -> Result<()> {
        let mode = match mode {
            Shutdown::Read => c::SHUT_RD,
            Shutdown::Write => c::SHUT_WR,
            Shutdown::Both => c::SHUT_RDWR,
        };

        unsafe{
        let a = c::shutdown(self.socket, mode);
        judgment(a).map(|_| ())
        }
    }


    ///Set socket option
    pub fn setsockopt<T>(&self, level:i32, option:i32, value: T) -> Result<()>{
        unsafe{
            let value = &value as *const _ as *const c::c_void;
            c::setsockopt(self.socket, level, option, value, mem::size_of::<T>() as c::socklen_t);
        }
        Ok(())
    }
    ///Get socket option
    pub fn getsockopt<T>(&self, level:i32, option:i32, value: T) -> Result<()>{
        unsafe{
            let value = &value as *const _ as *mut c::c_void;
            c::getsockopt(self.socket, level, option, value, mem::size_of::<T>() as *mut c::socklen_t);
        }
        Ok(())
    }
    
    ///Bind a address to socketfd
    ///IPv4のsocket addressをネットワーク汎用のsocketaddressに変換し、それをfdにバインドする
    pub fn bind<T>(&self,address:&T) -> Result<()>
        where T:ToSocketAddrs + ?Sized {
        let addr = tosocketaddrs_to_sockaddr(address).unwrap();
        unsafe{
            c::bind(self.socket, &addr, num::cast(mem::size_of::<c::sockaddr>()).unwrap());
        }
        Ok(())
    }  
    ///Server can listen the connect
    /// サーバがクライアントからの接続を fd で待ち受ける。
    pub fn listen(&self,backlog:i32) -> Result<()> {
        unsafe{
            let result = c::listen(self.socket, backlog);
            judgment(result).unwrap();
        }
        Ok(())
    }


    //Connect other host
    // Generally required on the client side, but we have also implemented on the server side
    pub fn connect<T>(&self,toaddress:&T) -> Result<()> 
        where T: ToSocketAddrs + ?Sized {
      let address = (tosocketaddrs_to_sockaddr(toaddress)).unwrap();
      unsafe{
        let result = c::connect(self.socket, 
                        &address as *const c::sockaddr, 
        num::cast(mem::size_of::<c::sockaddr>()).unwrap());
        if result < 0 {
          panic!("faild to connect");
        }else{
          Ok(())
        }
      }
    }

    ///Server accept the connection from client
    /// クライアントからの接続を受ける
    pub fn accept(&self) -> Result<(Socket, SocketAddr)> {
        unsafe{
            let mut addr:c::sockaddr = mem::zeroed();
        let mut addr_len: c::socklen_t = 
              mem::size_of::<c::sockaddr>() as c::socklen_t;
            let fd = c::accept(self.socket,
               &mut addr as *mut c::sockaddr,
               &mut addr_len as *mut c::socklen_t);
        judgment(fd).unwrap();
        Ok((Socket{socket:fd},sockaddr_to_socketaddr(&addr)))
            // if fd < 0 {
            //     Err(Error::last_os_error())
            // }else{
            //     Ok((Socket{socket:fd},sockaddr_to_socketaddr(&addr)))
            // }
        }
    }
    ///Get the address of socket
    pub fn getsockname(&self) -> Result<SocketAddr> {
        let mut sa: c::sockaddr = unsafe { mem::zeroed() };
        let mut len: c::socklen_t = mem::size_of::<c::sockaddr>() as c::socklen_t;
        unsafe{
        c::getsockname(self.socket,
                &mut sa as *mut c::sockaddr, &mut len as *mut c::socklen_t);
        }
        Ok(sockaddr_to_socketaddr(&sa))
    }

    ///Returns an iterator over the connections being received on server
    pub fn incoming(&self) -> Incoming<'_>{
        Incoming{server:self}
    }
    ///Receive message 
    pub fn read<A>(&self, buf: &mut [A]) -> Result<usize> {
        let ret = unsafe {
            c::read(
                self.socket,
                buf.as_mut_ptr() as *mut c::c_void,
                cmp::min(buf.len(), max_len())
            )
        };
        judgment(ret as i32).unwrap();

        Ok(ret as usize)
    }
    ///Send message
    pub fn write<A>(&self, buf: &[A]) -> Result<usize> {
        let ret = unsafe {
            c::write(
                self.socket,
                buf.as_ptr() as *const c::c_void,
                cmp::min(buf.len(), max_len())
            )
        };
        judgment(ret as i32).unwrap();

        Ok(ret as usize)
    }

    ///receive message by recv function from C
    /// Return the bumber of bytes read
    pub fn recv<A>(&self, buf:&mut [A], flags:i32) ->Result<usize> {
        unsafe{
            let result = c::recv(
                self.socket,
                buf.as_ptr() as *mut c::c_void, 
                cmp::min(buf.len(), max_len()),
                flags);
            if result < 0 {
                Err(Error::last_os_error())
            }else{
                Ok(result as usize)
            }
        }
    }
    ///Send message by "C"send function
    /// Return the bumber of bytes send
    pub fn send<A>(&self, buf:&mut [A], flags:i32) ->Result<usize> {
        unsafe{
            let result = c::send(
                self.socket,
                buf.as_ptr() as *const c::c_void, 
                cmp::min(buf.len(), max_len()),
                flags);
            if result < 0 {
                Err(Error::last_os_error())
            }else{
                Ok(result as usize)
            }
        }
        
    }
    ///Set no blocking
    pub fn set_nonblocking(&self, mut nonblocking: bool) -> Result<()> {
        unsafe{
            let result = c::ioctl(self.socket,c::FIONBIO, &mut nonblocking);
            judgment(result).unwrap();
        }
        Ok(())
    }

    ///Clone socket fd use function dup like C.
    pub fn clone(&self) -> Result<Socket> {
        unsafe{
        let fd_clone = c::dup(self.socket);
        judgment(fd_clone).unwrap();
        Ok(Self{socket:fd_clone})
        }
    }
    ///Creat new socket pair, return double socket fd
    pub fn new_fd_pair(sock_domain:i32, sock_types:i32, sock_protocol:i32) -> Result<(RawFd,RawFd)> {
        
        let mut fd_pair = [0, 0];
        unsafe{
        let fds = c::socketpair(sock_domain, sock_types, sock_protocol, fd_pair.as_mut_ptr());
        judgment(fds).unwrap();
        Ok((fd_pair[0],fd_pair[1]))
    }
}

}
impl Read for Socket {

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        Read::read(&mut &*self, buf)
    }
    // fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
    //     Read::read_to_end(&mut &*self, buf)
    // }
}

impl<'a> Read for &'a Socket {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        unsafe {
            let result = c::recv(self.socket, buf.as_mut_ptr() as *mut _, buf.len(), 0);
            if result < 0 {
                Err(Error::last_os_error())
            }else{
                Ok(result as usize)
            }
        }
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Write::write(&mut &*self, buf)
    }

    fn flush(&mut self) -> Result<()> {
        Write::flush(&mut &*self)
    }
}

impl<'a> Write for &'a Socket {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        unsafe {
            let result = c::send(self.socket, buf.as_ptr() as *const _, buf.len(), 0);
            if result < 0 {
                Err(Error::last_os_error())
            }else{
                Ok(result as usize)
            }
        }
    }

    fn flush(&mut self) -> Result<()> {
            Ok(())
        
    }
}


impl AsRawFd for Socket{
    fn as_raw_fd(&self) ->RawFd {
        self.socket
    }
}
impl FromRawFd for Socket {
    unsafe fn from_raw_fd(socket: RawFd) -> Self {
        Self { socket }
    }
}

impl IntoRawFd for Socket {
    fn into_raw_fd(self) -> RawFd {
        let fd = self.socket;
        mem::forget(self);
        fd
    }
}

///host byte order -> NW byte order(u16)
fn htons(hostint16: u16) -> u16 {
    hostint16.to_be()
}
///NW bete order -> host byte order(u16)
fn ntohs(netint16: u16) -> u16 {
    u16::from_be(netint16)
}
#[warn(dead_code)]
///host byte order -> NW byte order(u32)
fn _htonl(hostint32: u32) -> u32 {
    hostint32.to_be()
}
#[warn(dead_code)]
///NW bete order -> host byte order(u32)
fn _ntohl(netint32: u32) -> u32 {
    u32::from_be(netint32)
}

///convert to internet socket address
pub fn socketaddr_to_sockaddr(address: &SocketAddr) -> c::sockaddr {
    unsafe {
        match *address {
            SocketAddr::V4(v4) => {
                let mut sa: c::sockaddr_in = mem::zeroed();
                sa.sin_family = c::AF_INET as c::sa_family_t;
                sa.sin_port = htons(v4.port());
                sa.sin_addr = *(&v4.ip().octets() as *const u8 as *const c::in_addr);
                *(&sa as *const c::sockaddr_in as *const c::sockaddr)
            },
            SocketAddr::V6(v6) => {//Not tested for viability
                let mut sa: c::sockaddr_in6 = mem::zeroed();
                sa.sin6_family = c::AF_INET6 as c::sa_family_t;
                sa.sin6_port = htons(v6.port());
                sa.sin6_flowinfo = v6.flowinfo();
                sa.sin6_addr = *(&v6.ip().octets() as *const u8 as *const c::in6_addr);
                sa.sin6_scope_id = v6.scope_id();
                *(&sa as *const c::sockaddr_in6 as *const c::sockaddr)
                
            },
        }
    }
}

///convert to internet socket address
pub fn sockaddr_to_socketaddr(sa: &c::sockaddr) -> SocketAddr {
    match sa.sa_family as i32 {
        c::AF_INET => {
            let sin: &c::sockaddr_in = unsafe { mem::transmute(sa) };
            let ip_parts: [u8; 4] = unsafe { mem::transmute(sin.sin_addr) };
            SocketAddr::V4(
                SocketAddrV4::new(Ipv4Addr::new(
                    ip_parts[0],
                    ip_parts[1],
                    ip_parts[2],
                    ip_parts[3],
                ),
                ntohs(sin.sin_port))
            )
        },
        _ => {
            unreachable!("！！！")
        }
    }
}

pub fn tosocketaddrs_to_socketaddr<T: ToSocketAddrs + ?Sized>(address: &T) -> Result<SocketAddr> {
    let addresses: Vec<SocketAddr> = FromIterator::from_iter((address.to_socket_addrs()).unwrap());

    match addresses.len() {
        1 => {
            Ok(addresses[0])
        },
        n => Err(Error::new(
            ErrorKind::InvalidInput,
            &format!(
                "Expected only one，but got {}", n)[..],
        ))
    }
}
fn tosocketaddrs_to_sockaddr<T: ToSocketAddrs + ?Sized>(address: &T) -> Result<c::sockaddr> {
    Ok(socketaddr_to_sockaddr(&(tosocketaddrs_to_socketaddr(&address)).unwrap()))
}






/*-----------------------------------UNIX SOCKET---------------------------------------*/


///return offset of sun_path in unix socket address

fn sun_path_offset() -> usize {
    unsafe {
        let addr_size = mem::size_of::<c::sockaddr_un>();
        let name_size = mem::size_of_val(&mem::zeroed::<c::sockaddr_un>().sun_path);
        addr_size - name_size
        
    }
}
///
fn sockaddr_un<P: AsRef<Path>>(path: P) -> Result<(c::sockaddr_un, c::socklen_t)> {
    unsafe{
    let mut address: c::sockaddr_un = mem::zeroed();
    address.sun_family = c::AF_UNIX as c::sa_family_t;

    let bytes = path.as_ref().as_os_str().as_bytes();

    match (bytes.get(0), bytes.len().cmp(&address.sun_path.len())){
        (Some(&0), Ordering::Greater) | (_, Ordering::Greater) | (_, Ordering::Equal) => {
            return Err(Error::new(ErrorKind::InvalidInput,
                "path must be no longer than SUN_LEN"));
        }
        _ => {}
    }
    //des: sun_path, src: byte
    //the pointer of address.sun_path = bytes pointer. bind.
    for (dst, src) in address.sun_path.iter_mut().zip(bytes.iter()) {
        *dst = *src as libc::c_char;
    }
    let mut len = sun_path_offset() + bytes.len();
    match bytes.get(0) {
        Some(&0) | None => {}
        Some(_) => len += 1,
    }
    Ok((address, len as libc::socklen_t))
    }
}

#[derive(Clone,Debug, Copy)]
pub struct UnSocketAddr {
    address: c::sockaddr_un,
    len: c::socklen_t,
}

impl UnSocketAddr {
    fn new<F>(f:F) -> Result<UnSocketAddr>
    where F: FnOnce(*mut c::sockaddr, *mut c::socklen_t) -> c::c_int{
        unsafe{
            let mut addr:c::sockaddr_un = mem::zeroed();
            let mut len = mem::size_of::<c::sockaddr_un>() as c::socklen_t;
            //judgment(f(&mut addr as *mut _, &mut len)).unwrap();
            judgment(f(&mut addr as *mut _ as *mut _, &mut len))?;
            if len == 0{
                len = sun_path_offset() as libc::socklen_t;
            }
            Ok(UnSocketAddr {
                address: addr,
                len: len,
            })
        }
    }
}


#[derive(Debug,Clone,Copy,Serialize, Deserialize,)]
pub struct UnixSocket {
    unixsocket: RawFd,
}

// impl Drop for UnixSocket {
//     fn drop(&mut self) {
//         unsafe{
//             c::close(self.unixsocket);
//         }
//     }
// }
impl UnixSocket{

    ///Creat a new socket,return new socket fd
    pub fn new(sock_types:i32, sock_protocol:i32) -> Result<UnixSocket> {
        unsafe{
        let unixsockserv_fd = c::socket(c::AF_UNIX, sock_types, sock_protocol);
        judgment(unixsockserv_fd).unwrap();
        Ok(UnixSocket{unixsocket:unixsockserv_fd})
        }  
    }

    ///Creates an unnamed pair of connectd sockets
    pub fn unix_pair_socket(types:i32, protocol: i32) -> Result<(UnixSocket,UnixSocket)> {
        let (fd1,fd2) = Socket::new_fd_pair(c::AF_UNIX, types, protocol).unwrap();
        Ok((UnixSocket{unixsocket:fd1}, UnixSocket{unixsocket:fd2}))
    }

    ///creates a new SOCK_STREAM unix server socket bound to path
    pub fn bind<P: AsRef<Path>>(&self, path: P) -> Result<UnixSocket> {
        unsafe{
            let (address, len) = sockaddr_un(path.as_ref()).unwrap();
            judgment(libc::bind(self.unixsocket, &address as *const _ as *const _, len)).unwrap();
            //judgment(libc::listen(self.unixsocket, 128)).unwrap();

            Ok(UnixSocket{unixsocket:self.unixsocket})
        }
    }

    ///SOCK_DGRAM socekt
    pub fn unbond(&self) -> Result<UnixSocket> {
        Ok(UnixSocket{unixsocket:self.unixsocket})
    }

    ///Server can listen the connect
    /// サーバがクライアントからの接続を fd で待ち受ける。
    pub fn listen(&self,backlog:i32) -> Result<()> {
        unsafe{
            let result = c::listen(self.unixsocket, backlog);
            judgment(result).unwrap();
        }
        Ok(())
    }

    ///client unix socket connect to server by path
    pub fn connect<P: AsRef<Path>>(&self,path: P) -> Result<UnixSocket> {
        unsafe{
            //let clientfd = Unixsocket::new(types, protocol).unwrap();
            let(address, length) = sockaddr_un(path.as_ref()).unwrap();
            let result = c::connect(self.unixsocket, &address as *const _ as *const _, length);
            judgment(result).unwrap();
            Ok(UnixSocket{unixsocket:self.unixsocket})
        }
    }
    ///accept the connection request from client
    pub fn accept(&self) -> Result<(UnixSocket, UnSocketAddr)> {
        unsafe {
            let mut fd = 0;
            let address = (UnSocketAddr::new(|addr, len| {
                fd = c::accept(self.unixsocket, addr as *mut _ as *mut _, len);
                fd
            })).unwrap();
            Ok((UnixSocket{unixsocket:fd}, address))
        }
    }

    ///clone a new socket to handle communication
    pub fn clone(self) -> Result<UnixSocket> {
        Ok(UnixSocket{unixsocket: self.unixsocket.clone()})
    }

    pub fn local_addr(&self) -> Result<UnSocketAddr> {
        UnSocketAddr::new(|addr, len| unsafe { libc::getsockname(self.unixsocket, addr, len) })
    }
    ///return peer socket address
    pub fn peer_address(&self) -> Result<UnSocketAddr> {
        let result = UnSocketAddr::new(|address, len| {
            unsafe{
                c::getpeername(self.unixsocket, address, len)
            }
        }).unwrap();
        Ok(result)
    }
    // pub fn set_nonblocking(&self, nonblocking: bool) -> Result<()> {
    //     self.set_nonblocking(nonblocking)
    // }

    pub fn incoming<'a>(&'a self) -> UnixIncoming<'a> {
        UnixIncoming{server: self}
    }
    
    pub fn recv<A>(&self, buf:&mut [A]) ->Result<usize> {
        unsafe{
        let result = 
            c::recv(self.unixsocket,
                buf.as_mut_ptr() as *mut c::c_void,
                buf.len(),
                0
            );
        
            if result < 0 {
                Err(Error::last_os_error())
            }else{
                Ok(result as usize)
            }
        }
    }

    ///receive message 
    /// Return the bumber of bytes read
    pub fn read<A>(&self, buf: &mut [A]) -> Result<usize> {
        unsafe {
            let result = c::read(
                    self.unixsocket,
                    buf.as_mut_ptr() as *mut c::c_void,
                    cmp::min(buf.len(), max_len())
                );
            
            if result < 0 {
                Err(Error::last_os_error())
            }else{
                Ok(result as usize)
            }
        }
    }
    ///Send message 
    /// Return the bumber of bytes write
    pub fn write<A>(&self, buf: &[A]) -> Result<usize> {
        unsafe {
            let result = c::write(
                    self.unixsocket,
                    buf.as_ptr() as *const c::c_void,
                    cmp::min(buf.len(), max_len())
                );
            
            if result < 0 {
                Err(Error::last_os_error())
            }else{
                Ok(result as usize)
            }
        }
    }

    pub fn recv_from<A>(&self, buf: &mut [A]) -> Result<(usize, UnSocketAddr)> {
        let mut result = 0;
        let address = UnSocketAddr::new(|addr, len| {
            unsafe{
                result = c::recvfrom(
                    self.unixsocket,
                    buf.as_mut_ptr() as *mut c::c_void,
                    buf.len(),
                    0,
                    addr,
                    len);
                if result < 0 {
                    1
                }else if result == 0{
                    0
                }else{
                    -1
                }
            }
        }).unwrap();
        Ok((result as usize, address))
    }

    pub fn send_to<P: AsRef<Path>,A>(&self, buf: &[A], path: P) -> Result<usize> {
        unsafe {
            let (addr, len) = sockaddr_un(path).unwrap();

            let result = libc::sendto(
                self.unixsocket,
                buf.as_ptr() as *const _,
                buf.len(),
                0,
                &addr as *const _ as *const _,
                len);
            if result < 0{
                Err(Error::last_os_error())
            }else{
                Ok(result as usize)
            }                                  
        }
    }

    pub fn send<A>(&self, buf: &[A]) -> Result<usize> {
        unsafe {
            let result = libc::send(
                self.unixsocket,
                buf.as_ptr() as *const c::c_void,
                buf.len(),
                0);
            if result < 0 {
                Err(Error::last_os_error())
            }else{
                Ok(result as usize)
            }
            
        }
    }
    ///send the file descriptor by unix domain sockets
    pub fn send_fd(&self, fd: RawFd) -> Result<()> {
        let mut dummy: c::c_int = 0;
        let msg_len = unsafe{
            c::CMSG_SPACE(mem::size_of::<c::c_int>() as u32) as _
        };
        let mut u = HeaderAlignedBuf{ buf: [0; 256]};
        let mut iov = c::iovec {
            iov_base: &mut dummy as *mut c::c_int as *mut c::c_void,
            iov_len: mem::size_of_val(&dummy),
        };

        let msg = c::msghdr {
            msg_name: ptr::null_mut(),
            msg_namelen: 0,
            msg_iov: &mut iov,
            msg_iovlen: 1,
            msg_control: unsafe { u.buf.as_mut_ptr() as *mut c::c_void },
            msg_controllen: msg_len,
            msg_flags: 0,
        };

        unsafe{
            let hdr = c::cmsghdr {
                cmsg_level: c::SOL_SOCKET,
                cmsg_type: c::SCM_RIGHTS,
                cmsg_len: c::CMSG_LEN(mem::size_of::<c::c_int>() as u32) as _ ,
            };
            //#[allow(clippy::cast_ptr_alignment)]
            std::ptr::write_unaligned(libc::CMSG_FIRSTHDR(&msg), hdr);
            //#[allow(clippy::cast_ptr_alignment)]
            std::ptr::write_unaligned(
                libc::CMSG_DATA(u.buf.as_mut_ptr() as *const _) as *mut c::c_int,
                fd,
            );
        }
        let rv = unsafe{c::sendmsg(self.unixsocket, &msg, 0)};
        if rv < 0 {
            return Err(Error::last_os_error());
        }
        Ok(())
    }
    ///receive the file descriptor by unix domain sockets
    pub fn recv_fd(&self) -> Result<RawFd> {
        let mut dummy: c::c_int = -1;
        let msg_len = unsafe{c::CMSG_SPACE(mem::size_of::<c::c_int>() as u32) as _};
        let mut u = HeaderAlignedBuf {buf:[0; 256]};
        let mut iov = c::iovec {
            iov_base : &mut dummy as *mut c::c_int as *mut c::c_void,
            iov_len: mem::size_of_val(&dummy),
        };
        let mut msg = c::msghdr {
            msg_name: std::ptr::null_mut(),
            msg_namelen: 0,
            msg_iov: &mut iov,
            msg_iovlen: 1,
            msg_control: unsafe { u.buf.as_mut_ptr() as *mut c::c_void },
            msg_controllen: msg_len,
            msg_flags: 0,
        };
        unsafe{
            let receive = c::recvmsg(self.unixsocket, &mut msg, 0);
            match receive {
                0 =>Err(Error::new(ErrorKind::UnexpectedEof, "0 bytes be read")),
                receive if receive < 0 => Err(Error::last_os_error()),
                receive if receive == mem::size_of::<c::c_int>() as isize => {
                    let hdr: *mut c::cmsghdr = 
                    if msg.msg_controllen >= mem::size_of::<c::cmsghdr>() as _ {
                        msg.msg_control as *mut c::cmsghdr
                    }else{
                        return Err(Error::new(ErrorKind::InvalidData, "Bad control msg(header)",));
                    };
                    if (*hdr).cmsg_level != c::SOL_SOCKET ||(*hdr).cmsg_type != c::SCM_RIGHTS{
                        return Err(Error::new(ErrorKind::InvalidData,"Bad control msg (level)",));
                    }
                    if msg.msg_controllen != c::CMSG_SPACE(mem::size_of::<c::c_int>() as u32) as _ {
                        return Err(Error::new(ErrorKind::InvalidData, "Bad control msg (len)"));
                    }
                    let fd = ptr::read_unaligned(c::CMSG_DATA(hdr) as *mut c::c_int);
                    if c::fcntl(fd, c::F_SETFD, c::FD_CLOEXEC) < 0 {
                        return Err(Error::last_os_error());
                    }
                    Ok(fd)
                }
                _ => Err(Error::new(ErrorKind::InvalidData, "Bad control msg (ret code)",)),
            }
        }
    }

    ///shutdown
    #[warn(unconditional_recursion)]
    pub fn shutdown(&self, mode:Shutdown) -> Result<()> {
        self.shutdown(mode)
    }
}


impl AsRawFd for UnixSocket { 
    fn as_raw_fd(&self) -> RawFd {
        self.unixsocket
    }
}

impl FromRawFd for UnixSocket {
    unsafe fn from_raw_fd(fd: RawFd) -> UnixSocket {
        UnixSocket { unixsocket: fd }
    }
}

impl IntoRawFd for UnixSocket {
    fn into_raw_fd(self) -> RawFd {
        let fd = self.unixsocket;
        mem::forget(self);
        fd
    }
}

impl Read for UnixSocket {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        Read::read(&mut &*self, buf)
    }
    // fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
    //     Read::read_to_end(&mut &*self, buf)
    // }
}

impl<'a> Read for &'a UnixSocket {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        unsafe {
            let result = c::recv(self.unixsocket, buf.as_mut_ptr() as *mut _, buf.len(), 0);
            if result < 0 {
                Err(Error::last_os_error())
            }else{
                Ok(result as usize)
            }
        }
    }
    
    // fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
    //     Read::read_to_end(self, buf)
        
    // }
}

impl Write for UnixSocket {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Write::write(&mut &*self, buf)
    }

    fn flush(&mut self) -> Result<()> {
        Write::flush(&mut &*self)
    }
}

impl<'a> Write for &'a UnixSocket {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        unsafe {
            let result = c::send(self.unixsocket, buf.as_ptr() as *const _, buf.len(), 0);
            if result < 0 {
                Err(Error::last_os_error())
            }else{
                Ok(result as usize)
            }
        }
    }
    fn flush(&mut self) -> Result<()> {
            Ok(())
        
    }
}

#[derive(Debug)]
pub struct UnixIncoming<'a> {
    server: &'a UnixSocket,
}
impl<'a> IntoIterator for &'a UnixSocket {
    type Item = Result<UnixSocket>;
    type IntoIter = UnixIncoming<'a>;
    fn into_iter(self) -> UnixIncoming<'a> {
        self.incoming()
    }
}
impl<'a> Iterator for UnixIncoming<'a> {
    type Item = Result<UnixSocket>;

    fn next(&mut self) -> Option<Result<UnixSocket>> {
        Some(self.server.accept().map(|s| s.0))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::max_value(), None)
    }
}

#[repr(C)]
union HeaderAlignedBuf {
    buf: [libc::c_char; 256],
    align: libc::cmsghdr,
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

unsafe fn writetcp<A: marker::Send + 'static + serde::Serialize + DeserializeOwned>(socket: Socket, x: A) {
    let buffer: Vec<u8> = bincode::serialize(&x).unwrap();
    let mut writer = BufWriter::new(socket);
    Write::write_all(&mut writer,&buffer).unwrap();
    Write::flush(&mut writer).unwrap();
    thread::sleep(Duration::from_millis(5))
}
unsafe fn writeunix<A: marker::Send + 'static + serde::Serialize + DeserializeOwned>(socket: UnixSocket, x: A) {
    let buffer: Vec<u8> = bincode::serialize(&x).unwrap();
    let mut writer = BufWriter::new(socket);
    Write::write_all(&mut writer,&buffer).unwrap();
    Write::flush(&mut writer).unwrap();
    thread::sleep(Duration::from_millis(5))
}

unsafe fn readunix< A: DeserializeOwned + marker::Send + 'static>(socket: UnixSocket) -> A {
    let mut buffer:Vec<u8>  = [0u8;1024].to_vec();
    let mut reader = BufReader::new(socket);
    Read::read(&mut reader ,&mut buffer).unwrap();
    let message:A = bincode::deserialize(&mut buffer).unwrap();
    message
}
unsafe fn readtcp< A: DeserializeOwned + marker::Send + 'static>(socket: Socket) -> A {
    let mut buffer:Vec<u8>  = [0u8;1024].to_vec();
    let mut reader = BufReader::new(socket);
    Read::read(&mut reader ,&mut buffer).unwrap();
    let message:A = bincode::deserialize(&mut buffer).unwrap();
    message
}



// Copyright 2015 The coio Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! TCP

pub use mio::tcp::Shutdown;

use std::io;
use std::iter::Iterator;
use std::net::{SocketAddr, ToSocketAddrs};

#[cfg(unix)]
use std::os::unix::io::{FromRawFd, RawFd};

use mio::Ready;
use mio::tcp::{TcpListener as MioTcpListener, TcpStream as MioTcpStream};

use super::{each_addr, GenericEvented};

macro_rules! create_tcp_listener {
    ($inner:expr) => (TcpListener::new($inner, Ready::readable()));
}

macro_rules! create_tcp_stream {
    ($inner:expr) => (TcpStream::new($inner, Ready::readable() | Ready::writable()));
}

pub type TcpListener = GenericEvented<MioTcpListener>;

impl TcpListener {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<TcpListener> {
        each_addr(addr, |addr| {
            let inner = try!(MioTcpListener::bind(addr));
            create_tcp_listener!(inner)
        })
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        match self.get_inner().accept() {
            Ok((stream, addr)) => {
                trace!("TcpListener({:?}): accept() => Ok(..)", self.token);
                return create_tcp_stream!(stream).map(|stream| (stream, addr));
            }
            Err(err) => {
                trace!("TcpListener({:?}): accept() => Err(..)", self.token);
                return Err(err);
            }
        }
    }

    pub fn try_clone(&self) -> io::Result<TcpListener> {
        let inner = try!(self.get_inner().try_clone());
        create_tcp_listener!(inner)
    }

    pub fn incoming(&self) -> Incoming {
        Incoming(self)
    }
}

#[cfg(unix)]
impl FromRawFd for TcpListener {
    unsafe fn from_raw_fd(fd: RawFd) -> TcpListener {
        let inner = FromRawFd::from_raw_fd(fd);
        create_tcp_listener!(inner).unwrap()
    }
}


pub struct Incoming<'a>(&'a TcpListener);

impl<'a> Iterator for Incoming<'a> {
    type Item = io::Result<(TcpStream, SocketAddr)>;

    fn next(&mut self) -> Option<io::Result<(TcpStream, SocketAddr)>> {
        Some(self.0.accept())
    }
}

pub type TcpStream = GenericEvented<MioTcpStream>;

impl TcpStream {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<TcpStream> {
        each_addr(addr, |addr| {
            let inner = try!(MioTcpStream::connect(addr));
            create_tcp_stream!(inner)
        })
    }

    pub fn try_clone(&self) -> io::Result<TcpStream> {
        let inner = try!(self.get_inner().try_clone());
        create_tcp_stream!(inner)
    }
}

#[cfg(unix)]
impl FromRawFd for TcpStream {
    unsafe fn from_raw_fd(fd: RawFd) -> TcpStream {
        let inner = FromRawFd::from_raw_fd(fd);
        create_tcp_stream!(inner).unwrap()
    }
}

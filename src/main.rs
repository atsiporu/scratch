#![feature(loop_break_value)]

extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate bytes;

use std::rc::Rc;
use std::net::SocketAddr;
use futures::{Stream, Future, Poll, Async};
use tokio_core::io::{read_to_end, write_all};
use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;
use tokio_io::AsyncRead;
use bytes::{BufMut, Buf, BytesMut, Bytes};
use std::io::Write;

const LISTEN_TO: &'static str = "0.0.0.0:8080";

struct BytesStream<T> {
    source: T,
    buf: Rc<BytesMut>,
}

impl<T: AsyncRead> Stream for BytesStream<T> {
    type Item = Rc<BytesMut>;
    type Error = std::io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {

        let len = {
           
            let mut buf = Rc::get_mut(&mut self.buf).unwrap() as &mut [u8];
//            let mut buf = [0u8; 1024];

            let mut read = tokio_io::io::read(&mut self.source, &mut buf[..]);
            
            let res = read.poll().unwrap();
        
            let len = match res {
                Async::NotReady => { 
                    println!("Not ready");
                    0 
                },
                Async::Ready((rd, _, len)) => {
                    println!("Ready {}", len);
                    len
                }
            };

            len
        };

        if len > 0 {
            Ok(Async::Ready(Some(self.buf.clone())))
        }
        else {
            Ok(Async::NotReady)
        }
    }
}
/*
*/


fn main() {
    // Create the event loop that will drive this server.
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // Get a SocketAddr
    let socket: SocketAddr = LISTEN_TO.parse()
        .unwrap();
    
    // Start listening.
    let listener = TcpListener::bind(&socket, &handle)
        .unwrap();
    
    // For each incoming connection...
    let server = listener.incoming().for_each(|(stream, _client_addr)| {

        let (r, mut w) = stream.split();
       
        let mut buf = BytesMut::with_capacity(5);

        let mut stream = BytesStream { source: r, buf: Rc::new(buf) };
        let receive = stream.for_each(move |mut buf| {
            w.write("Writing:".as_bytes());
            w.write(&buf.as_ref()[..]);
            //unsafe { Rc::get_mut(&mut buf).unwrap().advance_mut(1); }
            Ok(())
        })
        .or_else(|e| {
            println!("Error! {:?}", e);
            Ok(())
        });
        
        /*
        let amt = tokio_io::io::copy(r, w);

        // After our copy operation is complete we just print out some helpful
        // information.
        let receive = amt.then(move |result| {
            match result {
                Ok((amt, _, _)) => println!("wrote {} bytes", amt),
                Err(e) => println!("error : {}", e),
            }

            Ok(())
        });
        */

        println!("Accepted connection");
        handle.spawn(receive);

        Ok(()) // keep accepting connections
/*
*/
    });

    // Run the reactor.
    core.run(server).unwrap();
}

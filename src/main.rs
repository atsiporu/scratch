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
            let mut read = tokio_io::io::read(&mut self.source, Rc::get_mut(&mut self.buf).unwrap() as &mut [u8]);
            
            let res = read.poll().unwrap();
        
            match res {
                Async::NotReady => { 
                    println!("Not ready");
                    0 
                },
                Async::Ready((rd, _, len)) => {
                    println!("Ready {}", len);
                    len
                }
            }
        };

        if len > 4 {
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
        
/*
            Some(
                tokio_io::io::read(&mut r, &mut buf as &mut [u8])
                              .map(|(r, data, len)| {
                                  buf.advance_mut(len)
                              })
            )
*/

        let mut stream = BytesStream { source: r, buf: Rc::new(BytesMut::with_capacity(5)) };
        let receive = stream.for_each(move |mut buf| {
            w.write(&buf.as_ref()[..1]);
            //unsafe { Rc::get_mut(&mut buf).unwrap().advance_mut(1); }
            Ok(())
        })
        .or_else(|e| {
            println!("Error! {:?}", e);
            Ok(())
        });

        println!("Accepted connection");
        handle.spawn(receive);

        Ok(()) // keep accepting connections
/*
*/
    });

    // Run the reactor.
    core.run(server).unwrap();
}

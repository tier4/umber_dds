use mio::*;
use std::collections::HashMap;
use mio::net::UdpSocket;
use mio::{Poll, Events, Interest, Token};
use std::net::SocketAddr;

pub struct EventLoop { poll: Poll
}

impl EventLoop {
    pub fn new(mut sockets: HashMap::<Token, UdpSocket> ) -> EventLoop {
        let poll = Poll::new().unwrap();
        for (token, lister) in &mut sockets {
            poll.registry().register(lister, *token, Interest::READABLE).unwrap();
        }
        EventLoop { poll }
    }

    pub fn event_loop(mut self) {
        let mut events = Events::with_capacity(1024);
        loop {
            println!("bbb");
            self.poll.poll(&mut events, None).unwrap();
            for event in events.iter() {
                match event.token() {
                    Token(0) => {
                        println!("hoge1");
                    }
                    Token(1) => {
                        println!("hoge2");
                    }
                    _ => {println!("aaa")}
                }
            }
        }

    }
}

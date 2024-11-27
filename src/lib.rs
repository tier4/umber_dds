//! Rust implementation of Data Distribution Service.
//! # Usage Example
//!
//! publish sample
//! ```ignore
//! use mio_extras::timer::Timer;
//! use mio_v06::{Events, Poll, PollOpt, Ready, Token};
//! use rand::SeedableRng;
//! use serde::{Deserialize, Serialize};
//! use std::time::{Duration, SystemTime};
//! use umberdds::dds::{qos::*, DomainParticipant};
//! use umberdds::structure::TopicKind;
//!
//! #[derive(Serialize, Deserialize, Clone, Debug)]
//! struct HelloWorld {
//!     index: u32,
//!     message: String,
//! }
//!
//! fn main() {
//!     let now = SystemTime::now()
//!         .duration_since(SystemTime::UNIX_EPOCH)
//!         .unwrap();
//!     let mut small_rng = rand::rngs::SmallRng::seed_from_u64(now.as_nanos() as u64);
//!
//!     let domain_id = 0;
//!     let participant = DomainParticipant::new(domain_id, &mut small_rng);
//!     let topic_qos = TopicQosBuilder::new()
//!         .reliability(policy::Reliability::default_reliable())
//!         .build();
//!     let topic = participant.create_topic(
//!         "HelloWorldTopic".to_string(),
//!         "HelloWorld".to_string(),
//!         TopicKind::WithKey,
//!         TopicQos::Policies(topic_qos),
//!     );
//!
//!     let poll = Poll::new().unwrap();
//!
//!     const WRITE_TIMER: Token = Token(0);
//!
//!     let publisher = participant.create_publisher(PublisherQos::Default);
//!     let dw_qos = DataWriterQosBuilder::new()
//!         .reliability(policy::Reliability::default_reliable())
//!         .build();
//!     let datawriter =
//!         publisher.create_datawriter::<HelloWorld>(DataWriterQos::Policies(dw_qos), topic);
//!     let mut send_count = 0;
//!
//!     let mut write_timer = Timer::default();
//!     poll.register(
//!         &mut write_timer,
//!         WRITE_TIMER,
//!         Ready::readable(),
//!         PollOpt::edge(),
//!     )
//!     .unwrap();
//!     write_timer.set_timeout(Duration::new(2, 0), ());
//!     loop {
//!         let mut events = Events::with_capacity(128);
//!         poll.poll(&mut events, None).unwrap();
//!         for event in events.iter() {
//!             match event.token() {
//!                 WRITE_TIMER => {
//!                     let send_msg = HelloWorld {
//!                         index: send_count,
//!                         message: "Hello, World!".to_string(),
//!                     };
//!                     println!("send: {:?}", send_msg);
//!                     datawriter.write(send_msg);
//!                     send_count += 1;
//!                     write_timer.set_timeout(Duration::new(2, 0), ());
//!                 }
//!                 _ => unreachable!(),
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! subscribe sample
//! ```ignore
//! use mio_v06::{Events, Poll, PollOpt, Ready, Token};
//! use rand::SeedableRng;
//! use serde::{Deserialize, Serialize};
//! use std::time::SystemTime;
//! use umberdds::dds::{qos::*, DataReaderStatusChanged, DomainParticipant};
//! use umberdds::structure::TopicKind;
//!
//! #[derive(Serialize, Deserialize, Clone, Debug)]
//! struct HelloWorld {
//!     index: u32,
//!     message: String,
//! }
//!
//! fn main() {
//!     let now = SystemTime::now()
//!         .duration_since(SystemTime::UNIX_EPOCH)
//!         .unwrap();
//!     let mut small_rng = rand::rngs::SmallRng::seed_from_u64(now.as_nanos() as u64);
//!
//!     let domain_id = 0;
//!     let participant = DomainParticipant::new(domain_id, &mut small_rng);
//!     let topic_qos = TopicQosBuilder::new()
//!         .reliability(policy::Reliability::default_reliable())
//!         .build();
//!     let topic = participant.create_topic(
//!         "HelloWorldTopic".to_string(),
//!         "HelloWorld".to_string(),
//!         TopicKind::WithKey,
//!         TopicQos::Policies(topic_qos),
//!     );
//!
//!     let poll = Poll::new().unwrap();
//!
//!     const DATAREADER: Token = Token(0);
//!
//!     let subscriber = participant.create_subscriber(SubscriberQos::Default);
//!     let dr_qos = DataReaderQosBuilder::new()
//!         .reliability(policy::Reliability::default_reliable())
//!         .build();
//!     let mut datareader =
//!         subscriber.create_datareader::<HelloWorld>(DataReaderQos::Policies(dr_qos), topic);
//!     poll.register(
//!         &mut datareader,
//!         DATAREADER,
//!         Ready::readable(),
//!         PollOpt::edge(),
//!     )
//!     .unwrap();
//!     let mut received = 0;
//!     loop {
//!         let mut events = Events::with_capacity(128);
//!         poll.poll(&mut events, None).unwrap();
//!         for event in events.iter() {
//!             match event.token() {
//!                 DATAREADER => {
//!                     while let Ok(drc) = datareader.try_recv() {
//!                         match drc {
//!                             DataReaderStatusChanged::DataAvailable => {
//!                                 let received_hello = datareader.take();
//!                                 for hello in received_hello {
//!                                     received += 1;
//!                                     println!("received: {:?}", hello);
//!                                 }
//!                                 if received > 5 {
//!                                     std::process::exit(0);
//!                                 }
//!                             }
//!                             _ => (),
//!                         }
//!                     }
//!                 }
//!                 _ => unreachable!(),
//!             }
//!         }
//!     }
//! }
//! ```

mod network;
use network::net_util;
pub mod dds;
mod rtps;
use serde::{Deserialize, Serialize};
mod discovery;
mod error;
pub mod helper;
mod message;
pub mod structure;

extern crate alloc;

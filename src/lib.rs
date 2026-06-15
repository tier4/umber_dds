//! An experimental Rust implementation of Data Distribution Service.
//! # Usage Example
//!
//! publish sample
//! ```ignore
//! use mio_extras::timer::Timer;
//! use mio_v06::{Events, Poll, PollOpt, Ready, Token};
//! use rand::SeedableRng;
//! use speedy::{Context, Readable, Writable};
//! use std::net::Ipv4Addr;
//! use std::time::{Duration, SystemTime};
//! use umber_dds::dds::{qos::*, DataWriterStatusChanged, DomainParticipant};
//! use umber_dds::DdsData;
//!
//! // for DdsData
//! use md5::compute;
//! use umber_dds::dds::key::KeyHash;
//!
//! #[derive(Clone, Debug, DdsData)]
//! struct HelloWorld {
//!     index: u32,
//!     message: String,
//! }
//! impl<'a, C: Context> Readable<'a, C> for HelloWorld {
//!     #[inline]
//!     fn read_from<R: speedy::Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
//!         let index = reader.read_u32()?;
//!         let message = {
//!             let cdr_str_len = reader.read_i32()?;
//!             let c = reader.read_string((cdr_str_len - 1) as usize)?;
//!             reader.read_u8()?; // null char
//!             reader.skip_bytes((4 - cdr_str_len as usize % 4) % 4)?;
//!             c
//!         };
//!         Ok(Self { index, message })
//!     }
//! }
//! impl<C: Context> Writable<C> for HelloWorld {
//!     #[inline]
//!     fn write_to<T: ?Sized + speedy::Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
//!         writer.write_u32(self.index)?;
//!         let cdr_str_len = self.message.len() + 1;
//!         writer.write_i32(cdr_str_len as i32)?;
//!         writer.write_bytes(self.message.as_bytes())?;
//!         writer.write_u8(0)?; // null char
//!
//!         // padding
//!         const ZEROS: [u8; 3] = [0; 3];
//!         writer.write_bytes(&ZEROS[..((4 - cdr_str_len % 4) % 4)])?;
//!         Ok(())
//!     }
//! }
//!
//! fn main() {
//!     let now = SystemTime::now()
//!         .duration_since(SystemTime::UNIX_EPOCH)
//!         .unwrap();
//!     let mut small_rng = rand::rngs::SmallRng::seed_from_u64(now.as_nanos() as u64);
//!
//!     let domain_id = 0;
//!     let participant =
//!         DomainParticipant::new(domain_id, vec![Ipv4Addr::new(127, 0, 0, 1)], None, &mut small_rng);
//!     let topic_qos = TopicQosBuilder::new()
//!         .reliability(policy::Reliability::default_reliable())
//!         .build();
//!     let topic = participant.create_topic::<HelloWorld>(
//!         "HelloWorldTopic".to_string(),
//!         TopicQos::Policies(Box::new(topic_qos)),
//!     );
//!
//!     let poll = Poll::new().unwrap();
//!
//!     const WRITE_TIMER: Token = Token(0);
//!     const DATA_WRITE: Token = Token(1);
//!
//!     let publisher = participant.create_publisher(PublisherQos::Default);
//!     let dw_qos = DataWriterQosBuilder::new()
//!         .reliability(policy::Reliability::default_reliable())
//!         .build();
//!     let mut datawriter =
//!         publisher.create_datawriter::<HelloWorld>(DataWriterQos::Policies(Box::new(dw_qos)), topic);
//!     poll.register(&datawriter, DATA_WRITE, Ready::readable(), PollOpt::edge())
//!         .unwrap();
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
//!                     datawriter.write(&send_msg);
//!                     send_count += 1;
//!                     write_timer.set_timeout(Duration::new(2, 0), ());
//!                 }
//!                 DATA_WRITE => {
//!                     while let Ok(dwc) = datawriter.try_recv() {
//!                         match dwc {
//!                             DataWriterStatusChanged::PublicationMatched(state) => {
//!                                 match state.current_count_change {
//!                                     1 => println!("PublicationMatched, guid: {}", state.guid),
//!                                     -1 => println!("PublicationUnmatched, guid: {}", state.guid),
//!                                     _ => unreachable!(),
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
//!
//! subscribe sample
//! ```ignore
//! use mio_v06::{Events, Poll, PollOpt, Ready, Token};
//! use rand::SeedableRng;
//! use speedy::{Context, Readable, Writable};
//! use std::net::Ipv4Addr;
//! use std::time::SystemTime;
//! use umber_dds::dds::{qos::*, DataReaderStatusChanged, DomainParticipant};
//! use umber_dds::DdsData;
//!
//! // for DdsData
//! use md5::compute;
//! use umber_dds::dds::key::KeyHash;
//!
//! #[derive(Clone, Debug, DdsData)]
//! struct HelloWorld {
//!     index: u32,
//!     message: String,
//! }
//! impl<'a, C: Context> Readable<'a, C> for HelloWorld {
//!     #[inline]
//!     fn read_from<R: speedy::Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
//!         let index = reader.read_u32()?;
//!         let message = {
//!             let cdr_str_len = reader.read_i32()?;
//!             let c = reader.read_string((cdr_str_len - 1) as usize)?;
//!             reader.read_u8()?; // null char
//!             reader.skip_bytes((4 - cdr_str_len as usize % 4) % 4)?;
//!             c
//!         };
//!         Ok(Self { index, message })
//!     }
//! }
//! impl<C: Context> Writable<C> for HelloWorld {
//!     #[inline]
//!     fn write_to<T: ?Sized + speedy::Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
//!         writer.write_u32(self.index)?;
//!         let cdr_str_len = self.message.len() + 1;
//!         writer.write_i32(cdr_str_len as i32)?;
//!         writer.write_bytes(self.message.as_bytes())?;
//!         writer.write_u8(0)?; // null char
//!
//!         // padding
//!         const ZEROS: [u8; 3] = [0; 3];
//!         writer.write_bytes(&ZEROS[..((4 - cdr_str_len % 4) % 4)])?;
//!         Ok(())
//!     }
//! }
//!
//! fn main() {
//!     let now = SystemTime::now()
//!         .duration_since(SystemTime::UNIX_EPOCH)
//!         .unwrap();
//!     let mut small_rng = rand::rngs::SmallRng::seed_from_u64(now.as_nanos() as u64);
//!
//!     let domain_id = 0;
//!     let participant =
//!         DomainParticipant::new(domain_id, vec![Ipv4Addr::new(127, 0, 0, 1)], None, &mut small_rng);
//!     let topic_qos = TopicQosBuilder::new()
//!         .reliability(policy::Reliability::default_reliable())
//!         .build();
//!     let topic = participant.create_topic::<HelloWorld>(
//!         "HelloWorldTopic".to_string(),
//!         TopicQos::Policies(Box::new(topic_qos)),
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
//!     let mut datareader = subscriber
//!         .create_datareader::<HelloWorld>(DataReaderQos::Policies(Box::new(dr_qos)), topic);
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
//!                                     println!("received: {:?}", hello.data());
//!                                 }
//!                                 if received >= 5 {
//!                                     println!("received 5 messages. exit.");
//!                                     std::process::exit(0);
//!                                 }
//!                             }
//!                             DataReaderStatusChanged::SubscriptionMatched(state) => {
//!                                 match state.current_count_change {
//!                                     1 => println!("SubscriptionMatched, guid: {}", state.guid),
//!                                     -1 => println!("SubscriptionUnmatched, guid: {}", state.guid),
//!                                     _ => unreachable!(),
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
mod utils;

pub use dds::key::DdsData;
pub use ddsdata_derive::DdsData;

extern crate alloc;

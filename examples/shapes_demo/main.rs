use cdr::{CdrBe, Infinite};
use clap::{Arg, Command};
use log::LevelFilter;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Config, Deserializers, Root},
    encode::pattern::PatternEncoder,
    init_config, init_file,
};
use md5::compute;
use mio_extras::timer::Timer;
use mio_v06::{Events, Poll, PollOpt, Ready, Token};
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use umber_dds::dds::key::KeyHash;
use umber_dds::dds::{qos::*, DataReaderStatusChanged, DataWriterStatusChanged, DomainParticipant};
use umber_dds::DdsData;

#[derive(Serialize, Deserialize, Clone, Debug, DdsData)]
#[dds_data(type_name = "ShapeType")]
struct Shape {
    #[key]
    color: String,
    x: i32,
    y: i32,
    shapesize: i32,
}
/*
Shape.idl
```
struct ShapeType {
    @key string color;
    long x;
    long y;
    long shapesize;
};
```
*/

fn main() {
    if let Err(_e) = init_file("shapes_logging.yml", Deserializers::default()) {
        let stderr = ConsoleAppender::builder()
            .target(log4rs::append::console::Target::Stderr)
            .encoder(Box::new(PatternEncoder::new(
                "[{l}] [{d(%s%.f)}] [{t}]: {m}{n}",
            )))
            .build();

        let config = Config::builder()
            .appender(Appender::builder().build("stderr", Box::new(stderr)))
            .build(Root::builder().appender("stderr").build(LevelFilter::Warn))
            .unwrap();

        init_config(config).unwrap();
    }
    let args = Command::new("shapes_demo")
        .arg(
            Arg::new("mode")
                .short('m')
                .help("Publisher(p|P) or Subscriber(s|S)")
                .required(true),
        )
        .arg(
            Arg::new("reliability")
                .short('r')
                .help("Reliable(r|R) or BestEffort(b|B)")
                .required(false),
        )
        .get_matches();
    let is_reliable =
        if let Some(reliability) = args.get_one::<String>("reliability").map(String::as_str) {
            match reliability {
                "r" | "R" => true,
                "b" | "B" => false,
                c => {
                    println!(
                        "Warning: unkonown reliability '{}'. reliability mut 'r', 'R', 'b' or 'B'",
                        c
                    );
                    false
                }
            }
        } else {
            false
        };

    let poll = Poll::new().unwrap();
    const DATAREADER: Token = Token(0);
    const DATAWRITER: Token = Token(1);
    const WRITETIMTER: Token = Token(2);

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let mut small_rng = rand::rngs::SmallRng::seed_from_u64(now.as_nanos() as u64);

    let domain_id = 0;
    // You should specify some Network Interface that DDS use.
    // let nic = vec![std::net::Ipv4Addr::new(127, 0, 0, 1)];
    let nic = Vec::new();
    let participant = DomainParticipant::new(domain_id, nic, &mut small_rng);
    let topic_qos = TopicQosBuilder::new()
        .reliability(if is_reliable {
            policy::Reliability::default_reliable()
        } else {
            policy::Reliability::default_besteffort()
        })
        .build();
    let topic = participant.create_topic::<Shape>(
        "Square".to_string(),
        TopicQos::Policies(Box::new(topic_qos)),
    );

    if let Some(pub_sub) = args.get_one::<String>("mode").map(String::as_str) {
        match pub_sub {
            "p" | "P" => {
                let publisher = participant.create_publisher(PublisherQos::Default);
                let dw_qos = DataWriterQosBuilder::new()
                    .reliability(if is_reliable {
                        policy::Reliability::default_reliable()
                    } else {
                        policy::Reliability::default_besteffort()
                    })
                    .build();
                let mut datawriter = publisher
                    .create_datawriter::<Shape>(DataWriterQos::Policies(Box::new(dw_qos)), topic);
                poll.register(&datawriter, DATAWRITER, Ready::readable(), PollOpt::edge())
                    .unwrap();
                let mut timer = Timer::default();
                poll.register(&timer, WRITETIMTER, Ready::readable(), PollOpt::edge())
                    .unwrap();
                timer.set_timeout(Duration::from_millis(100), ());
                let mut shape = Shape {
                    color: "Red".to_string(),
                    x: 0,
                    y: 0,
                    shapesize: 42,
                };
                loop {
                    let mut events = Events::with_capacity(64);
                    poll.poll(&mut events, None).unwrap();
                    for event in events.iter() {
                        match event.token() {
                            DATAWRITER => {
                                while let Ok(dwc) = datawriter.try_recv() {
                                    match dwc {
                                        DataWriterStatusChanged::PublicationMatched(state) => {
                                            match state.current_count_change {
                                                1 => {
                                                    println!(
                                                        "PublicationMatched, guid: {}",
                                                        state.guid
                                                    );
                                                }
                                                -1 => {
                                                    println!(
                                                        "PublicationUnmatched, guid: {}",
                                                        state.guid
                                                    );
                                                }
                                                _ => unreachable!(),
                                            }
                                        }
                                        DataWriterStatusChanged::OfferedIncompatibleQos(e) => {
                                            println!("OfferedIncompatibleQos:\n{}", e);
                                        }
                                        DataWriterStatusChanged::LivelinessLost(_) => {
                                            println!("LivelinessLost");
                                        }
                                        _ => (),
                                    }
                                }
                            }
                            WRITETIMTER => {
                                datawriter.write(&shape);
                                println!("send: {:?}", shape);
                                shape.x = (shape.x + 5) % 255;
                                shape.y = (shape.y + 5) % 255;
                                timer.set_timeout(Duration::from_millis(1000), ());
                            }
                            _ => (), // unreachable
                        }
                    }
                }
            }
            "s" | "S" => {
                let subscriber = participant.create_subscriber(SubscriberQos::Default);
                let dr_qos = DataReaderQosBuilder::new()
                    .reliability(if is_reliable {
                        policy::Reliability::default_reliable()
                    } else {
                        policy::Reliability::default_besteffort()
                    })
                    .build();
                let datareader = subscriber
                    .create_datareader::<Shape>(DataReaderQos::Policies(Box::new(dr_qos)), topic);
                poll.register(&datareader, DATAREADER, Ready::readable(), PollOpt::edge())
                    .unwrap();
                loop {
                    let mut events = Events::with_capacity(64);
                    poll.poll(&mut events, None).unwrap();
                    for event in events.iter() {
                        match event.token() {
                            DATAREADER => {
                                while let Ok(drc) = datareader.try_recv() {
                                    match drc {
                                        DataReaderStatusChanged::DataAvailable => {
                                            let received_shapes = datareader.take();
                                            for shape in received_shapes {
                                                println!("received: {:?}", shape);
                                            }
                                        }
                                        DataReaderStatusChanged::SubscriptionMatched(state) => {
                                            match state.current_count_change {
                                                1 => {
                                                    println!(
                                                        "SubscriptionMatched, guid: {}",
                                                        state.guid
                                                    );
                                                }
                                                -1 => {
                                                    println!(
                                                        "SubscriptionUnmatched, guid: {}",
                                                        state.guid
                                                    );
                                                }
                                                _ => unreachable!(),
                                            }
                                        }
                                        DataReaderStatusChanged::RequestedIncompatibleQos(e) => {
                                            println!("RequestedIncompatibleQos:\n{}", e);
                                        }
                                        DataReaderStatusChanged::LivelinessChanged(l) => {
                                            println!("LivelinessChanged: alive:{}, not_alive: {}, guid: {}", l.alive_count_change, l.not_alive_count_change, l.guid);
                                        }
                                        _ => (), // TODO
                                    }
                                }
                            }
                            _ => (), // unreachable
                        }
                    }
                }
            }
            c => println!(
                "unkonown mode '{}', if Publisher 'p' or 'P' , if Subscriber 's' or 'S'",
                c
            ),
        }
    } else {
        println!("mode is not specified");
    }
}

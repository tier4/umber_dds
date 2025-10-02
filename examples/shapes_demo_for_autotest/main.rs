use cdr::{CdrBe, Infinite};
use clap::{Arg, Command};
use log::LevelFilter;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use md5::compute;
use mio_extras::timer::Timer;
use mio_v06::{Events, Poll, PollOpt, Ready, Token};
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use umber_dds::dds::key::KeyHash;
use umber_dds::dds::{
    qos::*, DataReader, DataReaderStatusChanged, DataWriter, DataWriterStatusChanged,
    DomainParticipant,
};
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

enum Entity {
    Datareader(DataReader<Shape>),
    Datawriter(DataWriter<Shape>),
}

fn main() {
    println!("--- shapes_demo_for_autotest start");
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{l}] [{d(%s%.f)}] [{t}] [pid: {P}]: {m}{n}",
        )))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Trace))
        .unwrap();

    log4rs::init_config(config).unwrap();

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

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let mut small_rng = rand::rngs::SmallRng::seed_from_u64(now.as_nanos() as u64);

    let domain_id = 0;
    let participant = DomainParticipant::new(domain_id, Vec::new(), &mut small_rng);
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

    let poll = Poll::new().unwrap();
    let mut end_timer = Timer::default();
    end_timer.set_timeout(Duration::new(30, 0), ());

    const END_TIMER: Token = Token(0);
    const WRITE_TIMER: Token = Token(1);
    const DATAREADER: Token = Token(2);
    const DATAWRITER: Token = Token(3);

    poll.register(
        &mut end_timer,
        END_TIMER,
        Ready::readable(),
        PollOpt::edge(),
    )
    .unwrap();

    let entity;

    if let Some(pub_sub) = args.get_one::<String>("mode").map(String::as_str) {
        entity = match pub_sub {
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
                poll.register(
                    &mut datawriter,
                    DATAWRITER,
                    Ready::readable(),
                    PollOpt::edge(),
                )
                .unwrap();
                Entity::Datawriter(datawriter)
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
                let mut datareader = subscriber
                    .create_datareader::<Shape>(DataReaderQos::Policies(Box::new(dr_qos)), topic);
                poll.register(
                    &mut datareader,
                    DATAREADER,
                    Ready::readable(),
                    PollOpt::edge(),
                )
                .unwrap();
                Entity::Datareader(datareader)
            }
            c => panic!(
                "unkonown mode '{}', if Publisher 'p' or 'P' , if Subscriber 's' or 'S'",
                c
            ),
        };
    } else {
        panic!("mode is not specified");
    }

    let mut received = 0;
    let mut shape = Shape {
        color: "Red".to_string(),
        x: 0,
        y: 0,
        shapesize: 42,
    };
    let (datareader, datawriter) = match entity {
        Entity::Datareader(dr) => (Some(dr), None),
        Entity::Datawriter(dw) => (None, Some(dw)),
    };

    let mut write_timer = Timer::default();
    poll.register(
        &mut write_timer,
        WRITE_TIMER,
        Ready::readable(),
        PollOpt::edge(),
    )
    .unwrap();
    if datawriter.is_some() {
        write_timer.set_timeout(Duration::new(2, 0), ());
    }
    loop {
        let mut events = Events::with_capacity(128);
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                END_TIMER => {
                    println!("--- shapes_demo_for_autotest end");
                    if datareader.is_some() {
                        std::process::exit(-1);
                    } else {
                        std::process::exit(0);
                    }
                }
                WRITE_TIMER => {
                    if let Some(dw) = &datawriter {
                        dw.write(shape.clone());
                        println!("send: {:?}", shape);
                        shape.x = (shape.x + 5) % 255;
                        shape.y = (shape.y + 5) % 255;
                    }
                    write_timer.set_timeout(Duration::new(2, 0), ());
                }
                DATAREADER => {
                    if let Some(dr) = &datareader {
                        while let Ok(r) = dr.try_recv() {
                            match r {
                                DataReaderStatusChanged::DataAvailable => {
                                    let received_shapes = dr.take();
                                    for shape in received_shapes {
                                        received += 1;
                                        println!("received: {:?}", shape);
                                    }
                                    if received > 5 {
                                        println!("--- shapes_demo_for_autotest end");
                                        std::process::exit(0);
                                    }
                                }
                                DataReaderStatusChanged::SubscriptionMatched(state) => {
                                    if state.current_count_change == 1 {
                                        println!("SubscriptionMatched, guid: {:?}", state.guid);
                                    } else {
                                        println!("SubscriptionUnmatched, guid: {:?}", state.guid);
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }
                DATAWRITER => {
                    if let Some(dw) = &datawriter {
                        while let Ok(w) = dw.try_recv() {
                            match w {
                                DataWriterStatusChanged::PublicationMatched(state) => {
                                    if state.current_count_change == 1 {
                                        println!("PublicationMatched, guid: {:?}", state.guid);
                                    } else {
                                        println!("PublicationUnmatched, guid: {:?}", state.guid);
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

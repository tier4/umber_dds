use clap::{Arg, Command};
use mio_extras::timer::Timer;
use mio_v06::{Events, Poll, PollOpt, Ready, Token};
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use umberdds::dds::{
    qos::*, DataReader, DataReaderStatusChanged, DataWriter, DataWriterStatusChanged,
    DomainParticipant,
};
use umberdds::structure::TopicKind;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Shape {
    color: String,
    x: i32,
    y: i32,
    shapesize: i32,
}

enum Entity {
    Datareader(DataReader<Shape>),
    Datawriter(DataWriter<Shape>),
}

fn main() {
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
    let participant = DomainParticipant::new(domain_id, &mut small_rng);
    let topic = participant.create_topic(
        "Square".to_string(),
        "ShapeType".to_string(),
        TopicKind::WithKey,
        TopicQos::Default,
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
                let mut datawriter =
                    publisher.create_datawriter::<Shape>(DataWriterQos::Policies(dw_qos), topic);
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
                let mut datareader =
                    subscriber.create_datareader::<Shape>(DataReaderQos::Policies(dr_qos), topic);
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
                                        std::process::exit(0);
                                    }
                                }
                                DataReaderStatusChanged::SubscriptionMatched(state) => {
                                    println!("SubscriptionMatched, guid: {:?}", state.guid);
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
                                    println!("PublicationMatched, guid: {:?}", state.guid);
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

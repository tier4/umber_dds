use clap::{Arg, Command};
use mio_extras::timer::Timer;
use mio_v06::{Events, Poll, PollOpt, Ready, Token};
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use umberdds::dds::{
    datareader::DataReader, datawriter::DataWriter, participant::DomainParticipant, qos::*,
};
use umberdds::structure::topic_kind::TopicKind;

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
    let topic_qos = TopicQosBuilder::new()
        .reliability(if is_reliable {
            policy::Reliability::default_reliable()
        } else {
            policy::Reliability::default_besteffort()
        })
        .build();
    let topic = participant.create_topic(
        "Square".to_string(),
        "ShapeType".to_string(),
        TopicKind::WithKey,
        TopicQos::Policies(topic_qos),
    );

    let poll = Poll::new().unwrap();
    let mut timer = Timer::default();
    timer.set_timeout(Duration::new(30, 0), ());

    const TIMER: Token = Token(0);
    const DATAREADER: Token = Token(1);

    poll.register(&mut timer, TIMER, Ready::readable(), PollOpt::edge())
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
                let datawriter =
                    publisher.create_datawriter::<Shape>(DataWriterQos::Policies(dw_qos), topic);
                Entity::Datawriter(datawriter)
            }
            "s" | "S" => {
                let subscriber = participant.create_subscriber(SubscriberQos::Default);
                let dr_qos = DataReadedrQosBuilder::new()
                    .reliability(if is_reliable {
                        policy::Reliability::default_reliable()
                    } else {
                        policy::Reliability::default_besteffort()
                    })
                    .build();
                let mut datareader =
                    subscriber.create_datareader::<Shape>(DataReadedrQos::Policies(dr_qos), topic);
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
    loop {
        let mut events = Events::with_capacity(128);
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            match event.token() {
                TIMER => {
                    if datareader.is_some() {
                        std::process::exit(-1);
                    } else {
                        std::process::exit(0);
                    }
                }
                DATAREADER => {
                    if let Some(dr) = &datareader {
                        let received_shapes = dr.take();
                        for shape in received_shapes {
                            received += 1;
                            println!("received: {:?}", shape);
                        }
                        if received > 5 {
                            std::process::exit(0);
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        if let Some(dw) = &datawriter {
            dw.write(shape.clone());
            println!("send: {:?}", shape);
            shape.x = (shape.x + 5) % 255;
            shape.y = (shape.y + 5) % 255;
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

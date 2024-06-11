use clap::Parser;
use rs_rtps::dds::{participant::DomainParticipant, qos::*, topic::Topic, typedesc::TypeDesc};
use rs_rtps::structure::topic_kind::TopicKind;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Shape {
    color: String,
    x: i32,
    y: i32,
    shapesize: i32,
}

#[derive(Parser, Debug)]
struct Args {
    pub_sub: char,
}

fn main() {
    let args = Args::parse();
    let domain_id = 0;
    let participant = DomainParticipant::new(domain_id);
    let qos = QosBuilder::new().build();
    let topic = Topic::new(
        "Square".to_string(),
        TypeDesc::new("ShapeType".to_string()),
        participant.clone(),
        qos,
        TopicKind::WithKey,
    );

    match args.pub_sub {
        'p' | 'P' => {
            let publisher = participant.create_publisher(qos);
            let datawriter = publisher.create_datawriter::<Shape>(qos, topic);
            let mut shape = Shape {
                color: "Red".to_string(),
                x: 0,
                y: 0,
                shapesize: 42,
            };
            loop {
                datawriter.write(shape.clone());
                println!("send: {:?}", shape);
                shape.x = (shape.x + 1) % 255;
                shape.y = (shape.y + 1) % 255;
                std::thread::sleep(Duration::from_millis(1000));
            }
        }
        's' | 'S' => {
            let subscriber = participant.create_subscriber(qos);
            let datareader = subscriber.create_datareader::<Shape>(qos, topic);
            loop {
                let received_shapes = datareader.take();
                for shape in received_shapes {
                    println!("received: {:?}", shape);
                }
            }
        }
        c => println!("unkonown arg '{}'", c),
    }
}

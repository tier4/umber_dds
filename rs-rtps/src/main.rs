use std::net::{Ipv4Addr, UdpSocket};
mod network;
use network::{net_util, udp_listinig_socket};
mod dds;
mod rtps;
use dds::{
    datawriter::DataWriter, event_loop, participant::DomainParticipant, qos::*, topic::Topic,
    typedesc::TypeDesc,
};
use serde::{Deserialize, Serialize};
use structure::topic_kind::TopicKind;
mod discovery;
mod message;
mod structure;

#[derive(Serialize, Deserialize, Clone)]
struct Shape {
    color: String,
    x: i32,
    y: i32,
    shapesize: i32,
}

fn main() {
    let domain_id = 0;
    let participant = DomainParticipant::new(domain_id);
    let qos = QosBuilder::new().build();
    let topic = Topic::new(
        "Square".to_string(),
        TypeDesc::new("hoge".to_string()),
        participant.clone(),
        qos,
        TopicKind::WithKey,
    );
    /*
    let subscriber = participant.create_subscriber(qos);
    let datareader = subscriber.create_datareader::<Shape>(qos, topic);
    */
    let publisher = participant.create_publisher(qos);
    let datawriter = publisher.create_datawriter::<Shape>(qos, topic);

    let shape = Shape {
        color: "Red".to_string(),
        x: 3,
        y: 2,
        shapesize: 42,
    };

    for _i in 0..=5 {
        println!("@main datawriter.write");
        datawriter.write(shape.clone());
    }
    loop {}
}

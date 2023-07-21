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
    let publisher = participant.create_publisher(qos);
    loop {}
}

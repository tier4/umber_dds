use std::net::Ipv4Addr;
use std::net::UdpSocket;
use std::str::FromStr;
use std::thread;

fn recv_rtps(socket: UdpSocket, addr: &str) {
    loop {
        let mut buf = [0; 1024];
        let (num_of_bytes, src_addr) = socket.recv_from(&mut buf).expect("Didn't receive data");
        print!("{}, from {}, length: {}, ", addr, src_addr, num_of_bytes,);
        for i in 0..10 {
            print!("{:02X} ", buf[i])
        }
        println!();
    }
}

fn main() -> std::io::Result<()> {
    {
        let multi_cast = "239.255.0.1";

        let join_handle1: thread::JoinHandle<_> = thread::spawn(|| {
            let addr = "192.168.208.3:50726";
            let socket = UdpSocket::bind(addr).expect("Couldn't bind {addr}");
            socket
                .join_multicast_v4(
                    &Ipv4Addr::from_str(multi_cast).unwrap(),
                    &Ipv4Addr::UNSPECIFIED,
                )
                .expect("Couldn't join multicast");
            recv_rtps(socket, addr);
        });

        let join_handle2: thread::JoinHandle<_> = thread::spawn(|| {
            let addr = "0.0.0.0:7400";
            let socket = UdpSocket::bind(addr).expect("Couldn't bind {addr}");
            recv_rtps(socket, addr);
        });

        let join_handle3: thread::JoinHandle<_> = thread::spawn(|| {
            let addr = "0.0.0.0:7412";
            let socket = UdpSocket::bind(addr).expect("Couldn't bind {addr}");
            recv_rtps(socket, addr);
        });

        let join_handle4: thread::JoinHandle<_> = thread::spawn(|| {
            let addr = "0.0.0.0:7413";
            let socket = UdpSocket::bind(addr).expect("Couldn't bind {addr}");
            recv_rtps(socket, addr);
        });

        let join_handle5: thread::JoinHandle<_> = thread::spawn(|| {
            let addr = "0.0.0.0:52222";
            let socket = UdpSocket::bind(addr).expect("couldn't bind {addr}");
            recv_rtps(socket, addr);
        });

        join_handle1.join().unwrap();
        join_handle2.join().unwrap();
        join_handle3.join().unwrap();
        join_handle4.join().unwrap();
        join_handle5.join().unwrap();
    } // the socket is closed here

    Ok(())
}

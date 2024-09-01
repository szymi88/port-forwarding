#![warn(rust_2018_idioms)]

use std::env;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::net::UdpSocket;
use log::{debug, error, log_enabled, info, Level};


#[tokio::main]
async fn main() {
    env_logger::init();
    info!("App starting");

    let listen_ip = env::args().nth(1).expect("Listen IP missing");
    let target_ip = env::args().nth(2).expect("Target IP missing");
    let ports = env::args().nth(3).expect("Ports to forward missing");

    let ports_arr: Vec<u16> = ports.split(',')
        .map(|port| port.trim().parse().expect("Invalid port number"))
        .collect();

    let task1 = tokio::spawn({
        let listen_ip = listen_ip.clone();
        let target_ip = target_ip.clone();
        let port = ports_arr[0];
        async move {
            forward_port(&listen_ip, &target_ip, port).await.ok();
        }
    });

    let task2 = tokio::spawn({
        let listen_ip = listen_ip.clone();
        let target_ip = target_ip.clone();
        let port = ports_arr[1];
        async move {
            forward_port(&listen_ip, &target_ip, port).await.ok();
        }
    });

    tokio::join!(task1, task2);
}

async fn forward_port(listen_ip: &str, target_ip: &str, port: u16) -> std::io::Result<()> {
    info!("Forwarding the port {} from {} to {}", port, listen_ip, target_ip);

    let target_addr = format!("{}:{}", target_ip, port);
    let target_socket_addr = SocketAddr::from_str(&*target_addr).unwrap();


    let mut client_addr: Option<SocketAddr> = None;
    let sock = UdpSocket::bind(format!("{}:{}", listen_ip, port)).await?;
    let mut buf = [0; 10240];

    let mut client_forwarded_count = 0;
    let mut target_forwarded_count = 0;


    loop {
        let (len, req_addr) = sock.recv_from(&mut buf).await?;
        debug!("{:?} bytes received from {:?}", len, req_addr);

        if client_addr == None {
            client_addr = Option::from(req_addr);
            info!("Set client_addr tp {:?}", client_addr);
        }

        let forward_addr = if req_addr.to_string() == client_addr.unwrap().to_string() {
            target_forwarded_count += 1;
            target_socket_addr
        } else {
            client_forwarded_count += 1;
            client_addr.unwrap()
        };

        let len = sock.send_to(&buf[..len], forward_addr).await?;
        debug!("{:?} bytes sent to {}", len, forward_addr);
        debug!("Total packets moved {} / {}", client_forwarded_count, target_forwarded_count);
    }
}
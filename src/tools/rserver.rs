// register is used to build connection between TAPEs and resource cyber world

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

use log::{info, warn};

use crate::base::resource::RegisterServer;

const IP: [u8; 4] = [127, 0, 0, 1];
const PORT: u16 = 8000;

pub fn tape_server() {
    let mut tapes: Vec<RegisterServer> = Vec::new();
    let v4 = Ipv4Addr::new(IP[0], IP[1], IP[2], IP[3]);
    let ipv4 = IpAddr::V4(v4);
    let addr = SocketAddr::new(ipv4,PORT);
    let socket = UdpSocket::bind(addr).unwrap();
    // info!("process on");
    
    let mut buf = [0; 1024];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                // we should send two message: 
                // 1. can it be a TAPE
                // 2. its information so that we can match exactly.
                info!("process {src}");
                let data = match std::str::from_utf8(&buf[..amt]) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("parse error: {e}");
                        continue;
                    }
                };
                let s: RegisterServer = match serde_json::from_str(data) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("parse error: {e}");
                        continue;
                    },
                };
                // info!("process connect");
                let mut best_suit: Option<&RegisterServer> = None;
                for t in tapes.iter() {
                    if t.is_position_suit(&s) {
                        best_suit = Some(t);
                    }
                }
                if best_suit.is_some() {
                    let t_addr_json = serde_json::to_string(&best_suit).unwrap();
                    socket.send_to(&t_addr_json.as_bytes(), src).unwrap();
                    let _ = best_suit;
                } else {
                    warn!("no suit tape {}", src);
                }
                
                if s.is_tape() {
                    tapes.push(s);
                }
            },
            Err(e) => {
                warn!("receive error: {e}");
            }
        }
    }
}

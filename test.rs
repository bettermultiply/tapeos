use std::{error::Error, net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket}, thread::sleep, time::Duration};

fn main() -> Result<(), Box<dyn Error>>{

    let v4 = Ipv4Addr::new(127, 0, 0, 1);
    let ipv4 = IpAddr::V4(v4);
    let addr = SocketAddr::new(ipv4, 7009);
    let socket = UdpSocket::bind(addr)?;
    let target_addr1 = SocketAddr::new(ipv4, 28004);
    // let target_addr2 = SocketAddr::new(ipv4, 29002);
    // let target_addr3 = SocketAddr::new(ipv4, 29003);
    // let target_addr4 = SocketAddr::new(ipv4, 29004);

    // loop {
    for j in 0..1 {
        for i in 0..350 {
            let s = format!("change my name to: BetMul_{j}_{i}");
            socket.send_to(s.as_bytes(), target_addr1)?; 
        }
        // socket.send_to(s.as_bytes(), target_addr2)?; 
        // socket.send_to(s.as_bytes(), target_addr3)?; 
        // socket.send_to(s.as_bytes(), target_addr4)?;
    }
    Ok(())
}

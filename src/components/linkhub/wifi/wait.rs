use std::net::UdpSocket;
use std::thread;
use std::time::Duration;
use std::error::Error;

const TAPE_ADDRESS: &str = "0.0.0.0:8888";

pub fn wait() -> Result<(), Box<dyn Error>> {
    
    // TODO:
    // 1. 发现 外部seeker 的广播，并建立连接
    // 2. 监听来自 外部seeker 的请求和消息
    // 3. 接发 内部seeker 的信息
    

    Ok(())
}
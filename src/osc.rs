use anyhow::Result;
use rosc::{OscMessage, OscPacket, OscType};
use std::net::UdpSocket;

pub fn send_chatbox(address: &str, port: u16, text: &str) -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let msg = OscMessage {
        addr: "/chatbox/input".to_string(),
        args: vec![
            OscType::String(text.to_string()),
            OscType::Bool(true),
            OscType::Bool(false),
        ],
    };
    let packet = OscPacket::Message(msg);
    let buf = rosc::encoder::encode(&packet)?;
    socket.send_to(&buf, format!("{}:{}", address, port))?;
    Ok(())
}

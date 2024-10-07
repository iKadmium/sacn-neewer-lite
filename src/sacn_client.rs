use async_std::net::UdpSocket;
use ratatui::style::Color;
use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::sync::RwLock;

use crate::sacn_packet::SacnDmxPacket;
use crate::terminal_ui::TerminalUi;

const SACN_PORT: u16 = 5568;

pub struct SacnClient {
    socket: UdpSocket,
    universes: Vec<u16>,
}

impl SacnClient {
    pub async fn new(universes: Vec<u16>) -> io::Result<Self> {
        let socket_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, SACN_PORT);
        let socket = UdpSocket::bind(socket_addr).await?;
        for universe in &universes {
            let multicast_addr =
                Ipv4Addr::new(239, 255, (*universe >> 8) as u8, (*universe & 0xFF) as u8);
            socket.join_multicast_v4(multicast_addr, Ipv4Addr::UNSPECIFIED)?;
        }

        Ok(SacnClient { socket, universes })
    }

    pub async fn disconnect(&self, terminal: &RwLock<TerminalUi>) -> Result<(), btleplug::Error> {
        terminal
            .write()
            .await
            .set_sacn_status("Disconnected", Color::Red);

        for universe in &self.universes {
            self.socket
                .leave_multicast_v4(
                    Ipv4Addr::new(239, 255, (*universe >> 8) as u8, (*universe & 0xFF) as u8),
                    Ipv4Addr::UNSPECIFIED,
                )
                .unwrap();
        }
        Ok(())
    }

    pub async fn receive(&self) -> Result<SacnDmxPacket, io::Error> {
        let mut buf = [0; 1024];
        loop {
            let amt = self.socket.recv(&mut buf).await?;
            let packet = &buf[..amt];
            if SacnDmxPacket::is_data_packet(packet) {
                let sacn_packet = SacnDmxPacket::from_bytes(packet.to_vec()).unwrap();
                return Ok(sacn_packet);
            }
        }
    }

    pub fn get_socket(&self) -> &UdpSocket {
        return &self.socket;
    }
}

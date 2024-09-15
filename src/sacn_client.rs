use async_std::net::UdpSocket;
use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};

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

    pub async fn disconnect(&self) -> Result<(), btleplug::Error> {
        println!("Disconnecting from lights");

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

    pub fn get_socket(&self) -> &UdpSocket {
        return &self.socket;
    }
}

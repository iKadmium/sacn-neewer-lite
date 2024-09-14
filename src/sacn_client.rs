use std::io;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

use crate::light::Light;
use crate::sacn_packet::{from_bytes, is_data_packet, SacnDmxPacket};

const SACN_PORT: u16 = 5568;

pub struct SacnClient {
    lights: Vec<Light>,
    socket: UdpSocket,
}

impl SacnClient {
    pub fn new(universe: u16, lights: Vec<Light>) -> io::Result<Self> {
        let multicast_addr =
            Ipv4Addr::new(239, 255, (universe >> 8) as u8, (universe & 0xFF) as u8);
        let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, SACN_PORT))?;
        socket.join_multicast_v4(&multicast_addr, &Ipv4Addr::UNSPECIFIED)?;

        Ok(SacnClient { socket, lights })
    }

    pub async fn listen(&self) -> io::Result<()> {
        let mut buf = [0; 1024];
        loop {
            let (amt, _src) = self.socket.recv_from(&mut buf)?;
            let packet = &buf[..amt];
            if is_data_packet(packet) {
                let sacn_packet = from_bytes(packet.to_vec()).unwrap();
                self.handle_packet(&sacn_packet).await.unwrap();
            }
        }
    }

    async fn handle_packet(&self, packet: &SacnDmxPacket) -> Result<(), btleplug::Error> {
        for light in &self.lights {
            let red = packet.dmx_data[light.get_address() as usize];
            let green = packet.dmx_data[light.get_address() as usize + 1];
            let blue = packet.dmx_data[light.get_address() as usize + 2];
            light.set_color_rgb(red, green, blue).await?;
        }
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), btleplug::Error> {
        self.socket
            .leave_multicast_v4(&Ipv4Addr::new(239, 255, 0, 0), &Ipv4Addr::UNSPECIFIED)
            .unwrap();
        for light in &self.lights {
            light.disconnect().await?;
        }
        Ok(())
    }
}

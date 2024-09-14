#[derive(Debug, Clone)]
pub struct SacnDmxPacket {
    pub source_name: String,
    pub universe: u16,
    pub priority: u8,
    pub sequence_number: u8,
    pub options: u8,
    pub dmx_data: Vec<u8>,
    pub cid: [u8; 16],
}

impl SacnDmxPacket {
    pub fn new(
        source_name: String,
        universe: u16,
        priority: u8,
        sequence_number: u8,
        options: u8,
        dmx_data: Vec<u8>,
        cid: [u8; 16],
    ) -> Self {
        SacnDmxPacket {
            source_name,
            universe,
            priority,
            sequence_number,
            options,
            dmx_data,
            cid,
        }
    }
}

pub fn from_bytes(bytes: Vec<u8>) -> Result<SacnDmxPacket, &'static str> {
    if bytes.len() < 38 {
        return Err("Byte array too short");
    }

    let source_name = String::from_utf8_lossy(&bytes[44..108])
        .trim_end_matches('\0')
        .to_string();
    let universe = u16::from_be_bytes([bytes[113], bytes[114]]);
    let priority = bytes[108];
    let sequence_number = bytes[111];
    let options = bytes[112];
    let dmx_data_len = u16::from_be_bytes([bytes[123], bytes[124]]) as usize;
    let dmx_data = bytes[125..(125 + dmx_data_len)].to_vec();
    let mut cid = [0u8; 16];
    cid.copy_from_slice(&bytes[22..38]);

    Ok(SacnDmxPacket {
        source_name,
        universe,
        priority,
        sequence_number,
        options,
        dmx_data,
        cid,
    })
}

pub fn is_data_packet(bytes: &[u8]) -> bool {
    // Check if the byte vector is long enough to be a valid Data Packet
    if bytes.len() < 38 {
        return false;
    }

    // Check the ACN Packet Identifier ("ASC-E1.17")
    let acn_pid = &bytes[4..16];
    if acn_pid != b"ASC-E1.17\0\0\0" {
        return false;
    }

    // Check the Vector for the Root Layer (0x00000004)
    let vector_root_layer = u32::from_be_bytes([bytes[18], bytes[19], bytes[20], bytes[21]]);
    if vector_root_layer != 0x00000004 {
        return false;
    }

    // Check the Vector for the Framing Layer (0x00000002)
    let vector_framing_layer = u32::from_be_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]);
    if vector_framing_layer != 0x00000002 {
        return false;
    }

    // Check the Vector for the DMP Layer (0x02)
    let vector_dmp_layer = bytes[117];
    if vector_dmp_layer != 0x02 {
        return false;
    }

    true
}

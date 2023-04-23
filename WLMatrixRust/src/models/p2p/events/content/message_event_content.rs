use byteorder::{ByteOrder, LittleEndian};

use crate::models::{msn_user::MSNUser, p2p::p2p_transport_packet::P2PTransportPacket};

#[derive(Clone, Debug)]
pub struct MessageEventContent {
    pub packets: Vec<P2PTransportPacket>,
    pub sender: MSNUser,
    pub receiver: MSNUser,
}

// impl MessageEventContent {
//     pub fn as_direct_p2p(&self) -> Vec<u8> {
//         let mut msg_bytes: Vec<u8> = self.packet.to_vec();

//         let size = msg_bytes.len();
//         let mut buffer: [u8; 4] = [0, 0, 0, 0];
//         LittleEndian::write_u32(&mut buffer, size as u32);

//         let mut out = buffer.to_vec();
//         out.append(&mut msg_bytes);

//         return out;
//     }
// }

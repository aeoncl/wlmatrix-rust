use std::{str::FromStr, mem};

use byteorder::{ByteOrder, LittleEndian};
use log::{warn, info};

use crate::models::{errors::Errors, p2p::p2p_transport_packet::P2PTransportPacket};

#[derive(Clone, Debug)]

pub struct P2PCommand {
    nonce: Vec<u8>,
    is_foo: bool,
    pub data: Option<P2PTransportPacket>,
}

impl P2PCommand {
    pub fn new() -> Self {
        return P2PCommand {
            nonce: Vec::new(),
            is_foo: false,
            data: None
        };
    }

    pub fn foo() -> Self {
        return P2PCommand {
            nonce: Vec::new(),
            is_foo: true,
            data: None
        };
    }

    pub fn nonce(nonce: Vec<u8>) -> Self {
        return P2PCommand {
            nonce,
            is_foo: false,
            data: None
        };
    }

    pub fn data(p2p_packet: P2PTransportPacket) -> Self {
        return P2PCommand {
            nonce: Vec::new(),
            is_foo: false,
            data: Some(p2p_packet)
        };
    }

    pub fn append_data(&mut self, data: &[u8]) {
        if self.data.is_none() {
            return;
        }

        self.data.as_ref().unwrap();
    }

    pub fn is_foo(&self) -> bool {
        return self.is_foo;
    }

    pub fn is_data(&self) -> bool {
        return self.data.is_some();
    }

    pub fn is_nonce(&self) -> bool {
        return !self.nonce.is_empty();
    }
}

pub struct P2PCommandParser {

   incomplete_command: Vec<u8>
}

impl P2PCommandParser {

    pub fn new() -> Self {
        return P2PCommandParser{ incomplete_command: Vec::new() };
    }
    
    fn get_payload_size(&self, data: &[u8]) -> usize {
        return LittleEndian::read_u32(&data[0..4]) as usize;
    }

    pub fn get_transport_size(&self, data: &[u8]) -> usize {
        return P2PTransportPacket::extract_payload_length(data);
    }

    fn parseNonce(&self, data: &[u8]) -> Result<P2PCommand, Errors> {
        if data.len() >= 16 {
            let nonce = data[0..16].to_owned();
            return Ok(P2PCommand::nonce(nonce));
        }

        return Err(Errors::PayloadDeserializeError);
    }

    fn parse_p2p_payload(&self, data: &[u8], payload_size: usize) -> Result<P2PCommand, Errors> {

        let p2p_transport_packet = P2PTransportPacket::try_from(data)?;
        return Ok(P2PCommand::data(p2p_transport_packet));
    }

    fn store_chunked(&mut self, chunk: &[u8]) {
        self.incomplete_command = chunk.to_vec();
    }

    fn pop_chunked(&mut self) -> Vec<u8> {
        return mem::take(&mut self.incomplete_command);
    }

    pub fn parse_message(&mut self, message: &[u8], expecting_nonce: bool) -> Vec<P2PCommand> {
        
        let mut previous_chunks = self.pop_chunked();
        previous_chunks.extend_from_slice(message);
        let mut current_slice = previous_chunks.as_slice();;

        let mut out = Vec::new();


        //Todo change the way we handle chunks
        loop {

            if current_slice.len() >= 4 {
                let payload_size = self.get_payload_size(current_slice);

                //info!("payload size: {}", &payload_size);
    
                if payload_size > 1400 {
                    info!("debug previous_chunk: {:?}", &current_slice);
    
                    info!("debug message: {:?}", &message);
    
                    std::process::exit(0); //todo remove this
                    break;
                }
    
                let is_chunked = current_slice.len() < payload_size + 4;
    
                if !is_chunked {
                        let content = &current_slice[4..4+payload_size];
                        if content == "foo\0".as_bytes() {
                            //foo msg
                            out.push(P2PCommand::foo());
                        } else {
                            if payload_size != 16 || !expecting_nonce {
                                // We have a P2P Transport packet
                                if let Ok(p2p_payload_packet) = self.parse_p2p_payload(content, payload_size)
                                {
                                //    if let Some(paylod) = p2p_payload_packet.data.as_ref().unwrap().get_payload().as_ref() {
                                //        info!("out packet: {:?}", paylod);
                                //    }
                                    out.push(p2p_payload_packet);
                                } else {
                                    warn!("malformed P2P payload packet : len: {}, content: {:?}", payload_size, content);
                                }
                            } else {
                                // We have a Nonce packet
                                if let Ok(nonce_packet) = self.parseNonce(content) {
                                    out.push(nonce_packet);
                                } else {
                                    warn!("malformed P2P nonce packet : len: {}, content: {:?}", payload_size, content);
                                }
                            }
                        }
                        current_slice = &current_slice[payload_size + 4..current_slice.len()]; // removed what we already parsed
                } else {
                    //store chunk
                    self.store_chunked(&current_slice);
                 //   info!("store chunk");
                    break;
                }
            } else {
                 //store chunk
                 self.store_chunked(&current_slice);
                 //   info!("store chunk");
                    break;
            }
          


        }

        return out;
    }
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8_unchecked;

    use byteorder::{LittleEndian, ByteOrder};

    use super::P2PCommandParser;

    #[test]
    fn test_foo_and_nonce_command() {
        let command: [u8; 38] = [
            4, 0, 0, 0, 102, 111, 111, 0, 16, 0, 0, 0, 165, 126, 17, 100, 117, 202, 124, 65, 145,
            112, 91, 11, 96, 69, 196, 168, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let mut parser = P2PCommandParser::new();

        let result = parser.parse_message(&command, true);

        assert_eq!(result.len(), 2);
        assert!(result.get(0).unwrap().is_foo());
        assert!(result.get(1).unwrap().is_nonce());
        assert_eq!(result.get(1).unwrap().nonce.len(), 16);
    }

    #[test]
    fn test_invite_command() {
        let command : [u8;2048] = [23, 3, 0, 0, 8, 2, 3, 15, 145, 34, 52, 81, 8, 1, 0, 1, 0, 0, 0, 0, 73, 78, 86, 73, 84, 69, 32, 77, 83, 78, 77, 83, 71, 82, 58, 97, 101, 111, 110, 116, 101, 115, 116, 51, 64, 115, 104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 55, 55, 99, 52, 54, 97, 56, 102, 45, 51, 51, 97, 51, 45, 53, 50, 56, 50, 45, 57, 97, 53, 100, 45, 57, 48, 53, 101, 99, 100, 51, 101, 98, 48, 54, 57, 125, 32, 77, 83, 78, 83, 76, 80, 47, 49, 46, 48, 13, 10, 84, 111, 58, 32, 60, 109, 115, 110, 109, 115, 103, 114, 58, 97, 101, 111, 110, 116, 101, 115, 116, 51, 64, 115, 104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 55, 55, 99, 52, 54, 97, 56, 102, 45, 51, 51, 97, 51, 45, 53, 50, 56, 50, 45, 57, 97, 53, 100, 45, 57, 48, 53, 101, 99, 100, 51, 101, 98, 48, 54, 57, 125, 62, 13, 10, 70, 114, 111, 109, 58, 32, 60, 109, 115, 110, 109, 115, 103, 114, 58, 97, 101, 111, 110, 116, 101, 115, 116, 64, 115, 104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 102, 53, 50, 57, 55, 51, 98, 54, 45, 99, 57, 50, 54, 45, 52, 98, 97, 100, 45, 57, 98, 97, 56, 45, 55, 99, 49, 101, 56, 52, 48, 101, 52, 97, 98, 48, 125, 62, 13, 10, 86, 105, 97, 58, 32, 77, 83, 78, 83, 76, 80, 47, 49, 46, 48, 47, 84, 76, 80, 32, 59, 98, 114, 97, 110, 99, 104, 61, 123, 49, 49, 70, 66, 52, 50, 53, 70, 45, 70, 53, 51, 65, 45, 52, 54, 53, 67, 45, 57, 65, 67, 53, 45, 68, 52, 55, 53, 54, 53, 54, 67, 52, 48, 49, 54, 125, 13, 10, 67, 83, 101, 113, 58, 32, 48, 32, 13, 10, 67, 97, 108, 108, 45, 73, 68, 58, 32, 123, 50, 50, 56, 52, 50, 52, 69, 56, 45, 52, 48, 55, 65, 45, 52, 65, 69, 56, 45, 56, 56, 48, 50, 45, 53, 50, 50, 51, 66, 57, 49, 70, 54, 65, 68, 67, 125, 13, 10, 77, 97, 120, 45, 70, 111, 114, 119, 97, 114, 100, 115, 58, 32, 48, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 32, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 47, 120, 45, 109, 115, 110, 109, 115, 103, 114, 45, 116, 114, 97, 110, 115, 114, 101, 113, 98, 111, 100, 121, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 76, 101, 110, 103, 116, 104, 58, 32, 51, 50, 51, 13, 10, 13, 10, 78, 101, 116, 73, 68, 58, 32, 49, 48, 52, 48, 50, 57, 54, 49, 50, 56, 13, 10, 67, 111, 110, 110, 45, 84, 121, 112, 101, 58, 32, 70, 105, 114, 101, 119, 97, 108, 108, 13, 10, 84, 67, 80, 45, 67, 111, 110, 110, 45, 84, 121, 112, 101, 58, 32, 70, 105, 114, 101, 119, 97, 108, 108, 13, 10, 85, 80, 110, 80, 78, 97, 116, 58, 32, 102, 97, 108, 115, 101, 13, 10, 73, 67, 70, 58, 32, 102, 97, 108, 115, 101, 13, 10, 73, 80, 118, 54, 45, 103, 108, 111, 98, 97, 108, 58, 32, 50, 97, 48, 50, 58, 97, 48, 51, 102, 58, 97, 49, 51, 101, 58, 50, 54, 48, 48, 58, 56, 52, 53, 54, 58, 102, 97, 55, 101, 58, 51, 48, 49, 98, 58, 99, 52, 53, 52, 13, 10, 67, 97, 112, 97, 98, 105, 108, 105, 116, 105, 101, 115, 45, 70, 108, 97, 103, 115, 58, 32, 49, 13, 10, 78, 97, 116, 45, 84, 114, 97, 118, 45, 77, 115, 103, 45, 84, 121, 112, 101, 58, 32, 87, 76, 88, 45, 78, 97, 116, 45, 84, 114, 97, 118, 45, 77, 115, 103, 45, 68, 105, 114, 101, 99, 116, 45, 67, 111, 110, 110, 101, 99, 116, 45, 82, 101, 113, 13, 10, 66, 114, 105, 100, 103, 101, 115, 58, 32, 84, 82, 85, 68, 80, 118, 49, 32, 84, 67, 80, 118, 49, 32, 83, 66, 66, 114, 105, 100, 103, 101, 32, 84, 85, 82, 78, 118, 49, 13, 10, 72, 97, 115, 104, 101, 100, 45, 78, 111, 110, 99, 101, 58, 32, 123, 67, 66, 48, 53, 65, 51, 69, 52, 45, 49, 51, 68, 68, 45, 54, 54, 52, 50, 45, 68, 65, 53, 53, 45, 53, 56, 54, 69, 55, 48, 50, 51, 65, 48, 53, 53, 125, 13, 10, 13, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut parser = P2PCommandParser::new();

        let result = parser.parse_message(&command, false);

        assert_eq!(result.len(), 1);
        assert!(result.get(0).unwrap().is_data());

    }

    
    #[test]

    fn test_one_invite_one_data_chunk() {
        let command: [u8; 2048] = [23, 3, 0, 0, 8, 0, 3, 15, 235, 1, 236, 155, 8, 1, 0, 1, 0, 0, 0, 0, 73, 78, 86, 73, 84, 69, 32, 77, 83, 78, 77, 83, 71, 82, 58, 97, 101, 111, 110, 116, 101, 115, 116, 51, 64, 115, 104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 55, 55, 99, 52, 54, 97, 56, 102, 45, 51, 51, 97, 51, 45, 53, 50, 56, 50, 45, 57, 97, 53, 100, 45, 57, 48, 53, 101, 99, 100, 51, 101, 98, 48, 54, 57, 125, 32, 77, 83, 78, 83, 76, 80, 47, 49, 46, 48, 13, 10, 84, 111, 58, 32, 60, 109, 115, 110, 109, 115, 103, 114, 58, 97, 101, 111, 110, 116, 101, 115, 116, 51, 64, 115, 104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 55, 55, 99, 52, 54, 97, 56, 102, 45, 51, 51, 97, 51, 45, 53, 50, 56, 50, 45, 57, 97, 53, 100, 45, 57, 48, 53, 101, 99, 100, 51, 101, 98, 48, 54, 57, 125, 62, 13, 10, 70, 114, 111, 109, 58, 32, 60, 109, 115, 110, 109, 115, 103, 114, 58, 97, 101, 111, 110, 116, 101, 115, 116, 64, 115, 104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 102, 53, 50, 57, 55, 51, 98, 54, 45, 99, 57, 50, 54, 45, 52, 98, 97, 100, 45, 57, 98, 97, 56, 45, 55, 99, 49, 101, 56, 52, 48, 101, 52, 97, 98, 48, 125, 62, 13, 10, 86, 105, 97, 58, 32, 77, 83, 78, 83, 76, 80, 47, 49, 46, 48, 47, 84, 76, 80, 32, 59, 98, 114, 97, 110, 99, 104, 61, 123, 65, 65, 67, 49, 69, 56, 67, 50, 45, 68, 53, 65, 53, 45, 52, 69, 68, 69, 45, 65, 56, 66, 67, 45, 48, 52, 57, 67, 48, 69, 68, 56, 53, 49, 48, 68, 125, 13, 10, 67, 83, 101, 113, 58, 32, 48, 32, 13, 10, 67, 97, 108, 108, 45, 73, 68, 58, 32, 123, 54, 53, 50, 51, 65, 52, 52, 49, 45, 48, 50, 66, 69, 45, 52, 55, 66, 70, 45, 65, 48, 68, 53, 45, 56, 69, 56, 70, 67, 52, 49, 50, 65, 54, 50, 70, 125, 13, 10, 77, 97, 120, 45, 70, 111, 114, 119, 97, 114, 100, 115, 58, 32, 48, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 32, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 47, 120, 45, 109, 115, 110, 109, 115, 103, 114, 45, 116, 114, 97, 110, 115, 114, 101, 113, 98, 111, 100, 121, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 76, 101, 110, 103, 116, 104, 58, 32, 51, 50, 51, 13, 10, 13, 10, 78, 101, 116, 73, 68, 58, 32, 49, 48, 52, 48, 50, 57, 54, 49, 50, 56, 13, 10, 67, 111, 110, 110, 45, 84, 121, 112, 101, 58, 32, 70, 105, 114, 101, 119, 97, 108, 108, 13, 10, 84, 67, 80, 45, 67, 111, 110, 110, 45, 84, 121, 112, 101, 58, 32, 70, 105, 114, 101, 119, 97, 108, 108, 13, 10, 85, 80, 110, 80, 78, 97, 116, 58, 32, 102, 97, 108, 115, 101, 13, 10, 73, 67, 70, 58, 32, 102, 97, 108, 115, 101, 13, 10, 73, 80, 118, 54, 45, 103, 108, 111, 98, 97, 108, 58, 32, 50, 97, 48, 50, 58, 97, 48, 51, 102, 58, 97, 49, 51, 101, 58, 50, 54, 48, 48, 58, 56, 52, 53, 54, 58, 102, 97, 55, 101, 58, 51, 48, 49, 98, 58, 99, 52, 53, 52, 13, 10, 67, 97, 112, 97, 98, 105, 108, 105, 116, 105, 101, 115, 45, 70, 108, 97, 103, 115, 58, 32, 49, 13, 10, 78, 97, 116, 45, 84, 114, 97, 118, 45, 77, 115, 103, 45, 84, 121, 112, 101, 58, 32, 87, 76, 88, 45, 78, 97, 116, 45, 84, 114, 97, 118, 45, 77, 115, 103, 45, 68, 105, 114, 101, 99, 116, 45, 67, 111, 110, 110, 101, 99, 116, 45, 82, 101, 113, 13, 10, 66, 114, 105, 100, 103, 101, 115, 58, 32, 84, 82, 85, 68, 80, 118, 49, 32, 84, 67, 80, 118, 49, 32, 83, 66, 66, 114, 105, 100, 103, 101, 32, 84, 85, 82, 78, 118, 49, 13, 10, 72, 97, 115, 104, 101, 100, 45, 78, 111, 110, 99, 101, 58, 32, 123, 53, 57, 57, 69, 66, 53, 53, 48, 45, 48, 49, 49, 54, 45, 56, 50, 68, 55, 45, 50, 52, 70, 54, 45, 67, 69, 55, 56, 52, 53, 54, 48, 70, 48, 50, 65, 125, 13, 10, 13, 10, 0, 120, 5, 0, 0, 8, 0, 5, 112, 235, 1, 239, 170, 20, 7, 0, 0, 154, 199, 196, 147, 1, 8, 0, 0, 0, 0, 0, 0, 4, 173, 0, 0, 65, 115, 116, 105, 101, 32, 100, 101, 32, 115, 97, 105, 110, 116, 45, 115, 97, 99, 114, 97, 109, 101, 110, 116, 32, 100, 101, 32, 98, 97, 116, 195, 168, 99, 104, 101, 32, 100, 101, 32, 118, 101, 114, 114, 97, 116, 32, 100, 101, 32, 99, 114, 105, 115, 115, 101, 32, 100, 101, 32, 116, 97, 98, 97, 114, 110, 97, 107, 32, 100, 101, 32, 99, 104, 97, 114, 111, 103, 110, 101, 32, 100, 101, 32, 109, 97, 110, 103, 101, 117, 120, 32, 100, 39, 109, 97, 114, 100, 101, 32, 100, 101, 32, 109, 97, 117, 100, 105, 110, 101, 32, 100, 101, 32, 98, 111, 117, 116, 32, 100, 39, 99, 105, 97, 114, 103, 101, 32, 100, 101, 32, 116, 111, 114, 114, 105, 101, 117, 120, 32, 100, 101, 32, 99, 105, 98, 111, 108, 101, 32, 100, 39, 195, 169, 116, 111, 108, 101, 32, 100, 39, 101, 110, 102, 97, 110, 116, 32, 100, 39, 99, 104, 105, 101, 110, 110, 101, 32, 100, 101, 32, 109, 97, 114, 100, 101, 32, 100, 101, 32, 98, 97, 112, 116, 195, 170, 109, 101, 32, 100, 101, 32, 98, 111, 117, 116, 32, 100, 39, 99, 114, 105, 115, 115, 101, 32, 100, 101, 32, 100, 111, 117, 120, 32, 74, 195, 169, 115, 117, 115, 32, 100, 101, 32, 99, 97, 108, 118, 105, 110, 111, 117, 99, 104, 101, 32, 100, 101, 32, 118, 105, 97, 110, 100, 101, 32, 195, 160, 32, 99, 104, 105, 101, 110, 32, 100, 101, 32, 99, 114, 117, 99, 105, 102, 105, 120, 32, 100, 101, 32, 116, 111, 114, 118, 105, 115, 115, 101, 32, 100, 101, 32, 109, 97, 117, 100, 105, 116, 101, 32, 109, 97, 114, 100, 101, 32, 100, 101, 32, 98, 97, 116, 105, 110, 99, 101, 32, 100, 101, 32, 99, 105, 98, 111, 117, 108, 101, 97, 117, 32, 100, 101, 32, 99, 117, 108, 32, 100, 101, 32, 99, 104, 114, 105, 115, 116, 105, 101, 32, 100, 101, 32, 112, 117, 114, 195, 169, 101, 32, 100, 101, 32, 109, 111, 115, 117, 115, 32, 100, 101, 32, 99, 97, 108, 118, 97, 105, 114, 101, 32, 100, 101, 32, 99, 104, 97, 114, 114, 117, 101, 32, 100, 101, 32, 99, 105, 97, 114, 103, 101, 32, 100, 101, 32, 115, 97, 112, 114, 105, 115, 116, 105, 32, 100, 101, 32, 99, 111, 115, 115, 105, 110, 32, 100, 101, 32, 99, 97, 108, 116, 111, 114, 32, 100, 101, 32, 103, 195, 169, 114, 105, 98, 111, 105, 114, 101, 32, 100, 101, 32, 109, 97, 117, 116, 97, 100, 105, 116, 32, 100, 101, 32, 115, 97, 99, 114, 105, 115, 116, 105, 32, 100, 101, 32, 115, 97, 99, 114, 195, 169, 102, 105, 99, 101, 32, 100, 101, 32, 115, 97, 105, 110, 116, 45, 99, 105, 109, 111, 110, 97, 113, 117, 101, 46, 65, 115, 116, 105, 101, 32, 100, 101, 32, 115, 97, 105, 110, 116, 45, 115, 97, 99, 114, 97, 109, 101, 110, 116, 32, 100, 101, 32, 98, 97, 116, 195, 168, 99, 104, 101, 32, 100, 101, 32, 118, 101, 114, 114, 97, 116, 32, 100, 101, 32, 99, 114, 105, 115, 115, 101, 32, 100, 101, 32, 116, 97, 98, 97, 114, 110, 97, 107, 32, 100, 101, 32, 99, 104, 97, 114, 111, 103, 110, 101, 32, 100, 101, 32, 109, 97, 110, 103, 101, 117, 120, 32, 100, 39, 109, 97, 114, 100, 101, 32, 100, 101, 32, 109, 97, 117, 100, 105, 110, 101, 32, 100, 101, 32, 98, 111, 117, 116, 32, 100, 39, 99, 105, 97, 114, 103, 101, 32, 100, 101, 32, 116, 111, 114, 114, 105, 101, 117, 120, 32, 100, 101, 32, 99, 105, 98, 111, 108, 101, 32, 100, 39, 195, 169, 116, 111, 108, 101, 32, 100, 39, 101, 110, 102, 97, 110, 116, 32, 100, 39, 99, 104, 105, 101, 110, 110, 101, 32, 100, 101, 32, 109, 97, 114, 100, 101, 32, 100, 101, 32, 98, 97, 112, 116, 195, 170, 109, 101, 32, 100, 101, 32, 98, 111, 117, 116, 32, 100, 39, 99, 114, 105, 115, 115, 101, 32, 100, 101, 32, 100, 111, 117, 120, 32, 74, 195, 169, 115, 117, 115, 32, 100, 101, 32, 99, 97, 108, 118, 105, 110, 111, 117, 99, 104, 101, 32, 100, 101, 32, 118, 105, 97, 110, 100, 101, 32, 195, 160, 32, 99, 104, 105, 101, 110, 32, 100, 101, 32, 99, 114, 117, 99, 105, 102, 105, 120, 32, 100, 101, 32, 116, 111, 114, 118, 105, 115, 115, 101, 32, 100, 101, 32, 109, 97, 117, 100, 105, 116, 101, 32, 109, 97, 114, 100, 101, 32, 100, 101, 32, 98, 97, 116, 105, 110, 99, 101, 32, 100, 101, 32, 99, 105, 98, 111, 117, 108, 101, 97, 117, 32, 100, 101, 32, 99, 117, 108, 32, 100, 101, 32, 99, 104, 114, 105, 115, 116, 105, 101, 32, 100, 101, 32, 112, 117, 114, 195, 169, 101, 32, 100, 101, 32, 109, 111, 115, 117, 115, 32, 100, 101, 32, 99, 97, 108, 118, 97, 105, 114, 101, 32, 100, 101, 32, 99, 104, 97, 114, 114, 117, 101, 32, 100, 101, 32, 99, 105, 97, 114, 103, 101, 32, 100, 101, 32, 115, 97, 112, 114, 105, 115, 116, 105, 32, 100, 101, 32, 99, 111, 115, 115, 105, 110, 32, 100, 101, 32, 99, 97, 108, 116, 111, 114, 32, 100, 101, 32, 103, 195, 169, 114, 105, 98, 111, 105, 114, 101, 32, 100, 101, 32, 109, 97, 117, 116, 97, 100, 105, 116, 32, 100, 101, 32, 115, 97, 99, 114, 105, 115, 116, 105, 32, 100, 101, 32, 115, 97, 99, 114, 195, 169, 102, 105, 99, 101, 32, 100, 101, 32, 115, 97, 105, 110, 116, 45, 99, 105, 109, 111, 110, 97, 113, 117, 101, 46, 86, 105, 97, 110, 100, 101, 32, 195, 160, 32, 99, 104, 105, 101, 110, 32, 100, 101, 32, 99, 195, 162, 108, 105, 113, 117, 101, 32, 100, 101, 32, 103, 195, 169, 114, 105, 98, 111, 105, 114, 101, 32, 100, 101, 32, 118, 101, 114, 114, 97, 116, 32, 100, 39, 195, 169, 116, 111, 108, 101, 32, 100, 101, 32, 103, 195, 169, 114, 105, 116, 111, 108, 101, 32, 100, 101, 32, 116, 111, 114, 114, 105, 101, 117, 120, 32, 100, 101, 32, 99, 117, 108, 32, 100, 101, 32, 99, 114, 117, 99, 105, 102, 105, 120, 32, 100, 101, 32, 115, 97, 105, 110, 116, 101, 115, 32, 102, 101, 115, 115, 101, 115, 32, 100, 101, 32, 115, 97, 99, 114, 105, 115, 116, 105, 32, 100, 101, 32, 98, 195, 162, 116, 97, 114, 100, 32, 100, 101, 32, 99, 97, 108, 118, 105, 110, 99, 101, 32, 100, 101, 32, 115, 97, 105, 110, 116, 45, 99, 105, 109, 111, 110, 97, 113, 117, 101, 32, 100, 101, 32, 115, 97, 105, 110, 116, 45, 115, 97, 99, 114, 97, 109, 101, 110, 116, 32, 100, 101, 32, 98, 97, 116, 195, 168, 99, 104, 101, 32, 100, 101, 32, 99, 105];
        
        let mut parser = P2PCommandParser::new();

        
        let result = parser.parse_message(&command, false);

        assert_eq!(result.len(), 1);
        assert!(result.get(0).unwrap().is_data());

        //second payload is chunked
    }

    #[test]
    pub fn test_session_close_body() {
        let command: [u8; 2048] = [238, 1, 0, 0, 8, 0, 1, 230, 186, 128, 69, 48, 8, 1, 0, 2, 0, 0, 0, 0, 66, 89, 69, 32, 77, 83, 78, 77, 83, 71, 82, 58, 97, 101, 111, 110, 116, 101, 115, 116, 51, 64, 115, 104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 55, 55, 99, 52, 54, 97, 56, 102, 45, 51, 51, 97, 51, 45, 53, 50, 56, 50, 45, 57, 97, 53, 100, 45, 57, 48, 53, 101, 99, 100, 51, 101, 98, 48, 54, 57, 125, 32, 77, 83, 78, 83, 76, 80, 47, 49, 46, 48, 13, 10, 84, 111, 58, 32, 60, 109, 115, 110, 109, 115, 103, 114, 58, 97, 101, 111, 110, 116, 101, 115, 116, 51, 64, 115, 104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 55, 55, 99, 52, 54, 97, 56, 102, 45, 51, 51, 97, 51, 45, 53, 50, 56, 50, 45, 57, 97, 53, 100, 45, 57, 48, 53, 101, 99, 100, 51, 101, 98, 48, 54, 57, 125, 62, 13, 10, 70, 114, 111, 109, 58, 32, 60, 109, 115, 110, 109, 115, 103, 114, 58, 97, 101, 111, 110, 116, 101, 115, 116, 64, 115, 104, 108, 46, 108, 111, 99, 97, 108, 59, 123, 102, 53, 50, 57, 55, 51, 98, 54, 45, 99, 57, 50, 54, 45, 52, 98, 97, 100, 45, 57, 98, 97, 56, 45, 55, 99, 49, 101, 56, 52, 48, 101, 52, 97, 98, 48, 125, 62, 13, 10, 86, 105, 97, 58, 32, 77, 83, 78, 83, 76, 80, 47, 49, 46, 48, 47, 84, 76, 80, 32, 59, 98, 114, 97, 110, 99, 104, 61, 123, 48, 48, 52, 54, 52, 53, 48, 55, 45, 68, 57, 68, 68, 45, 52, 48, 56, 52, 45, 65, 48, 56, 65, 45, 49, 52, 50, 52, 57, 53, 54, 56, 69, 69, 56, 54, 125, 13, 10, 67, 83, 101, 113, 58, 32, 48, 32, 13, 10, 67, 97, 108, 108, 45, 73, 68, 58, 32, 123, 66, 50, 69, 69, 66, 54, 48, 55, 45, 49, 69, 54, 52, 45, 52, 54, 56, 54, 45, 65, 52, 65, 49, 45, 52, 50, 54, 51, 51, 54, 65, 53, 69, 48, 65, 53, 125, 13, 10, 77, 97, 120, 45, 70, 111, 114, 119, 97, 114, 100, 115, 58, 32, 48, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 84, 121, 112, 101, 58, 32, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 47, 120, 45, 109, 115, 110, 109, 115, 103, 114, 45, 115, 101, 115, 115, 105, 111, 110, 99, 108, 111, 115, 101, 98, 111, 100, 121, 13, 10, 67, 111, 110, 116, 101, 110, 116, 45, 76, 101, 110, 103, 116, 104, 58, 32, 50, 54, 13, 10, 13, 10, 83, 101, 115, 115, 105, 111, 110, 73, 68, 58, 32, 49, 49, 55, 48, 50, 53, 52, 50, 48, 54, 13, 10, 13, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let mut parser = P2PCommandParser::new();

        let result = parser.parse_message(&command, false);

        assert_eq!(result.len(), 1);
    }

    #[test]
    pub fn test_chunks() {
        let mut chunk1 = [0u8; 2048];
        LittleEndian::write_u32(&mut chunk1,6140);


        let mut chunk2 = [1u8; 2048];
        let mut chunk3 = [2u8; 2048];

        let mut parser = P2PCommandParser::new();

        let result = parser.parse_message(&chunk1, false);
        let result = parser.parse_message(&chunk2, false);
        let result = parser.parse_message(&chunk3, false);

        assert_eq!(result.len(), 1);

    }
}

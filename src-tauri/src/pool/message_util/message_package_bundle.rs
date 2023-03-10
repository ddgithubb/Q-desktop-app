use bytes::{Bytes};
use prost::{Message};

use crate::poolpb::{PoolMessagePackage, PoolMessage, PoolChunkMessage, PoolMessagePackageSourceInfo, PoolDirectMessage};

#[derive(Clone)]
pub struct MessagePackageBundle {
    pub msg_pkg: PoolMessagePackage,
    pub encoded_msg_pkg: Bytes,
    pub from_node_id: String,
    pub is_chunk: bool,
}

impl MessagePackageBundle {
    pub fn create(msg_pkg: PoolMessagePackage, from_node_id: String) -> Self {
        let buf = msg_pkg.encode_to_vec();
        let is_chunk = msg_pkg.chunk_msg.is_some();
        MessagePackageBundle {
            msg_pkg,
            encoded_msg_pkg: Bytes::from(buf),
            from_node_id,
            is_chunk
        }
    }
    
    // Checks if target node is a dest, then re-encodes if so
    // Performance penalty is negligible for now
    pub fn check_and_update_is_dest(&mut self, target_node_id: &String) -> bool {
        for i in 0..self.msg_pkg.dests.len() {
            if &self.msg_pkg.dests[i].node_id == target_node_id {
                self.msg_pkg.dests.remove(i);
                if !self.msg_pkg.dests.is_empty() {
                    let buf = self.msg_pkg.encode_to_vec();
                    self.encoded_msg_pkg = Bytes::from(buf);
                }
                return true;
            }
        }

        false
    }

    // // Checks if target node is a dest, then re-encodes if so
    // // Performance penalty is negligible for now
    // pub fn check_and_update_is_dest(&mut self, target_node_id: &String) -> (bool, bool) {
    //     // although browser version doesn't update, should update regardless
    //     let mut found = false;
    //     let mut all_visited = true;
    //     for dest in self.msg_pkg.dests.iter_mut() {
    //         if &dest.node_id == target_node_id {
    //             dest.visited = true;
    //             found = true;
    //         }
    //         if !dest.visited {
    //             all_visited = false;
    //         }
    //     }

    //     if !all_visited {
    //         let mut buf: Vec<u8> = Vec::new();
    //         let _ = self.msg_pkg.encode(&mut buf);
    //         self.encoded_msg_pkg = Bytes::from(buf);
    //     }

    //     (found, all_visited)
    // }

    // Doesn't work
    // fn set_visited_in_encoded_msg_pkg(&mut self, target_node_id: &String) -> anyhow::Result<()> {
    //     let mut dest_index = 0;
    //     for dest in &self.msg_pkg.dests {
    //         if &dest.node_id == target_node_id {
    //             break;
    //         }
    //         dest_index += 1;
    //     }

    //     if dest_index == self.msg_pkg.dests.len() {
    //         return Err(anyhow!("no dest found"));
    //     }

    //     let mut buf = self.encoded_msg_pkg.slice(..);
    //     let len = buf.len();
    //     let ctx = DecodeContext::default();
    //     while buf.has_remaining() {
    //         let (tag, wire_type) = decode_key(&mut buf)?; // tag is field number

    //         if tag == 1 {
    //             skip_field(wire_type, tag, &mut buf, ctx.clone());
    //         } else if tag == 2 {
    //             let size = decode_varint(&mut buf)?;
    //             let end = (len - buf.remaining()) + size as usize;
    //             let mut temp_index = 0;

    //             while len - buf.remaining() < end {
    //                 let (tag, wire_type) = decode_key(&mut buf)?;

    //                 if tag == 1 {
    //                     skip_field(wire_type, tag, &mut buf, ctx.clone());
    //                 } else if tag == 2 {
    //                     if temp_index == dest_index {
    //                         let pos = len - buf.remaining();
    //                         let join = Bytes::from_static(&[1]);
    //                         self.encoded_msg_pkg = [&*self.encoded_msg_pkg.slice(..pos), &join, &*self.encoded_msg_pkg.slice(pos+1..)].concat().into();
    //                         return Ok(());
    //                     }
    //                     let _ = decode_varint(&mut buf);
    //                     temp_index += 1;
    //                 } else {
    //                     skip_field(wire_type, tag, &mut buf, ctx.clone());
    //                 }
    //             }
    //         } else {
    //             break;
    //         }
    //     }

    //     Err(anyhow!("no dest found"))
    // }

    // pub fn has_more_dest(&self) -> bool {
    //     for dest in &self.msg_pkg.dests {
    //         if !dest.visited {
    //             return true;
    //         }
    //     }
    //     return false;
    // }

    pub fn take_msg(&mut self) -> PoolMessage {
        self.msg_pkg.msg.take().unwrap()
    }

    pub fn take_direct_msg(&mut self) -> PoolDirectMessage {
        self.msg_pkg.direct_msg.take().unwrap()
    }

    pub fn take_chunk_msg(&mut self) -> PoolChunkMessage {
        self.msg_pkg.chunk_msg.take().unwrap()
    }
    
    pub fn take_src(&mut self) -> PoolMessagePackageSourceInfo {
        self.msg_pkg.src.take().unwrap()
    }
    
    pub fn src_node_id(&self) -> String {
        self.msg_pkg.src.as_ref().unwrap().node_id.clone()
    }

}

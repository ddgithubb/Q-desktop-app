use crate::{poolpb::PoolMessagePackage, pool::chunk::chunk_util::{chunk_number_to_partner_int_path}};

pub trait MessageChecks {
    fn is_valid(&self) -> bool;
    fn is_valid_message(&self) -> bool;
    fn is_valid_direct_message(&self) -> bool;
    fn is_valid_chunk(&self) -> bool;
}

impl MessageChecks for PoolMessagePackage {
    fn is_valid(&self) -> bool {
        if let Some(src) = &self.src {
            if !src.node_id.is_empty() && !src.path.is_empty() {
                return true;
            }
        }
        false
    }

    fn is_valid_message(&self) -> bool {
        if !self.is_valid() {
            return false;
        }

        let msg = match &self.msg {
            Some(msg) => msg,
            None => return false,
        };

        if msg.msg_id.is_empty() || msg.created == 0 || msg.r#type < 0 || msg.user_id.is_empty() {
            return false;
        }

        return true;
    }

    fn is_valid_direct_message(&self) -> bool {
        if !self.is_valid() {
            return false;
        }

        let _ = match &self.direct_msg {
            Some(direct_msg) => direct_msg,
            None => return false,
        };

        return true;
    }

    fn is_valid_chunk(&self) -> bool {
        if !self.is_valid() {
            return false;
        }

        let chunk_info = match &self.chunk_msg {
            Some(chunk_info) => chunk_info,
            None => return false,
        };

        if chunk_info.file_id.is_empty() {
            return false;
        }

        if let Some(partner_int_path) = self.partner_int_path {
            if chunk_number_to_partner_int_path(chunk_info.chunk_number) != partner_int_path {
                return false;
            }
        }

        return true;
    }

}

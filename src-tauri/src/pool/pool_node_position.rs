use crate::sspb::ss_message::UpdateNodePositionData;

pub(super) type PoolPanelNodeIDs = [Option<String>; 3];

#[derive(Default, Clone)]
pub(super) struct PoolNodePosition {
    pub(super) path: Vec<u32>,
    pub(super) partner_int: usize,
    pub(super) panel_number: usize,
    pub(super) center_cluster: bool,
    pub(super) parent_cluster_node_ids: [PoolPanelNodeIDs; 3],
    pub(super) child_cluster_node_ids: [PoolPanelNodeIDs; 3],
}

impl PoolNodePosition {
    pub(super) fn from_update_node_position_data(
        update_node_position_data: UpdateNodePositionData,
    ) -> Self {
        let mut parent_cluster_node_ids: [PoolPanelNodeIDs; 3] = Default::default();
        let mut child_cluster_node_ids: [PoolPanelNodeIDs; 3] = Default::default();
        let mut cur_index = 0;
        for i in 0..3 {
            for j in 0..3 {
                if let Some(node_id) = update_node_position_data
                    .parent_cluster_node_ids
                    .get(cur_index)
                {
                    if !node_id.is_empty() {
                        parent_cluster_node_ids[i][j] = Some(node_id.clone());
                    }
                }
                cur_index += 1;
            }
        }
        let mut cur_index = 0;
        for i in 0..2 {
            for j in 0..3 {
                if let Some(node_id) = update_node_position_data
                    .child_cluster_node_ids
                    .get(cur_index)
                {
                    if !node_id.is_empty() {
                        child_cluster_node_ids[i][j] = Some(node_id.clone());
                    }
                }
                cur_index += 1;
            }
        }
        let panel_number = update_node_position_data.path.last().copied().unwrap(); // Will panic if Sync Server is not compliant
        PoolNodePosition {
            path: update_node_position_data.path,
            partner_int: update_node_position_data.partner_int as usize,
            panel_number: panel_number as usize,
            center_cluster: update_node_position_data.center_cluster,
            parent_cluster_node_ids,
            child_cluster_node_ids,
        }
    }

    pub(super) fn _get_position_of_node(&self, node_id: &String) -> Option<usize> {
        let mut position = 0;
        for i in 0..3 {
            for j in 0..3 {
                if let Some(id) = &self.parent_cluster_node_ids[i][j] {
                    if id == node_id {
                        return Some(position);
                    }
                }
                position += 1;
            }
        }
        for i in 0..2 {
            for j in 0..3 {
                if let Some(id) = &self.child_cluster_node_ids[i][j] {
                    if id == node_id {
                        return Some(position);
                    }
                }
                position += 1;
            }
        }
        return None
    }
}

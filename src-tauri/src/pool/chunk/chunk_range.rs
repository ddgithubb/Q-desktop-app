use crate::poolpb::PoolChunkRange;

impl PoolChunkRange {
    pub fn has_chunk(&self, chunk_number: u64) -> bool {
        return chunk_number >= self.start && chunk_number <= self.end;
    }
}

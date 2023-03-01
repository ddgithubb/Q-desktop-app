use std::cmp::{max, min};
use std::collections::HashMap;

use crate::poolpb::PoolChunkRange;

use super::chunk_util::{
    cache_chunk_number_to_chunk_number, cache_chunk_number_to_partner_int_path,
    chunk_number_to_cache_chunk_number, total_size_to_total_chunks,
};

pub type ChunkRanges = Vec<PoolChunkRange>;
pub type StaticChunkRanges = [PoolChunkRange];

pub fn create_full_chunk_range(total_size: u64) -> ChunkRanges {
    vec![PoolChunkRange {
        start: 0,
        end: total_size_to_total_chunks(total_size) - 1,
    }]
}

pub trait ChunkRangesUtil {
    fn compact(&mut self);

    fn add(&mut self, chunk_range: &PoolChunkRange);

    fn add_chunk(&mut self, chunk_number: u64);

    fn remove_cache_chunk(&mut self, cache_chunk_number: u64);

    fn search(&self, chunk_range_start: u64) -> i64;

    fn diff(&self, diff_chunk_ranges: &StaticChunkRanges) -> ChunkRanges;

    fn intersection(&self, chunk_ranges: &StaticChunkRanges) -> ChunkRanges;

    fn promise_valid_chunks(
        &self,
        promise_chunks: &StaticChunkRanges,
        promised_chunks: &mut ChunkRanges,
        partner_int_path: u32,
    ) -> ChunkRanges;

    fn map_promised(&self, promised_map: &mut HashMap<u64, ChunkRanges>);

    fn has_chunk(&self, chunk_number: u64) -> bool;

    fn find_chunk_range(&self, chunk_number: u64) -> Option<&PoolChunkRange>;
}

impl ChunkRangesUtil for ChunkRanges {
    // Sorts and compacts chunk ranges
    fn compact(&mut self) {
        let mut compacted = true;

        for i in (0..self.len() - 1).rev() {
            if self[i + 1].start > self[i + 1].end {
                self.remove(i + 1);
            }

            if self[i].start > self[i].end {
                self.remove(i);
            }

            if self[i].end + 1 >= self[i + 1].start {
                compacted = false;
            }
        }

        if compacted {
            return;
        }

        self.sort_by(|range1, range2| range1.end.cmp(&range2.end));
        for i in (1..self.len()).rev() {
            if self[i - 1].end + 1 >= self[i].start {
                if self[i].start < self[i - 1].start {
                    self[i - 1].start = self[i].start;
                }
                self[i - 1].end = self[i].end;
                self.remove(i);
            }
        }
    }

    // Adds chunk range
    // Precondition: compacted
    fn add(&mut self, chunk_range: &PoolChunkRange) {
        if self.is_empty() {
            self.push(chunk_range.clone());
            return;
        }

        let pos = self.search(chunk_range.start);
        let pos: usize = if pos < 0 {
            let pos = (-pos - 1) as usize;
            if pos != 0 && self[pos - 1].end + 1 >= chunk_range.start {
                self[pos - 1] = PoolChunkRange {
                    start: self[pos - 1].start,
                    end: chunk_range.end,
                };
                pos - 1
            } else if pos < self.len() && self[pos].start - 1 <= chunk_range.end {
                self[pos] = PoolChunkRange {
                    start: chunk_range.start,
                    end: max(chunk_range.end, self[pos].end),
                };
                pos
            } else {
                self.insert(pos, chunk_range.clone());
                return;
            }
        } else {
            let pos = pos as usize;
            if chunk_range.end <= self[pos].end {
                return;
            }
            self[pos] = PoolChunkRange {
                start: self[pos].start,
                end: chunk_range.end,
            };
            pos
        };

        let i = pos + 1;
        let mut cur_len = self.len();
        while i < cur_len {
            if self[i - 1].end + 1 >= self[i].start {
                if self[i].end > self[i - 1].end {
                    self[i - 1] = PoolChunkRange {
                        start: self[i - 1].start,
                        end: self[i].end,
                    };
                }
                self.remove(i);
                cur_len -= 1;
            } else {
                break;
            }
        }
    }

    // Precondition: compacted
    fn add_chunk(&mut self, chunk_number: u64) {
        self.add(&PoolChunkRange {
            start: chunk_number,
            end: chunk_number,
        })
    }

    // Removes all chunk ranges within a cache chunk
    // Precondition: chunkRanges abide within a cacheChunkNumber
    fn remove_cache_chunk(&mut self, cache_chunk_number: u64) {
        let pos = self.search(cache_chunk_number_to_chunk_number(cache_chunk_number));
        let pos: usize = if pos < 0 { -pos - 1 } else { pos } as usize;

        let mut del_count = 0;
        for i in pos as usize..self.len() {
            if chunk_number_to_cache_chunk_number(self[i].start) != cache_chunk_number {
                break;
            }
            del_count += 1;
        }

        if del_count != 0 {
            let _ = self.drain(pos..pos + del_count);
        }
    }

    // Searches chunk ranges for chunk range start
    // https://stackoverflow.com/questions/22697936/binary-search-in-javascript
    // pos is at beginning of multiple same values
    // (-pos - 1) is where it should go if value not found
    fn search(&self, chunk_range_start: u64) -> i64 {
        let mut m: i64 = 0;
        let mut n: i64 = self.len() as i64 - 1;
        while m <= n {
            let k = (n + m) >> 1;
            let cmp: i64 = chunk_range_start as i64 - self[k as usize].start as i64;
            if cmp > 0 {
                m = k + 1;
            } else if cmp < 0 {
                n = k - 1;
            } else {
                return k;
            }
        }
        return -m - 1;
    }

    // Creates new chunk ranges of self & !diff_chunk_ranges
    fn diff(&self, diff_chunk_ranges: &StaticChunkRanges) -> ChunkRanges {
        let i1_len = self.len();
        let i2_len = diff_chunk_ranges.len();

        let mut diff: ChunkRanges = Vec::with_capacity(i1_len);

        if i1_len == 0 {
            return diff;
        }

        let mut i1 = 0;
        let mut i2 = 0;
        let mut start = self[0].start;
        let mut end = self[0].end;
        while i1 < i1_len {
            while i2 < i2_len {
                if PoolChunkRange::has_chunk(&diff_chunk_ranges[i2], start) {
                    start = diff_chunk_ranges[i2].end + 1;
                    break;
                }
                if start < diff_chunk_ranges[i2].start {
                    diff.push(PoolChunkRange {
                        start: start,
                        end: min(end, diff_chunk_ranges[i2].start - 1),
                    });
                    break;
                }
                i2 += 1;
            }

            if start > end {
                // no diff, pass
            } else if i2 >= i2_len {
                diff.push(PoolChunkRange { start, end });
            } else if end > diff_chunk_ranges[i2].end {
                start = diff_chunk_ranges[i2].end + 1;
                continue;
            }

            i1 += 1;
            if i1 < i1_len {
                start = self[i1].start;
                end = self[i1].end;
            }
        }

        diff
    }

    // Creates new chunk ranges of self & chunk_ranges
    fn intersection(&self, chunk_ranges: &StaticChunkRanges) -> ChunkRanges {
        let i1_len = self.len();
        let i2_len = chunk_ranges.len();

        let mut intersection: ChunkRanges = Vec::with_capacity(i1_len);

        let mut i1 = 0;
        let mut i2 = 0;
        while i1 < i1_len && i2 < i2_len {
            let start = max(self[i1].start, chunk_ranges[i2].start);
            let end = min(self[i1].end, chunk_ranges[i2].end);

            if start <= end {
                intersection.push(PoolChunkRange { start, end })
            }

            if self[i1].end < chunk_ranges[i2].end {
                i1 += 1;
            } else {
                i2 += 1;
            }
        }

        intersection
    }

    // Creates new chunk ranges of self & promise_chunks with partner int path restriction
    fn promise_valid_chunks(
        &self,
        promise_chunks: &StaticChunkRanges,
        promised_chunks: &mut ChunkRanges,
        partner_int_path: u32,
    ) -> ChunkRanges {
        let i1_len = self.len();
        let i2_len = promise_chunks.len();

        let mut intersection: ChunkRanges = Vec::with_capacity(i1_len);

        let mut i1 = 0;
        let mut i2 = 0;
        while i1 < i1_len && i2 < i2_len {
            let start = max(self[i1].start, promise_chunks[i2].start);
            let end = min(self[i1].end, promise_chunks[i2].end);

            if start <= end {
                for i in chunk_number_to_cache_chunk_number(start)
                    ..=chunk_number_to_cache_chunk_number(end)
                {
                    if cache_chunk_number_to_partner_int_path(i) == partner_int_path {
                        let chunk_range = PoolChunkRange {
                            start: max(start, cache_chunk_number_to_chunk_number(i)),
                            end: min(end, cache_chunk_number_to_chunk_number(i + 1) - 1),
                        };

                        // In theory, the negation should never happen
                        if chunk_range.start <= chunk_range.end {
                            intersection.push(chunk_range.clone());
                            promised_chunks.push(chunk_range);
                        }
                    }
                }
            }

            if self[i1].end < promise_chunks[i2].end {
                i1 += 1;
            } else {
                i2 += 1;
            }
        }

        intersection
    }

    // Maps promised chunks to respective cache chunk number
    // Precondition: should only contain promised chunks within their own cache chunk range
    fn map_promised(&self, promised_map: &mut HashMap<u64, ChunkRanges>) {
        for chunk_range in self {
            let cache_chunk_number = chunk_number_to_cache_chunk_number(chunk_range.start);
            if cache_chunk_number != chunk_number_to_cache_chunk_number(chunk_range.end) {
                continue;
            }

            if let Some(chunk_ranges) = promised_map.get_mut(&cache_chunk_number) {
                chunk_ranges.add(&chunk_range);
            } else {
                let mut chunk_ranges = ChunkRanges::new();
                chunk_ranges.add(&chunk_range);
                promised_map.insert(cache_chunk_number, chunk_ranges);
            }
        }
    }

    // Returns whether chunk ranges has a chunk number
    fn has_chunk(&self, chunk_number: u64) -> bool {
        for chunk_range in self {
            if PoolChunkRange::has_chunk(chunk_range, chunk_number) {
                return true;
            }
        }
        false
    }

    // Gets chunk range with chunk number
    fn find_chunk_range(&self, chunk_number: u64) -> Option<&PoolChunkRange> {
        for chunk_range in self {
            if PoolChunkRange::has_chunk(chunk_range, chunk_number) {
                return Some(&chunk_range);
            }
        }
        None
    }
}

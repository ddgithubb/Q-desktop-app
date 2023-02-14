use crate::{config::{CACHE_CHUNK_SIZE, CACHE_CHUNK_TO_CHUNK_SIZE_FACTOR, CHUNK_SIZE}};

const CHUNK_SIZE_AS_U64: u64 = CHUNK_SIZE as u64;
const CACHE_CHUNK_TO_CHUNK_SIZE_FACTOR_AS_U64: u64 = CACHE_CHUNK_TO_CHUNK_SIZE_FACTOR as u64;
const _CACHE_CHUNK_SIZE_AS_U64: u64 = CACHE_CHUNK_SIZE as u64;

pub fn chunk_number_to_cache_chunk_number(chunk_number: u64) -> u64 {
    chunk_number / CACHE_CHUNK_TO_CHUNK_SIZE_FACTOR_AS_U64
}

pub fn _byte_size_to_cache_chunk_number(byte_size: u64) -> u64 {
    byte_size / _CACHE_CHUNK_SIZE_AS_U64
}

pub fn cache_chunk_number_to_chunk_number(cache_chunk_number: u64) -> u64 {
    return cache_chunk_number * CACHE_CHUNK_TO_CHUNK_SIZE_FACTOR_AS_U64
}

pub fn cache_chunk_number_to_partner_int_path(cache_chunk_number: u64) -> u32 {
    (cache_chunk_number % 3) as u32
}

pub fn chunk_number_to_partner_int_path(chunk_number: u64) -> u32 {
    ((chunk_number / CACHE_CHUNK_TO_CHUNK_SIZE_FACTOR_AS_U64) % 3) as u32
}

pub fn total_size_to_total_chunks(total_size: u64) -> u64 {
    (total_size + (CHUNK_SIZE_AS_U64) - 1) / CHUNK_SIZE_AS_U64
}
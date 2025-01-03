pub fn compute_checksum(data: &[u8], offset: usize) -> u32 {
    const CHECKSUM_CHUNK_SIZE: usize = size_of::<u32>();

    let skip_offset = offset/CHECKSUM_CHUNK_SIZE;
    let mut checksum = 0u32;

    for (i, chunk) in data.chunks_exact(CHECKSUM_CHUNK_SIZE).enumerate() {
        if i != skip_offset {
            let v = u32::from_be_bytes(chunk.try_into().unwrap());
            (checksum, _) = checksum.overflowing_add(v);
        }
    }
    checksum.wrapping_neg()
}

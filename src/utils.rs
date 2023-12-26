pub fn leaf_to_num(leaf: &[u8; 32]) -> u64 {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&leaf[24..32]);
    u64::from_le_bytes(buf)
}
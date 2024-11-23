#[repr(u8)]
pub enum BlockType {
    Header = 2,
    Data   = 8,
    List   = 16,
}

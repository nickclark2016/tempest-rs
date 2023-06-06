pub trait SparseTableIndex: Clone + Copy + Eq + PartialEq {
    fn index(self) -> u32;
    fn tombstone() -> Self;
    fn from_raw(value: u32) -> Self;
}
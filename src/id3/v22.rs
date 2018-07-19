bitflags! {
   pub(super) struct TagFlags: u8 {
      const UNSYNCHRONIZED = 0b1000_0000;
      const COMPRESSED = 0b0100_0000;
   }
}

bitflags! {
   pub(super) struct FrameFlags: u16 {
      // Status
      const TAG_ALTER_PRESERVATION = 0b1000_0000_0000_0000;
      const FILE_ALTER_PRESERVATION = 0b0100_0000_0000_0000;
      const READ_ONLY = 0b0010_0000_0000_0000;

      // Format
      const COMPRESSION = 0b0000_0000_1000_0000;
      const ENCRYPTION = 0b0000_0000_0100_0000;
      const GROUPING_IDENTITY = 0b0000_0000_0010_0000;
   }
}

bitflags! {
   pub(super) struct TagFlags: u8 {
      const UNSYNCHRONIZED = 0b1000_0000;
      const EXTENDED_HEADER = 0b0100_0000;
      const EXPERIMENTAL_INDICATOR = 0b0010_0000;
   }
}

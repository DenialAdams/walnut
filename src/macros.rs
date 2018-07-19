#[macro_export]
macro_rules! wtry {
   ($x:expr) => {{
      match $x {
         Err(e) => return Some(Err(e)),
         Ok(v) => v,
      }
   }};
}

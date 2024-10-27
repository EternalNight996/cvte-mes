macro_rules! cfg_api {
  ($($item:item)*) => {
      $(
          #[cfg(feature = "api")]
          $item
      )*
  }
}

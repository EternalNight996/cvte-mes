macro_rules! cfg_api {
  ($($item:item)*) => {
      $(
          #[cfg(feature = "api")]
          $item
      )*
  }
}

/// 定义枚举的宏
macro_rules! define_enum {
  ($name:ident, $($variant:ident = $value:expr),* $(,)?) => {
      #[derive(Debug, Deserialize, Serialize, FromRepr, Clone)]
      #[serde(from = "i32", into = "i32")]
      #[repr(i32)]
      pub enum $name {
          $($variant = $value),*
      }

      impl From<i32> for $name {
          fn from(value: i32) -> Self {
              Self::from_repr(value).unwrap_or_else(|| Self::from_repr(0).unwrap())
          }
      }

      impl From<$name> for i32 {
          fn from(value: $name) -> i32 {
              value as i32
          }
      }
  };
}

use serde::{Deserialize, Serialize};

pub mod cmes;
pub mod scm;

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum CVTE {
  #[default]
  CMES,
  SCM,
}

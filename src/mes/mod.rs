use serde::{Deserialize, Serialize};
use strum::{Display, VariantArray};

pub mod cmes;
pub mod itapi;
pub mod scm;

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, VariantArray, Display)]
pub enum CVTE {
  #[default]
  CMES,
  SCM,
}

use serde::{Deserialize, Serialize};

pub mod cmes;
pub mod scm;

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum CVTE {
  #[default]
  CMES,
  SCM,
}

//! MES 接口统一入口
//!
//! | 模块 | 系统 | 说明 |
//! |------|------|------|
//! | [`cmes`] | CMES | 登录、SN 校验、SN 信息、报工、测试项查询、过站 |
//! | [`scm`] | SCM WIP | 工单信息、投料单信息（MD5 网关签名） |
//! | [`itapi`] | ItAPI（客户 MES 主系统） | CPLM 生产文件、软件信息表 |
//! | [`mac`] | SN→MAC | 防混料用 MAC 绑定查询（兼容 CMES / ItAPI 鉴权） |
//!
//! 需要同步调用时启用 `blocking` feature 使用 [`crate::blocking`]。
use serde::{Deserialize, Serialize};
use strum::{Display, VariantArray};

pub mod cmes;
pub mod itapi;
pub mod mac;
pub mod scm;

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, VariantArray, Display)]
pub enum CVTE {
  #[default]
  CMES,
  SCM,
}

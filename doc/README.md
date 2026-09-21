# MES 接口文档（本地归档）

对接方提供的接口文档，已转成 Markdown 便于检索与核对。

> ⚠️ **本地留存，不进公开仓库。** `CVTE_SCM_WIP接口文档v1.0.md` 原文首部标注「文档密级：合作伙伴」，
> `CMES对外接口对接标准.md` 亦为对接方内部标准文档，因此这三份均已加入 `.gitignore`。
> 如需随仓库发布，请先确认密级与授权，再删除对应忽略规则。

## 文件

| 文件 | 来源 | 内容 |
| --- | --- | --- |
| `CMES对外接口对接标准.md` | `F:\MyApp\MES开发接口\CMES对外接口对接标准-整理版.md` | CMES 通用对接标准（Token / 响应解析 / 重试 / 安全）+ 2 个业务接口：组件关联列表、SN 包装完工数 |
| `CVTE_SCM_WIP接口文档v1.0.md` | 同名 docx（`anydoc` 转出） | SCM WIP 网关说明、MD5 签名算法、查询工单信息、查询投料单信息 |
| `ItAPI接口文档.md` | 同名 docx（`anydoc` 骨架 + **Windows OCR** 截图转写） | 批量查询 CPLM 生产文件、软件信息表 v2 |

原始 PDF、未整理的 md、docx 原件仍在 `F:\MyApp\MES开发接口`（未复制进本目录）。

> ItAPI 那份 docx 正文几乎全是截图，`anydoc` 只能提取文字骨架，字段表是用 Windows 内置 OCR 识别内嵌图片得到的；
> OCR 有少量误识（如 `PDT01` → `PDTØI`），**字段名以英文原样为准**。

## 与实现的核对结论（2026-09-20）

文档中出现的每个接口，`cvte-mes` 均已实现，且字段级一致：

| 文档接口 | CLI task | 库函数 | 核对 |
| --- | --- | --- | --- |
| 登录获取 JWT Token | `User:CreateToken` | `login::login` | 端点/参数一致（密码规则见下） |
| 按产品条码获取组件关联列表 | `Pack:GetOtherListBySn` | `pack::get_other_list_by_sn` | 入参 1/1、返参 16/16 |
| 按批次获取 SN 包装完工数 | `Pack:GetSnPackFinishQty` | `pack::get_sn_pack_finish_qty` | 入参 1/1、返参 3/3 |
| 查询 SCM 工单信息–工厂 | `SCM:MoFactory` | `scm::wip::query_mo_factory` | 入参 3/3、返参 29/29、签名一致 |
| 查询 SCM 投料单信息–工厂 | `SCM:ReqCommon` | `scm::wip::query_req_common` | 入参 2/2、返参 13/13 |
| 批量查询 CPLM 生产文件 | `ItAPI:BatchProdDocs` | `itapi::batch_production_documents` | 入参 7/7、返参 17/17 |
| 软件信息表 v2 | `ItAPI:SoftInfoListV2` | `itapi::soft_info_list_v2` | 入参 2/2、返参 19/19 |

通用约定（标准 §4–§7）逐条覆盖：Token 缓存 2h + 到期前刷新、401/`unAuthorizedRequest` 重取后**最多重试一次**、
500 / 401 / 200+success=false / 200+success=true / 其它状态 / 非 JSON 六层解析、`x-auth-token` 头（现场 `tokenName` 覆盖）、
凭据不入日志 —— 由 `--task SelfTest:Session` 的内置自检实测（见 README「内置自检」）。

## 已知不一致 / 待确认（2026-09-20）

1. **登录密码规则**：标准 §3.2 要求 `Pwd` 为 32 位小写 MD5 摘要；实现中账号**含 `@`** 走 MD5（合规），
   **不含 `@`** 走 Base64（与文档矛盾）。现场若仍有非 `@` 账号，需确认。
2. **硬编码测试凭证**：`src/mes/scm/sign.rs` 的 `DefaultSignSDK` 内含 SIT 网关测试 `apikey/api_secret` 常量，建议移除或改为占位。
3. **限流未处理**：标准 §9.3 声明按 `moLotNo` 维度限流（默认 10 秒窗口 30 次），实现未做客户端退避（文档也未约定返回码）。
4. **文档未覆盖的接口**：CMES 另有 11 个接口（过站校验、按 SN 查批次、报工、SPI 过站、投料单、`MoLotNo`/`Report` 组合等）
   不在这三份文档内，来源为现场提供；若拿到正式文档可再做一轮字段级核对。

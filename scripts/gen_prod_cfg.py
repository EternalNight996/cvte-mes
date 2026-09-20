"""从 etest.db 读取生产 MES 配置，生成 scripts/verify.prod.bat（含明文凭据，已被 .gitignore 忽略）

用法：
  python scripts/gen_prod_cfg.py --list                 # 只列出 etest.db 里的候选配置
  python scripts/gen_prod_cfg.py                        # 自动取字段最完整的一条生成
  python scripts/gen_prod_cfg.py --id 1                 # 指定 org_cvte_cmes.id
  python scripts/gen_prod_cfg.py --id 1 --sn A --lot B --sn2 C --lot2 D --parent-id E
  python scripts/gen_prod_cfg.py --db <etest.db 路径> --out <输出文件>

生成的 verify.prod.bat 会被 scripts/verify.bat / verify2.bat 自动加载（存在即生效），
因此换账号/换工厂时不必手改脚本本体。
"""

import argparse
import os
import sqlite3
import sys

DEFAULT_DB = r"F:\MyApp\eternal\etest\etest.db"
DEFAULT_OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "verify.prod.bat")


def die(msg):
    print(f"[ERROR] {msg}", file=sys.stderr)
    sys.exit(1)


def load_rows(db):
    if not os.path.exists(db):
        die(f"找不到 etest.db：{db}（用 --db 指定）")
    con = sqlite3.connect(f"file:{db}?mode=ro", uri=True)
    con.row_factory = sqlite3.Row
    rows = [
        dict(r)
        for r in con.execute(
            "select id, org_tag, station_name, station_code, prod_url, sys_url, username,"
            " password, token_name, cause_no, work_center, test_frameid_no"
            " from org_cvte_cmes order by id"
        )
    ]
    con.close()
    return rows


def front_url_from(prod_url):
    """由 prod 域名推导前端 API 根： http://heg-pc-cmesprod.gz.cvte.cn -> http://heg-pc-cmes.gz.cvte.cn/frontApi/prod"""
    if not prod_url or "cmesprod" not in prod_url:
        return ""
    return prod_url.replace("-cmesprod.", "-cmes.").rstrip("/") + "/frontApi/prod"


def pick(rows, wanted_id=None):
    usable = [r for r in rows if (r.get("prod_url") or "").strip() and (r.get("username") or "").strip()]
    if not usable:
        die("etest.db 的 org_cvte_cmes 里没有可用配置（prod_url / username 为空）")
    if wanted_id is not None:
        for r in usable:
            if int(r["id"]) == int(wanted_id):
                return r
        die(f"org_cvte_cmes 里没有 id={wanted_id} 的可用配置，用 --list 看候选")
    return usable[-1]


def render(row, args):
    lines = [
        "@echo off",
        "REM 由 scripts/gen_prod_cfg.py 从 etest.db 生成 —— 含明文凭据，勿提交/外发",
        f"REM 来源：org_cvte_cmes.id={row['id']}  站点={row.get('station_name') or ''}  企业={row.get('org_tag') or ''}",
    ]
    mapping = [
        ("PROD_URL", row.get("prod_url")),
        ("SYS_URL", row.get("sys_url")),
        ("USER_NAME", row.get("username")),
        ("PASS_WORD", row.get("password")),
        ("TOKEN_NAME", row.get("token_name")),
        ("CAUSE_NO", row.get("cause_no")),
        ("WORK_CENTER", row.get("work_center")),
        ("TEST_FRAME_ID", row.get("test_frameid_no")),
        ("STATION_CODE", row.get("station_code")),
    ]
    for key, value in mapping:
        lines.append(f'set "{key}={value if value is not None else ""}"')
    lines.append(f'set "FRONT_URL={front_url_from(row.get("prod_url") or "")}"')
    # 数据可按参数覆盖（缺省不动脚本里已有的值）
    for key, value in (
        ("SN", args.sn),
        ("SN2", args.sn2),
        ("LOT", args.lot),
        ("LOT2", args.lot2),
        ("PARENT_ID", args.parent_id),
    ):
        if value:
            lines.append(f'set "{key}={value}"')
    return "\r\n".join(lines) + "\r\n"


def main():
    ap = argparse.ArgumentParser(description="从 etest.db 生成 verify.prod.bat")
    ap.add_argument("--db", default=DEFAULT_DB)
    ap.add_argument("--out", default=DEFAULT_OUT)
    ap.add_argument("--id", type=int, default=None, help="org_cvte_cmes.id")
    ap.add_argument("--list", action="store_true", help="只列出候选配置")
    ap.add_argument("--sn")
    ap.add_argument("--sn2")
    ap.add_argument("--lot")
    ap.add_argument("--lot2")
    ap.add_argument("--parent-id", dest="parent_id")
    args = ap.parse_args()

    rows = load_rows(args.db)

    if args.list:
        print(f"etest.db: {args.db}")
        for r in rows:
            usable = "可用" if (r.get("prod_url") or "").strip() and (r.get("username") or "").strip() else "不完整"
            print(
                f"  id={r['id']:<3} [{usable}] {r.get('station_name') or '':<8} "
                f"{r.get('prod_url') or '':<40} {r.get('username') or ''}"
            )
        return

    row = pick(rows, args.id)
    text = render(row, args)
    with open(args.out, "w", encoding="utf-8", newline="") as f:
        f.write(text)
    print(f"[OK] 已生成 {args.out}")
    print(f"     来源 id={row['id']}  站点={row.get('station_name')}  {row.get('prod_url')}  账号={row.get('username')}")
    print("     该文件含明文密码，已在 .gitignore 中忽略；脚本会自动加载它。")


if __name__ == "__main__":
    main()

# pyright: basic
"""Non-authoritative, non-publishable seed senses for bootstrap matching."""

from __future__ import annotations

from typing import Any


def _sense(gloss: str, pos: str, example: str) -> dict[str, Any]:
    return {
        "gloss": gloss,
        "pos": pos,
        "example": example,
        "source_kind": "internal_seed",
        "is_authoritative": False,
        "is_publishable": False,
    }


SEED_SENSES = {
    "打": [
        _sense("用手或器具接触后施加力量；击打", "v", "他打了那个球。"),
        _sense("拨打电话", "v", "打个电话过去。"),
        _sense("进行某种活动", "v", "打篮球，打太极。"),
        _sense("从某处获取", "v", "去井里打水。"),
        _sense("制作、建造", "v", "打家具，打毛衣。"),
    ],
    "开": [
        _sense("使关闭的东西打开", "v", "开门，开窗。"),
        _sense("启动运转", "v", "开车，开机器。"),
        _sense("举办、进行", "v", "开会，开展览。"),
        _sense("开列、书写", "v", "开发票，开处方。"),
        _sense("扩展、扩大", "v", "开阔视野，开疆拓土。"),
    ],
    "发": [
        _sense("送出、发送", "v", "发邮件，发信息。"),
        _sense("产生、出现", "v", "发现问题，发生变化。"),
        _sense("生长", "v", "发芽，发育。"),
        _sense("散发、放出", "v", "发光，发热。"),
        _sense("财富增加", "v", "发财，发家致富。"),
    ],
    "上": [
        _sense("方位词：上面，较高处", "n", "桌子上有一本书。"),
        _sense("向上移动或前往", "v", "上山，上楼，上车。"),
        _sense("去某地工作或学习", "v", "上班，上学，上课。"),
        _sense("呈递、登载", "v", "上报，上架，上菜。"),
    ],
    "下": [
        _sense("方位词：下面，较低处", "n", "桌子下有一只猫。"),
        _sense("向下移动", "v", "下山，下楼，下车。"),
        _sense("离开工作或学习场所", "v", "下班，下课。"),
        _sense("实施某动作", "v", "下令，下决心，下手。"),
    ],
}

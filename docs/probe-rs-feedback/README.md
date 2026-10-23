# probe-rs 上游反馈包

2026-10-24 实机验证时发现的两个可提给 probe-rs 上游的改进点。
背景:CW32L0 系列**自 0.32 起已内置**(无需提交目标定义 yaml),
以下都是使用体验层面的意见。

## 文件

| 文件 | 主题 | 一句话 |
|------|------|--------|
| `issue-auto-detection.md` | 芯片自动识别 | CW32L012 已内置却识别不出,报错还像"不支持" |
| `issue-protocol-hint.md` | 协议默认值 UX | 不写 `--protocol swd` 时报错无提示,SWD-only 板子像坏了一样 |

两份都是**英文 issue 草稿**(上游是国际项目),直接复制正文提交即可。

## 去哪里提交

1. **新 issue**(两份草稿的去处):
   https://github.com/probe-rs/probe-rs/issues/new
2. **已有讨论串**——自动识别这条建议在正文里引用:
   [Automatic chip detection · probe-rs#2449](https://github.com/probe-rs/probe-rs/issues/2449)
   (也可以选择只去 #2449 底下评论我们的 CW32 实测数据,不另开新 issue)
3. **偏讨论性质的想法**(比如协议自动回退要不要做):
   https://github.com/probe-rs/probe-rs/discussions
4. **实时社区**(提问/找维护者确认意向,提 PR 前先打招呼):
   Matrix 频道 `#probe-rs:matrix.org`

## 提交前小贴士

- 一个 issue 一个主题:两份草稿分开提,别合并;
- `issue-auto-detection.md` 正文已引用 #2449,不用重复贴链接;
- 我们的全部命令输出都来自实机(probe-rs 0.32.0 + ST-Link V2 +
  CW32L012x8),上游若要复现,芯片电商平台可购,DFP 包
  `WHXY.CW32L012_DFP` 1.0.2 可从厂商官网下载。

## 以后要给 probe-rs 加新芯片系列呢

不用提 yaml 的 PR 也行——但标准通道是:fork
[probe-rs/probe-rs](https://github.com/probe-rs/probe-rs),往 `targets/`
目录加目标定义(可用 `target-gen` 从厂商 DFP 生成),提 PR。
CW32L0 系列已经在库里了,这条仅作备忘。

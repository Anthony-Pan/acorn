# Acorn 🌰

> 把今天的事,一颗颗收进来。

Acorn 是一个温暖的开源 AI 桌面助手。把脑子里乱糟糟的一天倒给它,它帮你拆成任务卡片,然后陪你一项一项做完。

[![MIT License](https://img.shields.io/badge/license-MIT-8B4513.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-B5803F.svg)](https://tauri.app)
[![Stars](https://img.shields.io/github/stars/onyxcraft/acorn?style=social)](https://github.com/onyxcraft/acorn)

[English README](README.md) · [架构说明](ARCHITECTURE.md) · [贡献指南](CONTRIBUTING.md)

---

## 它能做什么

打开 Acorn,把脑子里在转的事情一股脑写进去(或者按住麦克风说出来)—— *给牙医打电话、把幻灯片改完、看一下产品数据、顺路去趟银行* —— 然后点 **Stash it 🌰**。Acorn 会调用你选的 LLM,把这堆混乱拆成有序的任务卡片,带优先级和时间预估,陪你一颗颗取出来做完。

这是一个桌面 app。打开它、用它、关掉它。数据都在本地 SQLite 里。API key 在系统钥匙串里。任何不是你主动同步出去的数据,都不会离开你的电脑。

## 亮点

- **模型无关。** 内置 Anthropic、OpenAI、DeepSeek、OpenRouter、Ollama,还有 7 家备选。用你自己的 key,或者用 Ollama 跑本地 Qwen 完全免费。
- **原生桌面。** Tauri 2 + React 19。比同等 Electron 应用小 10 倍,体感也接近原生。
- **支持语音。** 按住 🎙️ 说话,Whisper 转录到输入框。
- **温暖,不冰冷。** 秋日色板,围绕真正的视觉调性设计,不是 Tailwind 默认主题。
- **数据归你。** 本地 SQLite + macOS Keychain / Windows Credential Manager / Linux Secret Service 存 key。

## 安装

### 预编译包

[Releases](https://github.com/onyxcraft/acorn/releases) 下载最新版本:

- **macOS** —— `.dmg`(Apple Silicon + Intel universal)
- **Windows** —— `.exe` 安装包
- **Linux** —— `.AppImage` 或 `.deb`

### 一键安装(macOS / Linux)

```sh
curl -fsSL https://raw.githubusercontent.com/onyxcraft/acorn/main/scripts/install.sh | bash
```

### 从源码构建

需要 [Rust](https://rustup.rs/)、[Node 20+](https://nodejs.org/)、[pnpm 10+](https://pnpm.io/)。

```sh
git clone https://github.com/onyxcraft/acorn.git
cd acorn
pnpm install
pnpm tauri dev
```

## 快速上手

1. 打开 Acorn,第一次会引导你配置一个 provider。
2. 设置 → 选一个(Anthropic Claude 是默认推荐;中国用户用 DeepSeek 最顺;Ollama 不需要 key)。
3. 把 API key 粘进去,**Save**,再点 **Make active**。
4. 回到主界面,把今天的事倒进 textarea,点 **Stash it 🌰**。

完事。

## 路线图

**v1.0(当前)**

- 多 provider AI 层 + 流式拆分
- Whisper 语音输入
- 本地 SQLite + 系统钥匙串存 key
- 今日 stash + 每个任务的生命周期(待开始 / 进行中 / 完成 / 跳过)
- 设置页含 provider 侧边栏和主题切换

**v1.1(下一个)**

- macOS 代码签名 + 公证
- Tauri updater 自动升级
- Gemini provider(请求格式不同,从 v1.0 推迟)
- 真正的流式 JSON 解析,加快首屏
- 全局快捷键唤起
- 过往日子的 stash 历史视图

**v2.0(更远)**

- Acorn Cloud(托管服务,不需要 key)
- 可选的团队/多设备同步
- Calendar / Notion / Gmail 集成
- 提醒 + 友好的轻推

## 技术栈

- [**Tauri 2**](https://tauri.app) —— 桌面外壳(Rust)
- [**React 19**](https://react.dev) + [**TypeScript 5.8**](https://www.typescriptlang.org/) —— 前端
- [**Tailwind CSS v4**](https://tailwindcss.com) + [**shadcn/ui**](https://ui.shadcn.com) —— 样式
- [**sqlx**](https://github.com/launchbadge/sqlx) —— 类型化 SQLite 访问
- [**reqwest**](https://github.com/seanmonstar/reqwest) —— LLM API 的 HTTP 客户端
- [**keyring**](https://github.com/hwchen/keyring-rs) —— 跨平台系统钥匙串
- [**Zustand**](https://zustand-demo.pmnd.rs/) + [**framer-motion**](https://www.framer.com/motion/) —— 状态 + 动画
- [**Biome**](https://biomejs.dev) —— 格式化 + lint(替代 ESLint/Prettier)

## 参与贡献

欢迎 PR。先读一下 [CONTRIBUTING.md](CONTRIBUTING.md):开发环境、提交规范、代码风格钩子。

短版本:**Conventional Commits**、CI 会跑 **`pnpm lint`** 和 **`cargo clippy -- -D warnings`**、TypeScript 里不要 `any`、生产 Rust 里不要 `.unwrap()`。

## 协议

[MIT](LICENSE) © 2026 Acorn Contributors。

## Star 历史

[![Star History Chart](https://api.star-history.com/svg?repos=onyxcraft/acorn&type=Date)](https://star-history.com/#onyxcraft/acorn&Date)

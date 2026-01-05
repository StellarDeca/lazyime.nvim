<div align="center">
  <h2>lazyime</h2>
  <p>🧠 基于语法感知的 Neovim 本地输入法自动切换插件</p>
</div>

🌍 [简体中文](./README.md)

---

## ✨ 简介

**lazyime** 是一个 Neovim 插件，用于根据光标所处的**语法上下文**，自动切换系统输入法。

它并不直接操作系统输入法，而是作为 **客户端**，连接一个本地运行的 TCP 服务端  
👉 **[LazyInputSwitcher](https://github.com/StellarDeca/LazyInputSwitcher)**（Rust 编写，低延迟、无 UI、无网络依赖）

***让你在写 英文代码 与 非英文 注释之间 无缝衔接***

---

## 🧠 工作原理概览

```
Neovim (lazyime)
        │
        │ JSON over TCP
        ▼
LazyInputSwitcher (Rust 本地服务)
        │
        ├─ Tree-sitter 语法分析
        └─ 系统输入法切换
```

- 插件在 nvim 启动 / 光标移动 / 输入状态变化时发送请求
- 服务端根据 Tree-sitter 判断光标是否位于 **代码 / 注释**
- 决定是否切换至 **English / Native 输入法**
- 全流程仅在本地完成，不进行任何互联网连接

---

## 🚀 特性

- ⚡ **低延迟**：平均请求耗时 < 5 ms
- 🌲 **Tree-sitter 驱动**的语法分析
- 💬 精确识别注释区域
- 🔄 **自动恢复**：服务端崩溃 / 断连可自动重建
- 🔐 **零隐私风险**：无互联网连接；源码不外传

---

## 💻 平台支持

| 平台      | 状态 | 说明                   |
|---------|----|----------------------|
| Windows | ✅  | 支持微软拼音（需安装 ≥2 种键盘布局） |
| Linux   | ✅  | 支持 Fcitx5            |
| macOS   | ⚠️ | 有实现但未充分测试，欢迎协助       |

---

## 📦 安装

### 使用 lazy.nvim

```lua
{
	"StellarDeca/lazyime.nvim",
	lazy = true,
	opts = {},
	event = { "VeryLazy" },
}
```

### 安装本地服务端（必需）

首次使用需要安装本地服务端，可通过以下用户命令完成：

```vim
:LazyimeInit
```

插件会：

- 根据当前操作系统与架构自动选择对应的 Release
- 下载并解压服务端至本地状态目录
- 后续由插件自动管理其生命周期

若自动安装的服务端在当前系统上无法运行，可参考：

👉 https://github.com/StellarDeca/LazyInputSwitcher

自行编译并手动放置可执行文件。

---

## 📂 文件路径说明

lazyime 使用 Neovim 的 **state 目录** 保存运行时文件：

```
stdpath("state")/lazyime/
    ├─ server/   # LazyInputSwitcher 可执行文件
    └─ logs/     # 插件运行日志
```

- 不会向用户项目目录写入任何文件
- 卸载插件后可手动删除该目录

---

## ⌨️ 行为说明

### 输入法切换规则（默认）

| 光标位置         | 输入法     |
|--------------|---------|
| 代码区域         | English |
| 注释区域         | Native  |
| 离开 Insert 模式 | English |

### 忽略的 Buffer / FileType

插件会自动忽略以下场景，以避免干扰 UI 插件：

- 非文件 buffer
- 插件窗口（Telescope / NvimTree / lazy / mason / notify / neo-tree 等）

---

## 📝 日志与隐私

插件会在本地记录运行日志，用于问题排查和行为分析：

- 日志会包含源码（用于语法分析定位）
- 所有日志仅保存在本地文件系统
- 不会上传、同步或发送至任何互联网服务

---

## 🔧 用户命令

| 命令               | 说明                   |
|------------------|----------------------|
| `:LazyimeInit`   | 下载并安装最新版本的本地服务端      |
| `:LazyimeUpdate` | 安装新版本服务端             |
| `:LazyimeUnload` | 运行时卸载插件（清除自动命令与模块缓存） |

---

## 🤝 贡献

欢迎任何形式的贡献：

- 新输入法后端支持
- 新编程语言 Tree-sitter 规则
- 架构 / 协议改进建议
- macOS 平台测试与优化

---

### 💖 贡献者

<div align="center">
    <a href="https://github.com/StellarDeca/lazyime.nvim/graphs/contributors">
        <img src="https://contrib.rocks/image?repo=StellarDeca/lazyime.nvim"  alt="Authors"/>
    </a>
</div>

---

## 📄 License

**GNU Affero General Public License v3.0 (AGPL-3.0)**

若你在网络服务中使用、修改并对外提供该程序，你有义务向使用者提供修改后的完整源代码。

---

## 🔗 相关项目

- LazyInputSwitcher（服务端）  
  https://github.com/StellarDeca/LazyInputSwitcher


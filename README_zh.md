<div align="center">
<h2>osu-mappack-creator-v2</h2>

一个基于 Tauri + Vue 3 的桌面应用，用于快速创建 osu! 曲目包（Map Pack）。

**[English](README.md)**
·
**[简体中文](README_zh.md)**
</div>


## 主要功能

- 选择你的 osu! `Songs` 目录，自动读取并分页展示谱面列表，支持搜索。
- 支持多选谱面，批量编辑 Pack 的标题 / 艺术家 / 制作者，并可单独修改每张谱面的 HP / OD。
- 自动读取谱面元数据，新建 Version 默认格式为：`Artist - Title [Creator] (Version)`。
- 一键生成谱面包，并可选择是否包含 `delete this` 文件。


## 使用方法

1. 启动程序，点击 **Select Folder**，选择你的 osu! `Songs` 目录。
2. 如果你的 Songs 目录中谱面有变动，请点击 **Refresh** 进行刷新。
3. 选择需要打包的谱面后，点击 **Next**，根据需要调整每张谱面的 Version / HP / OD。
4. 填写 Pack 的 Title / Artist / Creator（必填），选择输出目录，并决定是否包含 delete 文件。
5. 点击 **Create Pack**，即可在输出目录中生成 `.osz` 格式的谱面包。


## 日志

所有运行时错误与关键事件都会被写入以下日志文件：`%TEMP%\osu-mappack-creator-v2.log`

## 开发与构建

```bash
npm install
npx tauri dev
npx tauri build

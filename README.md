# Rust File Assistant — 部署与打包手册

> 跨平台 GUI 文件管理工具，基于 Rust + egui 构建

---

## 目录

1. [项目结构](#1-项目结构)
2. [环境准备](#2-环境准备)
3. [依赖说明](#3-依赖说明)
4. [构建运行](#4-构建运行)
5. [打包发布](#5-打包发布)
6. [功能说明](#6-功能说明)
7. [快捷键参考](#7-快捷键参考)
8. [故障排查](#8-故障排查)

---

## 1. 项目结构

```
rust-file-assistant/
├── Cargo.toml          # 项目配置与依赖
├── src/
│   ├── main.rs         # 程序入口，窗口初始化
│   ├── app.rs          # 应用状态 & 业务逻辑
│   ├── ui.rs           # 完整 GUI 渲染实现
│   ├── file_ops.rs     # 所有文件操作函数
│   └── types.rs        # 共享数据类型定义
└── MANUAL.md           # 本文档
```

---

## 2. 环境准备

### 2.1 安装 Rust 工具链

**Linux / macOS：**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Windows：**
前往 https://rustup.rs 下载 `rustup-init.exe` 并运行。

验证安装：
```bash
rustc --version    # >= 1.75.0
cargo --version
```

### 2.2 Linux 系统额外依赖

Ubuntu / Debian：
```bash
sudo apt update
sudo apt install -y \
    libgtk-3-dev \
    libglib2.0-dev \
    libcairo2-dev \
    libpango1.0-dev \
    libatk1.0-dev \
    libgdk-pixbuf2.0-dev \
    libssl-dev \
    pkg-config \
    build-essential
```

Fedora / RHEL：
```bash
sudo dnf install -y \
    gtk3-devel \
    glib2-devel \
    cairo-devel \
    pango-devel \
    atk-devel \
    gdk-pixbuf2-devel \
    openssl-devel \
    pkg-config
```

Arch Linux：
```bash
sudo pacman -S gtk3 glib2 cairo pango atk gdk-pixbuf2 openssl pkg-config
```

### 2.3 macOS 系统依赖

```bash
# 安装 Xcode 命令行工具
xcode-select --install

# 可选：通过 Homebrew 安装额外工具
brew install pkg-config
```

### 2.4 Windows 系统依赖

安装 Visual Studio Build Tools 2019+，勾选：
- **C++ build tools**
- **Windows 10 SDK**

---

## 3. 依赖说明

| 库 | 版本 | 用途 |
|---|---|---|
| `eframe` | 0.27 | 跨平台窗口 & 应用框架 |
| `egui` | 0.27 | 即时模式 GUI 渲染 |
| `walkdir` | 2.4 | 目录递归遍历 |
| `thiserror` | 1.0 | 自定义错误类型 |
| `anyhow` | 1.0 | 简化错误处理传播 |
| `chrono` | 0.4 | 时间格式化 |
| `md5` | 0.7 | 文件 MD5 哈希计算 |
| `rfd` | 0.14 | 原生文件选择对话框 |
| `open` | 5.1 | 用系统默认程序打开文件 |
| `humansize` | 2.1 | 字节数可读化格式 |

---

## 4. 构建运行

### 4.1 克隆并进入项目目录

```bash
cd rust-file-assistant
```

### 4.2 开发模式运行（快速编译，含调试信息）

```bash
cargo run
```

首次运行会下载所有依赖，约需 1-5 分钟。

### 4.3 Release 模式（优化构建，推荐发布使用）

```bash
cargo build --release
```

产物位置：
- Linux/macOS：`target/release/file-assistant`
- Windows：`target/release/file-assistant.exe`

直接运行：
```bash
# Linux/macOS
./target/release/file-assistant

# Windows
target\release\file-assistant.exe
```

---

## 5. 打包发布

### 5.1 Linux — 生成独立可执行文件

```bash
# 构建静态链接版本（可选，需 musl 工具链）
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl

# 产物（无任何动态依赖）
./target/x86_64-unknown-linux-musl/release/file-assistant
```

创建 `.desktop` 启动器：
```ini
# ~/.local/share/applications/file-assistant.desktop
[Desktop Entry]
Name=File Assistant
Comment=Rust GUI File Management Tool
Exec=/usr/local/bin/file-assistant
Icon=system-file-manager
Terminal=false
Type=Application
Categories=Utility;FileManager;
```

安装到系统：
```bash
sudo cp target/release/file-assistant /usr/local/bin/
cp file-assistant.desktop ~/.local/share/applications/
update-desktop-database ~/.local/share/applications/
```

### 5.2 macOS — 生成 .app Bundle

安装 cargo-bundle：
```bash
cargo install cargo-bundle
```

生成应用包：
```bash
cargo bundle --release
```

产物：`target/release/bundle/osx/File Assistant.app`

创建 DMG 安装包：
```bash
# 安装 create-dmg
brew install create-dmg

create-dmg \
  --volname "File Assistant" \
  --window-pos 200 120 \
  --window-size 600 300 \
  --icon-size 100 \
  --icon "File Assistant.app" 175 120 \
  --app-drop-link 425 120 \
  "FileAssistant-1.0.0.dmg" \
  "target/release/bundle/osx/"
```

通用二进制（支持 Intel + Apple Silicon）：
```bash
rustup target add x86_64-apple-darwin aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
lipo -create -output file-assistant \
  target/x86_64-apple-darwin/release/file-assistant \
  target/aarch64-apple-darwin/release/file-assistant
```

### 5.3 Windows — 生成 .exe 安装包

**方法 A：直接使用 Release 可执行文件**
```cmd
cargo build --release
# 分发 target\release\file-assistant.exe 即可
```

**方法 B：使用 cargo-wix 创建 MSI 安装包**
```cmd
cargo install cargo-wix
cargo wix init
cargo wix
# 产物：target\wix\file-assistant-1.0.0-x86_64.msi
```

**方法 C：使用 NSIS 创建安装程序**

安装 NSIS（https://nsis.sourceforge.io），创建 `installer.nsi`：
```nsis
!include "MUI2.nsh"
Name "File Assistant"
OutFile "FileAssistant-Setup.exe"
InstallDir "$PROGRAMFILES\FileAssistant"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
Section
  SetOutPath "$INSTDIR"
  File "target\release\file-assistant.exe"
  CreateShortCut "$DESKTOP\File Assistant.lnk" "$INSTDIR\file-assistant.exe"
SectionEnd
```
```cmd
makensis installer.nsi
```

### 5.4 交叉编译

在 Linux 上编译 Windows 目标：
```bash
# 安装 MinGW 工具链
sudo apt install mingw-w64
rustup target add x86_64-pc-windows-gnu

cargo build --release --target x86_64-pc-windows-gnu
# 产物：target/x86_64-pc-windows-gnu/release/file-assistant.exe
```

---

## 6. 功能说明

### 文件浏览器（Explorer 标签）

| 操作 | 说明 |
|---|---|
| 单击文件 | 选中文件，右侧显示基本信息 |
| 双击目录 | 进入该目录 |
| 双击文本文件 | 在内嵌编辑器中打开 |
| 双击其他文件 | 用系统默认程序打开 |
| 右键菜单 | 复制/剪切/粘贴/重命名/删除/属性 |
| Ctrl+单击 | 多选文件 |
| Shift+单击 | 范围选择 |
| 列标题单击 | 按该列排序（再次点击反向） |

### 批量操作（Batch Ops 标签）

1. 在文件列表中选中若干文件
2. 切换到 **Batch Ops** 标签
3. 填写 前缀/后缀/查找替换/序号 参数
4. 点击 **Preview** 预览效果
5. 确认无误后点击 **Apply** 执行

### 文件信息（File Info 标签）

选中任意文件后自动显示：
- 完整路径、文件大小（可读格式）
- 修改时间、创建时间
- 文件权限（Unix 八进制 / Windows 读写状态）
- MD5 哈希值（< 100 MB 文件自动计算）
- 文本文件行数

### 内嵌编辑器

- 支持格式：`.txt .md .rs .py .js .ts .html .css .json .toml .yaml .sh .log .xml .csv`
- 语法高亮显示
- 标题栏 `*` 标记未保存修改
- Ctrl+S（通过 Save 按钮）保存

### 操作日志（Logs 标签）

所有文件操作均有时间戳日志记录：
- ✅ 绿色：操作成功
- ⚠ 黄色：警告提示
- ❌ 红色：操作失败

---

## 7. 快捷键参考

| 快捷键 | 功能 |
|---|---|
| `Ctrl+C` | 复制选中项 |
| `Ctrl+X` | 剪切选中项 |
| `Ctrl+V` | 粘贴到当前目录 |
| `Ctrl+A` | 全选 |
| `Delete` | 删除选中项 |
| `F2` | 重命名选中项 |
| `F5` | 刷新目录 |
| `Backspace` | 返回上级目录 |
| `Enter` | 打开选中项 |

---

## 8. 故障排查

### 编译错误：找不到 GTK 头文件（Linux）

```
error: failed to run custom build command for `gtk-sys`
```
解决：
```bash
sudo apt install libgtk-3-dev pkg-config
```

### 运行时崩溃：OpenGL 不可用

```
Failed to create OpenGL context
```
解决（Linux 远程/虚拟机）：
```bash
LIBGL_ALWAYS_SOFTWARE=1 ./file-assistant
```

### Windows：MSVC 链接器错误

确保安装了 Visual Studio Build Tools 并选择了 C++ 工作负载：
```cmd
rustup toolchain install stable-x86_64-pc-windows-msvc
rustup default stable-x86_64-pc-windows-msvc
```

### macOS：代码签名警告

```bash
# 临时绕过 Gatekeeper（开发测试用）
xattr -cr "File Assistant.app"
```

### 编译速度优化

在项目根目录创建 `.cargo/config.toml`：
```toml
[build]
# 使用 mold 链接器（Linux，速度更快）
# rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[profile.dev]
opt-level = 1          # 开发模式开启基础优化
incremental = true     # 增量编译

[profile.release]
opt-level = 3
lto = "thin"           # 使用 thin LTO 加快链接
codegen-units = 4
```

---

## 版本历史

| 版本 | 日期 | 说明 |
|---|---|---|
| 1.0.0 | 2026-03 | 初始版本，完整 GUI 文件管理器 |

---
## 界面截图
<img width="1920" height="1030" alt="4e384f6e-7872-4e81-ad34-b79a9244aae9" src="https://github.com/user-attachments/assets/7b1b75b1-4796-4cc1-ba06-7b60d33e54ac" />

*MIT License — 自由使用、修改、分发*

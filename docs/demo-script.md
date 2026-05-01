# Folio 演示视频脚本

目标时长：7–8 分钟  
观众：写 Markdown 想要 Word 输出的人——研究生 / 技术博主 / 文档作者

录制前准备：
- 编译好的二进制：`cargo build --release -p scribe-cli`
- `demo/demo-cn.md`、`demo/demo-en.md`（已包含公式、表格、代码块、图片等全部功能点）
- `demo/templates/` 下的顶会模板：`acm.docx`、`ieee.docx`、`springer-lncs.docx`
- `demo/scripts/python-demo.py` / `python-demo.ipynb` 备用
- 一份干净的 macOS / Windows 桌面（演示 Folio 桌面应用）
- 终端字号调到 18pt+ 看得清

---

## 0. 冷开场（约 5 秒）

**画面**：屏幕左半 Markdown 源码、右半屏崩坏的 Word 截图（公式变图片、表格错位、代码字体不对）

**字幕**：

> Markdown 转 Word，最后 10% 总在崩。

---

## 1. 问题与解法（约 25 秒）

**画面**：纯黑底显出 Folio logo，下面三行字逐次出现

**旁白**：

> 大多数 Markdown 转 Word 的工具，公式会被压成图片、表格的样式会丢、代码字体莫名其妙。Folio 是一个用 Rust 写的小工具，把 Markdown 直接生成 **Word 原生结构**——公式可编辑、表格规整、字体可控。
>
> 今天演示三种用法：命令行、Python 代码、桌面应用。

**字幕**：

```
Folio
  Markdown → polished .docx, without the cleanup pass.
  CLI · Python · Desktop
```

---

## 2. CLI 用法（约 110 秒）

### 2.1 编译二进制（10 秒）

**画面**：终端

```bash
$ git clone https://github.com/Livia-Tassel/Folio
$ cd Folio
$ cargo build --release -p scribe-cli
$ ./target/release/scribe-cli --version
scribe-cli 0.2.1
```

**旁白**：

> 源码在 GitHub 上，clone 后一行 `cargo build --release` 编译出二进制。也可以直接到 Releases 页下载预编译包。

### 2.2 最简单的转换（20 秒）

**画面**：

```bash
$ ./target/release/scribe-cli demo/demo-en.md -o demo/outputs/demo-en.docx
wrote demo/outputs/demo-en.docx
```

切到 Finder / Explorer，双击 `demo-en.docx` 在 Word 里打开。**镜头特写**：标题层级、表格框线、行内代码字体、图片正确嵌入、然后**双击公式**——公式变成可编辑的 Word 公式编辑器。

**旁白**：

> 一行命令出 .docx。打开看：标题、表格、代码字体都对。重点——双击公式，是 Word 自己的公式编辑器，**不是图片**。这就是和 Pandoc 之类工具最大的区别。

### 2.3 列出内置主题（10 秒）

**画面**：

```bash
$ ./target/release/scribe-cli --list-themes
academic
thesis-cn
report
```

**旁白**：

> 内置三个主题：英文学术、中文论文、商务报告。

### 2.4 用 thesis-cn 主题（30 秒）

**画面**：

```bash
$ ./target/release/scribe-cli demo/demo-cn.md \
    --theme thesis-cn \
    -o demo/outputs/demo-cn.docx
wrote demo/outputs/demo-cn.docx
```

打开 `demo-cn.docx`，**镜头特写**：宋体正文、黑体标题、首行缩进 2 字符、1.5 倍行距。

**旁白**：

> 加 `--theme thesis-cn`：宋体正文、黑体标题、首行缩进、1.5 倍行距，国内本科到博士论文最大公约数都在这一份默认里了。

### 2.5 用自带模板（30 秒）

**画面**：

```bash
$ ./target/release/scribe-cli demo/demo-en.md \
    --reference-doc demo/templates/acm.docx \
    -o demo/outputs/demo-en-acm.docx
wrote demo/outputs/demo-en-acm.docx
```

打开输出，对比刚才的版本，**镜头特写**：字体、边距、页眉页脚都跟顶会模板一致（也可以换成 ieee.docx 或 springer-lncs.docx）。

**旁白**：

> `demo/templates/` 下已备好 ACM、IEEE、Springer LNCS 三份顶会模板。加 `--reference-doc demo/templates/acm.docx`，输出的字体、页边距、纸张大小，全部继承自对应模板。和 Pandoc 的 `--reference-doc` 一样的玩法，但更简单也更稳。

### 2.6 Python 端口（10 秒过渡）

**旁白**：

> 命令行先到这。如果你写 Python 脚本想集成进流水线——

---

## 3. Python 用法（约 100 秒）

### 3.1 安装（15 秒）

**画面**：

```bash
$ pip install folio-docx
Successfully installed folio-docx-0.2.1
```

**旁白**：

> `pip install folio-docx`。一份 wheel 覆盖 Python 3.8 以上所有版本，macOS、Linux、Windows 都有原生包，不需要装 Rust 工具链。

### 3.2 字符串到 .docx 字节（25 秒）

**画面**：打开 Python 终端 / Jupyter

```python
>>> import folio
>>> folio.__version__
'0.2.1'
>>> data = folio.convert("# 你好\n\nThe formula is $E = mc^2$.")
>>> len(data), data[:2]
(8421, b'PK')
>>> open("hello.docx", "wb").write(data)
```

切去打开 `hello.docx`。

**旁白**：

> `folio.convert` 拿到的是 .docx 字节，可以直接写文件、上传到 S3、塞进邮件附件。中英混排没问题，公式照样可编辑。

### 3.3 文件到文件 + 主题（30 秒）

**画面**：

```python
>>> folio.convert_file(
...     "paper.md",
...     "paper.docx",
...     theme="academic"
... )

>>> folio.list_themes()
['academic', 'thesis-cn', 'report']
```

**旁白**：

> 文件到文件用 `convert_file`，相对路径的图片会按 Markdown 文件所在目录解析。`theme=` 选内置主题，跟命令行 `--theme` 一致。

### 3.4 自带模板（15 秒）

**画面**：

```python
>>> folio.convert_file(
...     "paper.md",
...     "paper.docx",
...     reference_doc="thesis-template.docx"
... )
```

**旁白**：

> `reference_doc=` 同样支持。Python 接口和命令行接口完全对齐——一个学的另一个就会。

### 3.5 多线程并行（15 秒，可选 advanced）

**画面**：

```python
>>> from concurrent.futures import ThreadPoolExecutor
>>> with ThreadPoolExecutor() as pool:
...     pool.map(lambda md: folio.convert(md), big_list_of_markdowns)
```

**旁白**：

> 顺便提一句，`convert` 主动释放了 GIL，多线程并行批量转换不会被锁住。

---

## 4. 桌面应用（约 110 秒）

### 4.1 下载安装（20 秒）

**画面**：浏览器打开 `https://github.com/Livia-Tassel/Folio/releases/tag/v0.2.1`，圈出两个安装包：
- `Folio_0.2.1_aarch64.dmg`（macOS）
- `Folio_0.2.1_x64-setup.exe`（Windows）

下载 `.dmg`（如果你录制 macOS 演示）→ 拖进 Applications。

**旁白**：

> 不想碰命令行的人，到 Releases 页下安装包。macOS 选 `aarch64.dmg` 拖进 Applications，Windows 选 `x64-setup.exe` 双击安装。

### 4.2 第一次打开（15 秒）

**画面**：从 Launchpad / 开始菜单打开 Folio。出现两栏界面：左 Markdown 编辑器、右实时 HTML 预览，自带一段 sample markdown。

**旁白**：

> 打开就是这个样子。左边写 Markdown，右边实时预览。

### 4.3 实时预览（15 秒）

**画面**：在左侧添加一段公式 `$$\int_0^\infty e^{-x^2} dx = \frac{\sqrt\pi}{2}$$`，右侧 250ms 后出现渲染。

**旁白**：

> 边输入边渲染。这是导出前的最后一道检查。

### 4.4 主题选择（25 秒）

**画面**：右上角 **Theme** 下拉打开，依次选 Default、academic、thesis-cn、report。每选一个，下面 **Export .docx** 按钮的样式不变（说明 dropdown 只影响导出）。

**旁白**：

> 右上角的 Theme 下拉就是命令行的 `--theme` 和 Python 的 `theme=` 一回事——三种入口完全对齐。选好之后点 Export .docx。

### 4.5 导出 + 在 Word 里看（30 秒）

**画面**：选 thesis-cn → 点 Export .docx → 浏览器下载条出来 → 双击 `folio.docx` 打开 Word → **镜头特写**：宋体、首行缩进、公式可编辑。

**旁白**：

> 文件下来直接进 Word。中文论文格式、公式可点击编辑、整个文档不用任何手动调整。

### 4.6 切换 reference-doc 一笔带过（5 秒）

**画面**：Theme 下拉显示当前没有"Custom .docx"项

**旁白**：

> 桌面端目前还没暴露自定义 .docx 模板的入口，下个版本补。要用自定义模板，先用命令行或 Python 接口。

---

## 5. 收尾（约 25 秒）

**画面**：分屏展示 GitHub 仓库 / PyPI 页面 / Folio 桌面图标

```
github.com/Livia-Tassel/Folio
pip install folio-docx
```

**旁白**：

> Folio 是 MIT 许可、开源项目。源码、桌面安装包、Python 包都在 GitHub 仓库里有链接，PyPI 上 `pip install folio-docx`。
>
> 如果你也在写 Markdown 出 Word 的工作流，欢迎试一下，issue 和 PR 都收。

**字幕**：

```
Folio v0.2.1
  github.com/Livia-Tassel/Folio
  pip install folio-docx
  License: MIT
```

---

## 演示用文件（已准备好，无需手动创建）

| 文件 | 说明 |
|---|---|
| `demo/demo-en.md` | 英文演示文档，涵盖所有功能（公式、表格、代码、图片、脚注、列表等） |
| `demo/demo-cn.md` | 中文演示文档，同等覆盖度，中英混排 |
| `demo/templates/acm.docx` | ACM 学术会议模板 |
| `demo/templates/ieee.docx` | IEEE 期刊模板 |
| `demo/templates/springer-lncs.docx` | Springer LNCS 模板 |
| `demo/scripts/cli-demo.sh` | CLI 演示 Shell 脚本（逐步暂停版） |
| `demo/scripts/python-demo.py` | Python 演示脚本（10 个 cell，从安装到批量并发） |
| `demo/scripts/python-demo.ipynb` | 同上的 Jupyter 版本 |

---

## 录制小贴士

1. **CLI 段用预编译二进制**，不要 `cargo run`——编译几十秒会打断节奏。先跑 `cargo build --release -p scribe-cli`。
2. **对比镜头**最有说服力的是"双击公式 → Word 弹出公式编辑器"——务必拍到这一秒。
3. **图片镜头**：展示 `demo-en.md` 里的 SVG banner 和 PNG icon 正确嵌入 Word，和公式一起作为"功能点"亮点。
4. **Python 段建议用 Jupyter Notebook**——单元格分隔比终端 prompt 美观，逐格演示更清晰。
5. **顶会模板段**：切换 acm → ieee → springer-lncs，三次 `--reference-doc`，字体和边距明显变化，视觉冲击强。
6. **桌面段录屏分辨率**用 1920×1080，缩放 Folio 窗口到大约 1500×900，留白看着透气。
7. **背景音乐**轻一点（lo-fi 之类）；旁白用 `ffmpeg -af "loudnorm=I=-16:LRA=11:TP=-1.5"` 标准化响度。
8. 总时长建议剪到 **7 分 30 秒上下**。每段如果意犹未尽，可以单独剪 2-3 分钟的"深入"短片。

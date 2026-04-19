<p align="center">
  <img src="docs/images/folio-logo.png" alt="Folio logo" width="120" />
</p>

<h1 align="center">Folio</h1>

<p align="center">
  面向高质量 <code>.docx</code> 输出的 Markdown 转 Word 工具，尽量不再需要后期手工修格式。
</p>

<p align="center">
  <a href="README.md"><strong>English</strong></a>
  ·
  <a href="README.zh-CN.md">简体中文</a>
</p>

> [!NOTE]
> 英文版 README 是主版本；本文件为中文说明与对照。

Folio 是一个跨平台桌面应用和 Rust 工作区，用于把 Markdown 转成结构正确、可继续编辑的 Microsoft Word 文档，而不是“能打开但还得再修一遍”的导出结果。它重点解决公式、图片、表格、层级样式等常见问题，让导出的文档在 Word 或 LibreOffice 中看起来更像最终成品。

## 为什么做 Folio

很多 Markdown 转 DOCX 的流程最后都会卡在“最后 10%”：

- 数学公式被拍平成图片，或者生成错误的 XML
- 列表和表格还要手工修
- 图片尺寸不可控，容易溢出或缩放异常
- 报告、论文、正式文档的版式仍然要回到 Word 里慢慢调

Folio 的目标就是把这些问题尽量前移到转换流程里解决。

## 示例导出

下面这个示例来自综合回归样例 [`test/folio-comprehensive.md`](test/folio-comprehensive.md)，导出文件为 [`test/output/folio-comprehensive.docx`](test/output/folio-comprehensive.docx)，截图来自对应 PDF 的渲染结果。

![Folio 示例导出](docs/images/demo-sample-export.png)

<p align="center">
  <a href="test/output/folio-comprehensive.docx">下载 DOCX</a>
  ·
  <a href="test/output/folio-comprehensive.pdf">查看 PDF</a>
  ·
  <a href="test/folio-comprehensive.md">查看 Markdown 源文件</a>
</p>

当前示例覆盖了：

- 标题层级
- 行内强调、代码和链接
- 无序列表、有序列表、任务列表
- 引用块与代码块
- 对齐表格
- 行内与块级 LaTeX 公式（导出为可编辑 OMML）
- 位图与 SVG 图片嵌入
- 脚注

## 当前能力

Folio 目前仍处于 **pre-alpha**，但核心转换链路已经不是原型脚本，而是一个相对完整的多 crate Rust 工程。

已实现：

- CommonMark / GFM Markdown 解析
- 供后续转换使用的 typed AST
- LaTeX -> MathML -> OMML 转换
- 图片加载、归一化和 SVG 光栅化
- 带样式、编号、脚注、表格、图片的 DOCX 输出
- 桌面端 HTML 预览
- 基于 Tauri + Svelte 的桌面壳层

尚未完整实现：

- 从用户 `reference.docx` 中读取模板样式
- 更丰富的论文/文档预设
- 图表公式交叉引用与自动编号
- 批量转换相关 UX 打磨
- 更高一致性的预览效果

## 技术栈

### 核心转换引擎

- Rust stable workspace
- `pulldown-cmark`：Markdown 解析
- `latex2mathml` + 自定义 `MathML -> OMML` 转换器
- `docx-rs`：OpenXML / DOCX 生成
- `image` + `resvg`：位图与 SVG 资源处理
- `syntect`：代码高亮
- `zip` + `quick-xml`：DOCX 包后处理

### 桌面应用

- Tauri 2：原生桌面壳
- Svelte 5 + SvelteKit：前端界面
- Vite：前端构建工具
- Tailwind CSS 4：样式层
- TypeScript：前端代码

## 仓库结构

产品名称已经切换为 **Folio**。为了避免一次性的大规模包重命名，当前内部 crate 仍然保留历史 `scribe-*` 前缀。

```text
crates/
  scribe-ast        Markdown typed AST
  scribe-parser     Markdown -> AST
  scribe-math       LaTeX -> MathML -> OMML
  scribe-images     图片加载与尺寸处理
  scribe-highlight  代码高亮
  scribe-template   模板与样式管线
  scribe-docx       AST -> .docx 输出
  scribe-preview    AST -> HTML 预览
  scribe-core       公共编排层
  scribe-tauri      桌面应用壳层
scribe-cli/         CLI 转换入口
fixtures/           小型功能样例
test/               综合回归样例与导出产物
docs/               设计文档与 README 资源
```

## 开发

### 环境要求

- Rust stable
- Node.js 20+
- `pnpm`

### 安装前端依赖

```bash
pnpm --dir crates/scribe-tauri/frontend install
```

### 运行测试

```bash
cargo test --workspace
pnpm --dir crates/scribe-tauri/frontend check
```

### 启动桌面应用

```bash
cd crates/scribe-tauri
cargo tauri dev
```

## 样例与回归测试

仓库中既有 [`fixtures/`](fixtures/) 下的聚焦样例，也有 [`test/`](test/) 下的一体化综合回归样例。

常用命令：

```bash
cargo run -p scribe-cli -- fixtures/english/m2-kitchen-sink.md -o /tmp/folio-m2.docx
cargo run -p scribe-cli -- fixtures/english/m3-math.md -o /tmp/folio-m3.docx
cargo run -p scribe-cli -- test/folio-comprehensive.md -o test/output/folio-comprehensive.docx
soffice --headless --convert-to pdf --outdir test/output test/output/folio-comprehensive.docx
```

## 设计文档

- [`docs/superpowers/specs/2026-04-17-scribe-md-to-docx-design.md`](docs/superpowers/specs/2026-04-17-scribe-md-to-docx-design.md)
- [`docs/superpowers/plans/2026-04-17-scribe-v1-plan.md`](docs/superpowers/plans/2026-04-17-scribe-v1-plan.md)

## Stars 历史

[![Star History Chart](https://api.star-history.com/svg?repos=Livia-Tassel/Folio&type=Date)](https://www.star-history.com/#Livia-Tassel/Folio&Date)

## License

MIT，详见 [`LICENSE`](LICENSE)。

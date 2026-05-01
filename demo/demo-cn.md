# Folio 演示文档

> Folio 把 Markdown 直接转换成 **Word 原生结构**——可编辑公式、规整表格、对齐字体，无需手动调整。

## 标题层级

下面几段展示二级、三级、四级标题的视觉差异。

### 三级标题：研究背景

学术写作和技术文档共同的痛点是 **格式整理**：内容写完后，光是把样式从 Markdown 统一到 Word 就要花掉半天。Folio 的设计目标就是把这一步消除。

#### 四级标题：实现说明

Folio 使用纯 Rust 管道——无需 Pandoc 依赖，命令行和桌面应用也不需要 Python 运行时。公式以原生 OMML 格式输出，而非图片。

## 行内格式

一段带有强制换行的文字：\
第二句在同一段落内另起一行。

支持 **加粗**、*斜体*、~~删除线~~、`行内代码`、[超链接](https://github.com/Livia-Tassel/Folio)，以及行内公式 $E = mc^2$。

中英混排：研究表明，**Markdown to Word** 在 GitHub 上每月有超过 10,000 次相关搜索，但现有工具在 *公式渲染* 上几乎全部失败。

## 公式

行内公式：当 $a \ne 0$，关于 $x$ 的一元二次方程 $ax^2 + bx + c = 0$ 有以下解。

显示公式：

$$x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}$$

爱因斯坦质能方程：

$$E = mc^2$$

求和与积分：

$$\sum_{i=1}^{n} i = \frac{n(n+1)}{2} \qquad \int_{0}^{1} x^2 \, dx = \frac{1}{3}$$

矩阵：

$$\begin{pmatrix} a & b \\ c & d \end{pmatrix}$$

## 代码块

带语法高亮的 Rust 代码：

```rust
fn main() {
    let s = "你好";
    println!("{}, Folio!", s);
}
```

Python 示例：

```python
import folio

bytes_ = folio.convert("# Hello\n\n$E = mc^2$", theme="academic")
with open("paper.docx", "wb") as f:
    f.write(bytes_)
```

JSON 配置：

```json
{
  "工具": "Folio",
  "版本": "0.2.1",
  "功能": ["公式", "表格", "图片", "脚注"]
}
```

## 表格

对齐表格（左对齐 / 居中 / 右对齐）：

| 主题 | 字体 | 适用场景 |
| :---- | :---: | -----: |
| `academic` | Times New Roman 12pt | 英文学术论文 |
| `thesis-cn` | 宋体 12pt + 黑体标题 | 中文学位论文 |
| `report` | Calibri + 蓝色点缀 | 商务报告、内部备忘 |

## 列表与任务

带嵌套的无序列表：

- 命令行 (`scribe-cli`)
  - 单文件转换
  - Shell 循环批量转换
- Python 包 (`folio-docx`)
  - `folio.convert()` — 字符串转字节
  - `folio.convert_file()` — 文件转文件
- 桌面应用 (Tauri)

有序列表：

1. 写 Markdown
2. 选主题或自带模板
3. 导出 Word，**不再调整样式**

任务列表：

- [x] Markdown → Word 原生结构
- [x] 内置主题
- [x] reference-doc 继承样式 + 页面设置
- [ ] 自定义主题脚手架（计划中）

## 图片

应用图标（PNG 光栅图）：

![Folio 应用图标](assets/folio-icon.png "Folio 应用图标")

宽横幅（SVG，自动缩放至页宽）：

![Folio 横幅](assets/folio-banner.svg "Folio 横幅")

布局示意图（SVG）：

![Folio 布局图](assets/layout-diagram.svg "Folio 布局图")

## 引用与脚注

> "Markdown to polished `.docx` output, without the cleanup pass."
>
> —— Folio 项目主页

行内文字带脚注[^1]，可以在文档底部展开。

[^1]: 脚注会被转换成 Word 原生 footnote，不是页内链接，复制到其他 Word 文档中依然有效。

## 分隔线

下面是分隔线（`---`）：

---

至此演示结束。运行下面的命令体验：

```bash
scribe-cli demo-cn.md -o demo-cn.docx --theme thesis-cn
```

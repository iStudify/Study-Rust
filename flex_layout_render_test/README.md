# Flex Layout Render

一个灵活的布局渲染引擎，支持 YAML DSL 和模板变量系统。

## 特性

- 🎨 **灵活的布局系统** - 基于 Flexbox 的强大布局引擎
- 📝 **YAML DSL** - 简洁直观的模板语法
- 🔧 **模板变量** - 支持 `{{variable}}` 语法的动态内容
- 🖼️ **多种内容类型** - 支持文本、图片和容器
- 🎯 **命令行工具** - 完整的 CLI 支持
- ⚡ **高性能** - 基于 Rust 的高效渲染

## 安装

```bash
cargo install --path .
```

## 快速开始

### 1. 创建模板文件

创建一个 YAML 模板文件 `template.yaml`：

```yaml
template:
  width: 800
  height: 600
  background: "#f0f0f0"

container:
  display: flex
  flex_direction: column
  justify_content: center
  align_items: center
  padding: 20
  children:
    - type: text
      content: "{{title}}"
      font_size: 48
      color: "#333333"
      font_weight: bold
    
    - type: text
      content: "{{subtitle}}"
      font_size: 24
      color: "#666666"
```

### 2. 创建变量文件

创建一个 JSON 变量文件 `variables.json`：

```json
{
  "title": "Hello World",
  "subtitle": "Welcome to Flex Layout Render"
}
```

### 3. 渲染图像

```bash
flex-render --input template.yaml --output output.png --var-file variables.json
```

## 命令行工具

### 基本用法

```bash
flex-render --input <template.yaml> --output <output.png> [OPTIONS]
```

### 选项

- `-i, --input <FILE>` - 输入 YAML 模板文件
- `-o, --output <FILE>` - 输出图像文件
- `-v, --variables <JSON>` - 通过 JSON 字符串传递变量
- `--var-file <FILE>` - 从 JSON 文件加载变量
- `--validate` - 仅验证模板，不进行渲染
- `--list-vars` - 列出模板中的所有变量
- `-h, --help` - 显示帮助信息
- `-V, --version` - 显示版本信息

### 示例

#### 列出模板变量

```bash
flex-render --input template.yaml --output output.png --list-vars
```

#### 验证模板

```bash
flex-render --input template.yaml --output output.png --validate
```

#### 使用命令行变量

```bash
flex-render --input template.yaml --output output.png --variables '{"title":"Hello","subtitle":"World"}'
```

#### 使用变量文件

```bash
flex-render --input template.yaml --output output.png --var-file variables.json
```

## YAML DSL 语法

### 模板配置

```yaml
template:
  width: 800          # 画布宽度
  height: 600         # 画布高度
  background: "#fff"  # 背景颜色
```

### 容器属性

```yaml
container:
  display: flex                    # 布局类型
  flex_direction: column           # 主轴方向: row, column
  justify_content: center          # 主轴对齐: flex-start, center, flex-end, space-between, space-around
  align_items: center              # 交叉轴对齐: flex-start, center, flex-end, stretch
  padding: 20                      # 内边距
  margin: 10                       # 外边距
  background: "#f0f0f0"           # 背景颜色
  border_radius: 10                # 圆角半径
  children: []                     # 子元素列表
```

### 文本元素

```yaml
- type: text
  content: "{{variable}}"          # 文本内容（支持变量）
  font_size: 24                    # 字体大小
  color: "#333333"                # 文字颜色
  font_weight: bold                # 字体粗细: normal, bold
  text_align: center               # 文本对齐: left, center, right
  margin: 10                       # 外边距
  padding: 5                       # 内边距
```

### 图片元素

```yaml
- type: image
  src: "path/to/image.png"        # 图片路径
  width: 200                       # 图片宽度
  height: 150                      # 图片高度
  object_fit: cover                # 适应方式: cover, contain, fill
```

## 模板变量

### 变量语法

使用 `{{variable_name}}` 语法在模板中引用变量：

```yaml
- type: text
  content: "Hello {{name}}!"
  color: "{{theme_color}}"
```

### 变量类型

支持以下 JSON 数据类型：

- **字符串**: `"Hello World"`
- **数字**: `42`, `3.14`
- **布尔值**: `true`, `false`
- **数组**: `["item1", "item2"]`
- **对象**: `{"key": "value"}`

## API 使用

### Rust API

```rust
use flex_layout_render::{FlexRenderer, TemplateVariables};
use std::collections::HashMap;

// 从 YAML 创建渲染器
let mut renderer = FlexRenderer::from_yaml(yaml_content)?;

// 设置变量
let mut variables = HashMap::new();
variables.insert("title".to_string(), serde_json::Value::String("Hello".to_string()));
renderer.set_variables(variables);

// 渲染图像
let image = renderer.render()?;
image.save("output.png")?;
```

## 示例

查看 `examples/` 目录中的示例文件：

- `examples/hello_world.yaml` - 基本模板示例
- `examples/variables.json` - 变量文件示例
- `examples/output.png` - 渲染结果

## 开发

### 构建

```bash
cargo build --release
```

### 测试

```bash
cargo test
```

### 运行示例

```bash
cargo run --release -- --input examples/hello_world.yaml --output examples/output.png --var-file examples/variables.json
```

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！
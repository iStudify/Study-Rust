# Rust 图文排版软件设计方案

## 1. 项目概述

本项目旨在实现一个基于 Rust 的图文排版软件，支持将图片和文本组合渲染成图片。核心特性包括：

- Flexbox 布局系统
- DSL 模板支持
- 高性能图像渲染
- 模板变量系统

## 2. 整体架构

### 2.1 核心模块

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   DSL Parser    │───▶│  Layout Engine  │───▶│    Renderer     │
│   (解析器)      │    │   (布局引擎)    │    │   (渲染引擎)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│Template Engine  │    │Resource Manager │    │  Image Output   │
│  (模板引擎)     │    │  (资源管理器)   │    │   (图像输出)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### 2.2 数据流

1. **输入阶段**: DSL 文件 + 变量数据
2. **解析阶段**: DSL → 布局树
3. **计算阶段**: Flex 布局计算
4. **渲染阶段**: 绘制到画布
5. **输出阶段**: 生成图片文件

## 3. 技术栈

### 3.1 核心依赖

```toml
[dependencies]
# 布局引擎
taffy = "0.3"

# 图像处理
image = "0.24"
ab_glyph = "0.2"
fontdue = "0.7"

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"

# 解析器
pest = "2.7"
pest_derive = "2.7"

# 错误处理
thiserror = "1.0"
anyhow = "1.0"

# 异步支持
tokio = { version = "1.0", features = ["full"] }

# 颜色处理
csscolorparser = "0.6"
```

## 4. DSL 设计

### 4.1 语法结构 (YAML)

```yaml
# 模板配置
template:
  name: "产品海报"
  width: 800
  height: 600
  background: "#ffffff"
  dpi: 300

# 根容器
container:
  display: flex
  flex_direction: column
  justify_content: center
  align_items: center
  padding: [20, 20, 20, 20] # top, right, bottom, left

  children:
    # 图片节点
    - type: image
      src: "{{product_image}}"
      width: 300
      height: 200
      object_fit: cover
      border_radius: 10

    # 文本节点
    - type: text
      content: "{{product_title}}"
      font_family: "PingFang SC"
      font_size: 24
      font_weight: bold
      color: "#333333"
      text_align: center
      margin: [20, 0, 10, 0]

    # 嵌套容器
    - type: container
      display: flex
      flex_direction: row
      justify_content: space_between
      align_items: center
      width: 100%
      padding: [10, 20]

      children:
        - type: text
          content: "价格: ¥{{price}}"
          font_size: 18
          color: "#e74c3c"
          font_weight: bold

        - type: text
          content: "{{discount}}折"
          font_size: 16
          color: "#27ae60"
          background: "#e8f5e8"
          padding: [5, 10]
          border_radius: 15
```

### 4.2 支持的节点类型

- **container**: 容器节点，支持 Flex 布局
- **text**: 文本节点
- **image**: 图片节点
- **shape**: 形状节点 (矩形、圆形等)

### 4.3 支持的样式属性

#### Flex 布局属性

- `display`: flex | block
- `flex_direction`: row | column | row-reverse | column-reverse
- `justify_content`: flex-start | flex-end | center | space-between | space-around
- `align_items`: flex-start | flex-end | center | stretch
- `flex_wrap`: nowrap | wrap | wrap-reverse
- `flex_grow`: number
- `flex_shrink`: number
- `flex_basis`: auto | length

#### 尺寸属性

- `width`: auto | length | percentage
- `height`: auto | length | percentage
- `min_width`: length
- `max_width`: length
- `min_height`: length
- `max_height`: length

#### 间距属性

- `margin`: [top, right, bottom, left]
- `padding`: [top, right, bottom, left]

#### 外观属性

- `background`: color | gradient | image
- `border`: width | style | color
- `border_radius`: length
- `opacity`: 0.0 - 1.0

## 5. 核心数据结构

### 5.1 布局节点

```rust
use taffy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutNode {
    Container {
        style: ContainerStyle,
        children: Vec<LayoutNode>,
    },
    Text {
        content: String,
        style: TextStyle,
    },
    Image {
        src: String,
        style: ImageStyle,
    },
    Shape {
        shape_type: ShapeType,
        style: ShapeStyle,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStyle {
    // Flex 属性
    pub display: Display,
    pub flex_direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub flex_wrap: FlexWrap,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Dimension,

    // 尺寸属性
    pub width: Dimension,
    pub height: Dimension,
    pub min_width: Dimension,
    pub max_width: Dimension,
    pub min_height: Dimension,
    pub max_height: Dimension,

    // 间距属性
    pub margin: Rect<LengthPercentageAuto>,
    pub padding: Rect<LengthPercentage>,

    // 外观属性
    pub background: Option<Background>,
    pub border: Option<Border>,
    pub border_radius: f32,
    pub opacity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub color: Color,
    pub text_align: TextAlign,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub text_decoration: TextDecoration,

    // 继承容器样式
    pub margin: Rect<LengthPercentageAuto>,
    pub padding: Rect<LengthPercentage>,
    pub background: Option<Background>,
    pub border: Option<Border>,
    pub border_radius: f32,
    pub opacity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageStyle {
    pub object_fit: ObjectFit,
    pub object_position: ObjectPosition,

    // 继承容器样式
    pub width: Dimension,
    pub height: Dimension,
    pub margin: Rect<LengthPercentageAuto>,
    pub padding: Rect<LengthPercentage>,
    pub border: Option<Border>,
    pub border_radius: f32,
    pub opacity: f32,
}
```

### 5.2 辅助类型

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Background {
    Color(Color),
    Gradient(Gradient),
    Image(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Border {
    pub width: f32,
    pub style: BorderStyle,
    pub color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FontWeight {
    Normal,
    Bold,
    Weight(u16),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectFit {
    Fill,
    Contain,
    Cover,
    ScaleDown,
    None,
}
```

## 6. 模块实现计划

### 6.1 第一阶段：基础框架

1. **项目结构搭建**

   - 创建模块目录结构
   - 配置 Cargo.toml 依赖
   - 设置基础错误处理

2. **数据结构定义**
   - 定义核心数据结构
   - 实现序列化/反序列化
   - 添加默认值和验证

### 6.2 第二阶段：DSL 解析器

1. **YAML 解析**

   - 实现 YAML 到数据结构的转换
   - 添加语法验证
   - 错误处理和提示

2. **模板变量系统**
   - 实现变量替换
   - 支持条件渲染
   - 支持循环渲染

### 6.3 第三阶段：布局引擎

1. **Taffy 集成**

   - 将自定义样式转换为 Taffy 样式
   - 实现布局计算
   - 处理约束和尺寸

2. **布局树构建**
   - 从 DSL 构建布局树
   - 计算最终位置和尺寸
   - 优化布局性能

### 6.4 第四阶段：渲染引擎

1. **基础渲染**

   - 实现画布创建
   - 文本渲染
   - 图片渲染

2. **高级渲染**
   - 边框和圆角
   - 阴影效果
   - 渐变背景

### 6.5 第五阶段：资源管理

1. **字体管理**

   - 字体加载和缓存
   - 字体回退机制
   - 字体度量计算

2. **图片管理**
   - 图片加载和缓存
   - 格式转换
   - 尺寸优化

### 6.6 第六阶段：API 和工具

1. **核心 API**

   - 简化的使用接口
   - 批量处理支持
   - 异步渲染

2. **命令行工具**
   - CLI 接口
   - 配置文件支持
   - 批量处理

## 7. 性能优化策略

### 7.1 缓存机制

- **字体缓存**: 避免重复加载字体文件
- **图片缓存**: 缓存解码后的图片数据
- **布局缓存**: 缓存布局计算结果
- **渲染缓存**: 缓存中间渲染结果

### 7.2 内存管理

- 使用 `Arc<T>` 共享不可变数据
- 延迟加载大型资源
- 及时释放不需要的资源
- 内存池复用画布

### 7.3 并发处理

- 异步资源加载
- 并行布局计算
- 多线程渲染
- 批量处理优化

## 8. 错误处理

### 8.1 错误类型

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LayoutError {
    #[error("DSL 解析错误: {0}")]
    ParseError(String),

    #[error("布局计算错误: {0}")]
    LayoutError(String),

    #[error("渲染错误: {0}")]
    RenderError(String),

    #[error("资源加载错误: {0}")]
    ResourceError(String),

    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("图片处理错误: {0}")]
    ImageError(#[from] image::ImageError),
}
```

### 8.2 错误恢复

- 提供默认值
- 跳过错误节点
- 降级渲染
- 详细错误信息

## 9. 测试策略

### 9.1 单元测试

- 数据结构测试
- 解析器测试
- 布局算法测试
- 渲染功能测试

### 9.2 集成测试

- 端到端渲染测试
- 性能基准测试
- 内存泄漏测试
- 并发安全测试

### 9.3 视觉回归测试

- 渲染结果对比
- 像素级精度验证
- 跨平台一致性

## 10. 扩展性设计

### 10.1 插件系统

- 自定义节点类型
- 自定义渲染器
- 自定义过滤器
- 主题系统

### 10.2 多格式支持

- PNG/JPEG/WebP 输出
- SVG 矢量输出
- PDF 文档输出
- 动画 GIF 支持

### 10.3 平台支持

- Web Assembly 支持
- 移动端适配
- 服务端渲染
- 桌面应用集成

## 11. 开发里程碑

### Milestone 1: 基础框架 (1-2 周)

- [ ] 项目结构搭建
- [ ] 核心数据结构
- [ ] 基础错误处理
- [ ] 单元测试框架

### Milestone 2: DSL 解析 (2-3 周)

- [ ] YAML 解析器
- [ ] 语法验证
- [ ] 模板变量系统
- [ ] 解析器测试

### Milestone 3: 布局引擎 (3-4 周)

- [ ] Taffy 集成
- [ ] 布局计算
- [ ] 约束处理
- [ ] 布局测试

### Milestone 4: 渲染引擎 (4-5 周)

- [ ] 基础渲染
- [ ] 文本渲染
- [ ] 图片渲染
- [ ] 高级效果

### Milestone 5: 优化和工具 (2-3 周)

- [ ] 性能优化
- [ ] CLI 工具
- [ ] 文档完善
- [ ] 发布准备

## 12. 总结

本设计方案提供了一个完整的 Rust 图文排版软件架构，具有以下特点：

- **模块化设计**: 清晰的模块划分，便于开发和维护
- **高性能**: 基于 Rust 的内存安全和性能优势
- **灵活的 DSL**: 支持复杂的布局和样式定义
- **可扩展性**: 插件系统和多格式支持
- **工程化**: 完整的测试、错误处理和文档

通过分阶段实现，可以逐步构建一个功能完整、性能优异的图文排版系统。

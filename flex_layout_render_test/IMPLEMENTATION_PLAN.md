# 实现计划

## 开发顺序和具体步骤

### 第一阶段：项目基础设施 (Week 1-2)

#### 1.1 项目结构搭建

```
src/
├── main.rs              # 程序入口
├── lib.rs               # 库入口
├── error.rs             # 错误定义
├── types.rs             # 基础类型定义
├── parser/              # DSL 解析模块
│   ├── mod.rs
│   ├── yaml_parser.rs   # YAML 解析
│   ├── template.rs      # 模板处理
│   └── validator.rs     # 语法验证
├── layout/              # 布局引擎模块
│   ├── mod.rs
│   ├── engine.rs        # 布局计算
│   ├── node.rs          # 布局节点
│   └── style.rs         # 样式转换
├── render/              # 渲染引擎模块
│   ├── mod.rs
│   ├── canvas.rs        # 画布管理
│   ├── text.rs          # 文本渲染
│   ├── image.rs         # 图片渲染
│   └── effects.rs       # 视觉效果
├── resource/            # 资源管理模块
│   ├── mod.rs
│   ├── font.rs          # 字体管理
│   ├── image.rs         # 图片管理
│   └── cache.rs         # 缓存系统
└── cli/                 # 命令行工具
    ├── mod.rs
    └── commands.rs
```

#### 1.2 更新 Cargo.toml

```toml
[package]
name = "flex_layout_render"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "A flexible layout rendering engine with DSL support"
license = "MIT"
repository = "https://github.com/yourusername/flex_layout_render"

[dependencies]
# 布局引擎
taffy = "0.3"

# 图像处理
image = { version = "0.24", features = ["png", "jpeg", "webp"] }
ab_glyph = "0.2"
fontdue = "0.7"

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"

# 解析和模板
handlebars = "4.4"
regex = "1.10"

# 错误处理
thiserror = "1.0"
anyhow = "1.0"

# 异步支持
tokio = { version = "1.0", features = ["full"], optional = true }

# 颜色处理
csscolorparser = "0.6"

# 命令行
clap = { version = "4.4", features = ["derive"] }

# 日志
log = "0.4"
env_logger = "0.10"

# 开发依赖
[dev-dependencies]
pretty_assertions = "1.4"
tempfile = "3.8"
criterion = "0.5"

# 特性
[features]
default = ["async"]
async = ["tokio"]

# 基准测试
[[bench]]
name = "layout_benchmark"
harness = false

# 二进制文件
[[bin]]
name = "flex-render"
path = "src/main.rs"
```

#### 1.3 基础类型和错误定义

**src/error.rs**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlexRenderError {
    #[error("DSL 解析错误: {message} (行 {line}, 列 {column})")]
    ParseError {
        message: String,
        line: usize,
        column: usize,
    },
    
    #[error("模板变量错误: 未找到变量 '{name}'")]
    TemplateVariableError { name: String },
    
    #[error("布局计算错误: {0}")]
    LayoutError(String),
    
    #[error("渲染错误: {0}")]
    RenderError(String),
    
    #[error("字体加载错误: {path}")]
    FontLoadError { path: String },
    
    #[error("图片加载错误: {path} - {reason}")]
    ImageLoadError { path: String, reason: String },
    
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("图片处理错误: {0}")]
    ImageError(#[from] image::ImageError),
    
    #[error("YAML 解析错误: {0}")]
    YamlError(#[from] serde_yaml::Error),
    
    #[error("JSON 解析错误: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, FlexRenderError>;
```

**src/types.rs**
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// 基础类型定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    
    pub fn from_hex(hex: &str) -> crate::Result<Self> {
        let color = csscolorparser::parse(hex)
            .map_err(|e| crate::error::FlexRenderError::ParseError {
                message: format!("无效的颜色值: {}", e),
                line: 0,
                column: 0,
            })?;
        
        Ok(Self {
            r: (color.r * 255.0) as u8,
            g: (color.g * 255.0) as u8,
            b: (color.b * 255.0) as u8,
            a: (color.a * 255.0) as u8,
        })
    }
    
    pub fn transparent() -> Self {
        Self::new(0, 0, 0, 0)
    }
    
    pub fn white() -> Self {
        Self::new(255, 255, 255, 255)
    }
    
    pub fn black() -> Self {
        Self::new(0, 0, 0, 255)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

// 模板变量类型
pub type TemplateVariables = HashMap<String, serde_json::Value>;

// 常用枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Unit {
    Px(f32),
    Percent(f32),
    Auto,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FontWeight {
    Normal,
    Bold,
    Weight(u16),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ObjectFit {
    Fill,
    Contain,
    Cover,
    ScaleDown,
    None,
}
```

### 第二阶段：DSL 解析器 (Week 2-4)

#### 2.1 布局节点定义

**src/layout/node.rs**
```rust
use crate::types::*;
use serde::{Deserialize, Serialize};
use taffy::prelude::*;

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
    pub background: Option<Color>,
    pub border_width: f32,
    pub border_color: Color,
    pub border_radius: f32,
    pub opacity: f32,
}

impl Default for ContainerStyle {
    fn default() -> Self {
        Self {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            flex_wrap: FlexWrap::NoWrap,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: Dimension::Auto,
            
            width: Dimension::Auto,
            height: Dimension::Auto,
            min_width: Dimension::Auto,
            max_width: Dimension::Auto,
            min_height: Dimension::Auto,
            max_height: Dimension::Auto,
            
            margin: Rect::zero(),
            padding: Rect::zero(),
            
            background: None,
            border_width: 0.0,
            border_color: Color::transparent(),
            border_radius: 0.0,
            opacity: 1.0,
        }
    }
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
    
    // 继承容器样式
    pub width: Dimension,
    pub height: Dimension,
    pub margin: Rect<LengthPercentageAuto>,
    pub padding: Rect<LengthPercentage>,
    pub background: Option<Color>,
    pub border_width: f32,
    pub border_color: Color,
    pub border_radius: f32,
    pub opacity: f32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: "Arial".to_string(),
            font_size: 16.0,
            font_weight: FontWeight::Normal,
            color: Color::black(),
            text_align: TextAlign::Left,
            line_height: 1.2,
            letter_spacing: 0.0,
            
            width: Dimension::Auto,
            height: Dimension::Auto,
            margin: Rect::zero(),
            padding: Rect::zero(),
            background: None,
            border_width: 0.0,
            border_color: Color::transparent(),
            border_radius: 0.0,
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageStyle {
    pub object_fit: ObjectFit,
    
    // 继承容器样式
    pub width: Dimension,
    pub height: Dimension,
    pub margin: Rect<LengthPercentageAuto>,
    pub padding: Rect<LengthPercentage>,
    pub border_width: f32,
    pub border_color: Color,
    pub border_radius: f32,
    pub opacity: f32,
}

impl Default for ImageStyle {
    fn default() -> Self {
        Self {
            object_fit: ObjectFit::Fill,
            
            width: Dimension::Auto,
            height: Dimension::Auto,
            margin: Rect::zero(),
            padding: Rect::zero(),
            border_width: 0.0,
            border_color: Color::transparent(),
            border_radius: 0.0,
            opacity: 1.0,
        }
    }
}
```

#### 2.2 YAML 解析器

**src/parser/yaml_parser.rs**
```rust
use crate::layout::node::*;
use crate::types::*;
use crate::error::*;
use serde_yaml::Value;
use std::collections::HashMap;

pub struct YamlParser;

impl YamlParser {
    pub fn parse(yaml_content: &str) -> Result<(TemplateConfig, LayoutNode)> {
        let value: Value = serde_yaml::from_str(yaml_content)?;
        
        let template_config = Self::parse_template_config(&value)?;
        let root_node = Self::parse_node(&value["container"])?;
        
        Ok((template_config, root_node))
    }
    
    fn parse_template_config(value: &Value) -> Result<TemplateConfig> {
        let template = &value["template"];
        
        Ok(TemplateConfig {
            name: template["name"].as_str().unwrap_or("Untitled").to_string(),
            width: template["width"].as_f64().unwrap_or(800.0) as f32,
            height: template["height"].as_f64().unwrap_or(600.0) as f32,
            background: Self::parse_color(&template["background"])?,
            dpi: template["dpi"].as_f64().unwrap_or(72.0) as f32,
        })
    }
    
    fn parse_node(value: &Value) -> Result<LayoutNode> {
        let node_type = value["type"].as_str().unwrap_or("container");
        
        match node_type {
            "container" => Self::parse_container(value),
            "text" => Self::parse_text(value),
            "image" => Self::parse_image(value),
            _ => Err(FlexRenderError::ParseError {
                message: format!("未知的节点类型: {}", node_type),
                line: 0,
                column: 0,
            }),
        }
    }
    
    fn parse_container(value: &Value) -> Result<LayoutNode> {
        let style = Self::parse_container_style(value)?;
        let mut children = Vec::new();
        
        if let Some(children_value) = value["children"].as_sequence() {
            for child_value in children_value {
                children.push(Self::parse_node(child_value)?);
            }
        }
        
        Ok(LayoutNode::Container { style, children })
    }
    
    fn parse_text(value: &Value) -> Result<LayoutNode> {
        let content = value["content"].as_str()
            .ok_or_else(|| FlexRenderError::ParseError {
                message: "文本节点缺少 content 属性".to_string(),
                line: 0,
                column: 0,
            })?
            .to_string();
            
        let style = Self::parse_text_style(value)?;
        
        Ok(LayoutNode::Text { content, style })
    }
    
    fn parse_image(value: &Value) -> Result<LayoutNode> {
        let src = value["src"].as_str()
            .ok_or_else(|| FlexRenderError::ParseError {
                message: "图片节点缺少 src 属性".to_string(),
                line: 0,
                column: 0,
            })?
            .to_string();
            
        let style = Self::parse_image_style(value)?;
        
        Ok(LayoutNode::Image { src, style })
    }
    
    // 样式解析方法...
    fn parse_container_style(value: &Value) -> Result<ContainerStyle> {
        let mut style = ContainerStyle::default();
        
        // 解析 flex 属性
        if let Some(display) = value["display"].as_str() {
            style.display = match display {
                "flex" => Display::Flex,
                "block" => Display::Block,
                _ => Display::Flex,
            };
        }
        
        // 更多样式解析...
        
        Ok(style)
    }
    
    fn parse_color(value: &Value) -> Result<Color> {
        if let Some(color_str) = value.as_str() {
            Color::from_hex(color_str)
        } else {
            Ok(Color::white())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub name: String,
    pub width: f32,
    pub height: f32,
    pub background: Color,
    pub dpi: f32,
}
```

### 第三阶段：布局引擎 (Week 4-6)

#### 3.1 布局引擎核心

**src/layout/engine.rs**
```rust
use crate::layout::node::*;
use crate::types::*;
use crate::error::*;
use taffy::prelude::*;
use std::collections::HashMap;

pub struct LayoutEngine {
    taffy: Taffy,
    node_map: HashMap<NodeId, LayoutNode>,
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            taffy: Taffy::new(),
            node_map: HashMap::new(),
        }
    }
    
    pub fn compute_layout(
        &mut self, 
        root: &LayoutNode, 
        available_space: Size
    ) -> Result<ComputedLayout> {
        // 构建 Taffy 节点树
        let root_id = self.build_taffy_tree(root)?;
        
        // 计算布局
        self.taffy.compute_layout(
            root_id,
            taffy::geometry::Size {
                width: AvailableSpace::Definite(available_space.width),
                height: AvailableSpace::Definite(available_space.height),
            },
        ).map_err(|e| FlexRenderError::LayoutError(format!("布局计算失败: {:?}", e)))?;
        
        // 提取布局结果
        self.extract_layout(root_id, root)
    }
    
    fn build_taffy_tree(&mut self, node: &LayoutNode) -> Result<NodeId> {
        match node {
            LayoutNode::Container { style, children } => {
                let mut child_ids = Vec::new();
                
                for child in children {
                    let child_id = self.build_taffy_tree(child)?;
                    child_ids.push(child_id);
                }
                
                let taffy_style = self.convert_container_style(style)?;
                let node_id = self.taffy.new_with_children(taffy_style, &child_ids)
                    .map_err(|e| FlexRenderError::LayoutError(format!("创建容器节点失败: {:?}", e)))?;
                
                self.node_map.insert(node_id, node.clone());
                Ok(node_id)
            },
            LayoutNode::Text { style, .. } => {
                let taffy_style = self.convert_text_style(style)?;
                let node_id = self.taffy.new_leaf(taffy_style)
                    .map_err(|e| FlexRenderError::LayoutError(format!("创建文本节点失败: {:?}", e)))?;
                
                self.node_map.insert(node_id, node.clone());
                Ok(node_id)
            },
            LayoutNode::Image { style, .. } => {
                let taffy_style = self.convert_image_style(style)?;
                let node_id = self.taffy.new_leaf(taffy_style)
                    .map_err(|e| FlexRenderError::LayoutError(format!("创建图片节点失败: {:?}", e)))?;
                
                self.node_map.insert(node_id, node.clone());
                Ok(node_id)
            },
        }
    }
    
    fn convert_container_style(&self, style: &ContainerStyle) -> Result<Style> {
        Ok(Style {
            display: style.display,
            flex_direction: style.flex_direction,
            justify_content: style.justify_content,
            align_items: style.align_items,
            flex_wrap: style.flex_wrap,
            flex_grow: style.flex_grow,
            flex_shrink: style.flex_shrink,
            flex_basis: style.flex_basis,
            
            size: taffy::geometry::Size {
                width: style.width,
                height: style.height,
            },
            min_size: taffy::geometry::Size {
                width: style.min_width,
                height: style.min_height,
            },
            max_size: taffy::geometry::Size {
                width: style.max_width,
                height: style.max_height,
            },
            
            margin: style.margin,
            padding: style.padding,
            
            ..Default::default()
        })
    }
    
    fn extract_layout(&self, node_id: NodeId, original_node: &LayoutNode) -> Result<ComputedLayout> {
        let layout = self.taffy.layout(node_id)
            .map_err(|e| FlexRenderError::LayoutError(format!("获取布局失败: {:?}", e)))?;
        
        let bounds = Rect {
            x: layout.location.x,
            y: layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        };
        
        let mut children = Vec::new();
        if let LayoutNode::Container { children: child_nodes, .. } = original_node {
            let child_ids = self.taffy.children(node_id)
                .map_err(|e| FlexRenderError::LayoutError(format!("获取子节点失败: {:?}", e)))?;
            
            for (child_id, child_node) in child_ids.iter().zip(child_nodes.iter()) {
                children.push(self.extract_layout(*child_id, child_node)?);
            }
        }
        
        Ok(ComputedLayout {
            node: original_node.clone(),
            bounds,
            children,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ComputedLayout {
    pub node: LayoutNode,
    pub bounds: Rect,
    pub children: Vec<ComputedLayout>,
}
```

### 第四阶段：渲染引擎 (Week 6-8)

#### 4.1 渲染引擎核心

**src/render/canvas.rs**
```rust
use crate::types::*;
use crate::error::*;
use crate::layout::engine::ComputedLayout;
use image::{ImageBuffer, Rgba, RgbaImage};

pub struct Canvas {
    buffer: RgbaImage,
    width: u32,
    height: u32,
}

impl Canvas {
    pub fn new(width: u32, height: u32, background: Color) -> Self {
        let mut buffer = ImageBuffer::new(width, height);
        
        // 填充背景色
        for pixel in buffer.pixels_mut() {
            *pixel = Rgba([background.r, background.g, background.b, background.a]);
        }
        
        Self {
            buffer,
            width,
            height,
        }
    }
    
    pub fn render_layout(&mut self, layout: &ComputedLayout) -> Result<()> {
        self.render_node(layout)?;
        Ok(())
    }
    
    fn render_node(&mut self, layout: &ComputedLayout) -> Result<()> {
        match &layout.node {
            LayoutNode::Container { style, .. } => {
                self.render_container_background(style, &layout.bounds)?;
                self.render_container_border(style, &layout.bounds)?;
                
                // 渲染子节点
                for child in &layout.children {
                    self.render_node(child)?;
                }
            },
            LayoutNode::Text { content, style } => {
                self.render_text(content, style, &layout.bounds)?;
            },
            LayoutNode::Image { src, style } => {
                self.render_image(src, style, &layout.bounds)?;
            },
        }
        
        Ok(())
    }
    
    fn render_container_background(&mut self, style: &ContainerStyle, bounds: &Rect) -> Result<()> {
        if let Some(background) = &style.background {
            self.fill_rect(bounds, *background);
        }
        Ok(())
    }
    
    fn render_container_border(&mut self, style: &ContainerStyle, bounds: &Rect) -> Result<()> {
        if style.border_width > 0.0 {
            self.stroke_rect(bounds, style.border_color, style.border_width);
        }
        Ok(())
    }
    
    fn fill_rect(&mut self, rect: &Rect, color: Color) {
        let x1 = rect.x.max(0.0) as u32;
        let y1 = rect.y.max(0.0) as u32;
        let x2 = (rect.x + rect.width).min(self.width as f32) as u32;
        let y2 = (rect.y + rect.height).min(self.height as f32) as u32;
        
        for y in y1..y2 {
            for x in x1..x2 {
                if x < self.width && y < self.height {
                    self.buffer.put_pixel(x, y, Rgba([color.r, color.g, color.b, color.a]));
                }
            }
        }
    }
    
    fn stroke_rect(&mut self, rect: &Rect, color: Color, width: f32) {
        // 简化的边框绘制实现
        let border_width = width as u32;
        
        // 上边框
        for i in 0..border_width {
            let border_rect = Rect {
                x: rect.x,
                y: rect.y + i as f32,
                width: rect.width,
                height: 1.0,
            };
            self.fill_rect(&border_rect, color);
        }
        
        // 下边框
        for i in 0..border_width {
            let border_rect = Rect {
                x: rect.x,
                y: rect.y + rect.height - 1.0 - i as f32,
                width: rect.width,
                height: 1.0,
            };
            self.fill_rect(&border_rect, color);
        }
        
        // 左边框
        for i in 0..border_width {
            let border_rect = Rect {
                x: rect.x + i as f32,
                y: rect.y,
                width: 1.0,
                height: rect.height,
            };
            self.fill_rect(&border_rect, color);
        }
        
        // 右边框
        for i in 0..border_width {
            let border_rect = Rect {
                x: rect.x + rect.width - 1.0 - i as f32,
                y: rect.y,
                width: 1.0,
                height: rect.height,
            };
            self.fill_rect(&border_rect, color);
        }
    }
    
    pub fn to_image(self) -> RgbaImage {
        self.buffer
    }
}
```

### 第五阶段：集成和优化 (Week 8-10)

#### 5.1 主要 API

**src/lib.rs**
```rust
pub mod error;
pub mod types;
pub mod parser;
pub mod layout;
pub mod render;
pub mod resource;

use crate::error::*;
use crate::types::*;
use crate::parser::yaml_parser::{YamlParser, TemplateConfig};
use crate::layout::engine::LayoutEngine;
use crate::render::canvas::Canvas;
use image::RgbaImage;
use std::path::Path;

pub struct FlexRenderer {
    template_config: TemplateConfig,
    root_node: layout::node::LayoutNode,
    variables: TemplateVariables,
}

impl FlexRenderer {
    pub fn from_yaml(yaml_content: &str) -> Result<Self> {
        let (template_config, root_node) = YamlParser::parse(yaml_content)?;
        
        Ok(Self {
            template_config,
            root_node,
            variables: TemplateVariables::new(),
        })
    }
    
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_yaml(&content)
    }
    
    pub fn set_variables(&mut self, variables: TemplateVariables) {
        self.variables = variables;
    }
    
    pub fn render(&self) -> Result<RgbaImage> {
        // 应用模板变量
        let processed_node = self.apply_template_variables(&self.root_node)?;
        
        // 计算布局
        let mut layout_engine = LayoutEngine::new();
        let available_space = Size {
            width: self.template_config.width,
            height: self.template_config.height,
        };
        let computed_layout = layout_engine.compute_layout(&processed_node, available_space)?;
        
        // 渲染到画布
        let mut canvas = Canvas::new(
            self.template_config.width as u32,
            self.template_config.height as u32,
            self.template_config.background,
        );
        canvas.render_layout(&computed_layout)?;
        
        Ok(canvas.to_image())
    }
    
    pub fn render_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let image = self.render()?;
        image.save(path)?;
        Ok(())
    }
    
    fn apply_template_variables(&self, node: &layout::node::LayoutNode) -> Result<layout::node::LayoutNode> {
        // 模板变量替换逻辑
        // 这里需要实现 Handlebars 模板处理
        Ok(node.clone()) // 临时实现
    }
}
```

#### 5.2 命令行工具

**src/main.rs**
```rust
use clap::{Arg, Command};
use flex_layout_render::FlexRenderer;
use std::collections::HashMap;
use std::path::Path;

fn main() {
    env_logger::init();
    
    let matches = Command::new("flex-render")
        .version("0.1.0")
        .about("Flexible layout rendering engine")
        .arg(
            Arg::new("template")
                .short('t')
                .long("template")
                .value_name("FILE")
                .help("Template YAML file")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output image file")
                .required(true),
        )
        .arg(
            Arg::new("variables")
                .short('v')
                .long("variables")
                .value_name("JSON")
                .help("Template variables as JSON string"),
        )
        .get_matches();
    
    let template_path = matches.get_one::<String>("template").unwrap();
    let output_path = matches.get_one::<String>("output").unwrap();
    
    let mut renderer = match FlexRenderer::from_file(template_path) {
        Ok(renderer) => renderer,
        Err(e) => {
            eprintln!("加载模板失败: {}", e);
            std::process::exit(1);
        }
    };
    
    if let Some(variables_json) = matches.get_one::<String>("variables") {
        let variables: HashMap<String, serde_json::Value> = match serde_json::from_str(variables_json) {
            Ok(vars) => vars,
            Err(e) => {
                eprintln!("解析变量失败: {}", e);
                std::process::exit(1);
            }
        };
        renderer.set_variables(variables);
    }
    
    if let Err(e) = renderer.render_to_file(output_path) {
        eprintln!("渲染失败: {}", e);
        std::process::exit(1);
    }
    
    println!("渲染完成: {}", output_path);
}
```

## 测试策略

### 单元测试示例

**tests/parser_tests.rs**
```rust
use flex_layout_render::parser::yaml_parser::YamlParser;

#[test]
fn test_parse_simple_template() {
    let yaml = r#"
template:
  width: 800
  height: 600
  background: "#ffffff"

container:
  display: flex
  flex_direction: column
  children:
    - type: text
      content: "Hello World"
      font_size: 24
"#;
    
    let result = YamlParser::parse(yaml);
    assert!(result.is_ok());
    
    let (config, node) = result.unwrap();
    assert_eq!(config.width, 800.0);
    assert_eq!(config.height, 600.0);
}
```

### 集成测试示例

**tests/integration_tests.rs**
```rust
use flex_layout_render::FlexRenderer;
use std::collections::HashMap;

#[test]
fn test_end_to_end_rendering() {
    let yaml = r#"
template:
  width: 400
  height: 300
  background: "#f0f0f0"

container:
  display: flex
  justify_content: center
  align_items: center
  children:
    - type: text
      content: "{{title}}"
      font_size: 20
      color: "#333333"
"#;
    
    let mut renderer = FlexRenderer::from_yaml(yaml).unwrap();
    
    let mut variables = HashMap::new();
    variables.insert("title".to_string(), serde_json::Value::String("Test Title".to_string()));
    renderer.set_variables(variables);
    
    let image = renderer.render().unwrap();
    assert_eq!(image.width(), 400);
    assert_eq!(image.height(), 300);
}
```

## 下一步行动

1. **立即开始**: 搭建项目基础结构
2. **第一周目标**: 完成错误处理和基础类型定义
3. **第二周目标**: 实现 YAML 解析器的核心功能
4. **持续集成**: 每个模块完成后立即添加测试
5. **文档更新**: 随着实现进度更新设计文档

这个实现计划提供了详细的代码结构和开发步骤，可以按照这个计划逐步实现一个完整的 Rust 图文排版系统。
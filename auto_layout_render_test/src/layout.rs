//! Auto Layout 核心数据结构定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 元素唯一标识符
pub type ElementId = String;

/// 颜色定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };
    
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 && hex.len() != 8 {
            return Err("Invalid hex color format".to_string());
        }
        
        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex color")?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex color")?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex color")?;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).map_err(|_| "Invalid hex color")?
        } else {
            255
        };
        
        Ok(Color { r, g, b, a })
    }
}

/// 尺寸定义
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

/// 位置定义
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// 矩形定义
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

/// 内边距定义
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Padding {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl Padding {
    pub fn all(value: f32) -> Self {
        Self { top: value, bottom: value, left: value, right: value }
    }
    
    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self { top: vertical, bottom: vertical, left: horizontal, right: horizontal }
    }
}

/// 文本对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[derive(Default)]
pub enum TextAlignment {
    #[default]
    Leading,
    Center,
    Trailing,
    Justified,
}


/// 字体粗细
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[derive(Default)]
pub enum FontWeight {
    Light,
    #[default]
    Normal,
    Bold,
}


/// 换行模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LineBreakMode {
    #[default]
    WordWrap,
    CharWrap,
    Clip,
    TruncateHead,
    TruncateTail,
    TruncateMiddle,
}

/// 图片缩放模式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[derive(Default)]
pub enum ScaleMode {
    #[default]
    Fit,
    Fill,
    Stretch,
    Center,
}


/// 对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[derive(Default)]
pub enum Alignment {
    Leading,
    #[default]
    Center,
    Trailing,
    Top,
    Bottom,
    FirstBaseline,
    LastBaseline,
}


/// 分布方式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[derive(Default)]
pub enum Distribution {
    #[default]
    Fill,
    FillEqually,
    FillProportionally,
    EqualSpacing,
    EqualCentering,
}


/// 约束优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u32)]
pub enum Priority {
    Required = 1000,
    High = 750,
    #[default]
    Medium = 500,
    Low = 250,
}

impl Priority {
    pub fn value(&self) -> u32 {
        match self {
            Priority::Required => 1000,
            Priority::High => 750,
            Priority::Medium => 500,
            Priority::Low => 250,
        }
    }
}

/// 尺寸约束类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SizeConstraint {
    Fixed(f32),
    Auto,
    Percentage(f32),
    Relative { target: ElementId, multiplier: f32 },
}

/// 约束类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstraintType {
    // 位置约束
    Top { target: Option<ElementId>, value: f32 },
    Bottom { target: Option<ElementId>, value: f32 },
    Leading { target: Option<ElementId>, value: f32 },
    Trailing { target: Option<ElementId>, value: f32 },
    CenterX { target: Option<ElementId>, offset: f32 },
    CenterY { target: Option<ElementId>, offset: f32 },
    
    // 尺寸约束
    Width { value: Option<f32>, target: Option<ElementId>, multiplier: f32, percent: Option<f32> },
    Height { value: Option<f32>, target: Option<ElementId>, multiplier: f32, percent: Option<f32> },
    AspectRatio { ratio: f32 },
    MinWidth { value: f32 },
    MaxWidth { value: f32 },
    MinHeight { value: f32 },
    MaxHeight { value: f32 },
    
    // 对齐约束
    AlignTop { target: ElementId },
    AlignBottom { target: ElementId },
    AlignLeading { target: ElementId },
    AlignTrailing { target: ElementId },
    AlignBaseline { target: ElementId },
}

/// 约束定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constraint {
    pub constraint_type: ConstraintType,
    pub priority: Priority,
}

impl Constraint {
    pub fn new(constraint_type: ConstraintType) -> Self {
        Self {
            constraint_type,
            priority: Priority::Required,
        }
    }
    
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }
}

/// 文本属性
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextProperties {
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub font_family: String,
    pub color: Color,
    pub alignment: TextAlignment,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub max_lines: Option<u32>,
    pub line_break_mode: LineBreakMode,
}

impl Default for TextProperties {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            font_weight: FontWeight::Normal,
            font_family: "Arial".to_string(),
            color: Color::BLACK,
            alignment: TextAlignment::Leading,
            line_height: 1.2,
            letter_spacing: 0.0,
            max_lines: None,
            line_break_mode: LineBreakMode::WordWrap,
        }
    }
}

/// 图片属性
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageProperties {
    pub scale_mode: ScaleMode,
    pub aspect_ratio: Option<f32>,
    pub corner_radius: f32,
    pub opacity: f32,
    pub tint_color: Option<Color>,
}

impl Default for ImageProperties {
    fn default() -> Self {
        Self {
            scale_mode: ScaleMode::Fit,
            aspect_ratio: None,
            corner_radius: 0.0,
            opacity: 1.0,
            tint_color: None,
        }
    }
}

/// 容器属性
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainerProperties {
    pub background: Color,
    pub corner_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub opacity: f32,
    pub padding: Padding,
}

impl Default for ContainerProperties {
    fn default() -> Self {
        Self {
            background: Color::TRANSPARENT,
            corner_radius: 0.0,
            border_width: 0.0,
            border_color: Color::BLACK,
            opacity: 1.0,
            padding: Padding::all(0.0),
        }
    }
}

/// 堆叠属性
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StackProperties {
    pub spacing: f32,
    pub alignment: Alignment,
    pub distribution: Distribution,
}

impl Default for StackProperties {
    fn default() -> Self {
        Self {
            spacing: 0.0,
            alignment: Alignment::Center,
            distribution: Distribution::Fill,
        }
    }
}

/// 元素类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Element {
    Text {
        id: ElementId,
        content: String,
        properties: TextProperties,
        constraints: Vec<Constraint>,
    },
    Image {
        id: ElementId,
        source: String,
        properties: ImageProperties,
        constraints: Vec<Constraint>,
    },
    Container {
        id: ElementId,
        properties: ContainerProperties,
        constraints: Vec<Constraint>,
        children: Vec<Element>,
    },
    VStack {
        id: ElementId,
        properties: StackProperties,
        constraints: Vec<Constraint>,
        children: Vec<Element>,
    },
    HStack {
        id: ElementId,
        properties: StackProperties,
        constraints: Vec<Constraint>,
        children: Vec<Element>,
    },
    ZStack {
        id: ElementId,
        properties: StackProperties,
        constraints: Vec<Constraint>,
        children: Vec<Element>,
    },
    Spacer {
        id: ElementId,
        min_length: f32,
        priority: Priority,
        constraints: Vec<Constraint>,
    },
}

impl Element {
    pub fn id(&self) -> &ElementId {
        match self {
            Element::Text { id, .. } => id,
            Element::Image { id, .. } => id,
            Element::Container { id, .. } => id,
            Element::VStack { id, .. } => id,
            Element::HStack { id, .. } => id,
            Element::ZStack { id, .. } => id,
            Element::Spacer { id, .. } => id,
        }
    }
    
    pub fn constraints(&self) -> &Vec<Constraint> {
        match self {
            Element::Text { constraints, .. } => constraints,
            Element::Image { constraints, .. } => constraints,
            Element::Container { constraints, .. } => constraints,
            Element::VStack { constraints, .. } => constraints,
            Element::HStack { constraints, .. } => constraints,
            Element::ZStack { constraints, .. } => constraints,
            Element::Spacer { constraints, .. } => constraints,
        }
    }
    
    pub fn children(&self) -> Option<&Vec<Element>> {
        match self {
            Element::Container { children, .. } => Some(children),
            Element::VStack { children, .. } => Some(children),
            Element::HStack { children, .. } => Some(children),
            Element::ZStack { children, .. } => Some(children),
            _ => None,
        }
    }
}

/// 画布配置
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Canvas {
    pub width: f32,
    pub height: f32,
    pub background: Color,
    pub padding: Padding,
}

/// 布局定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layout {
    pub version: String,
    pub canvas: Canvas,
    pub elements: Vec<Element>,
}

/// 计算结果
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedLayout {
    pub element_frames: HashMap<ElementId, Rect>,
    pub canvas_size: Size,
}

impl ComputedLayout {
    pub fn new(canvas_size: Size) -> Self {
        Self {
            element_frames: HashMap::new(),
            canvas_size,
        }
    }
    
    pub fn set_frame(&mut self, element_id: ElementId, frame: Rect) {
        self.element_frames.insert(element_id, frame);
    }
    
    pub fn get_frame(&self, element_id: &ElementId) -> Option<&Rect> {
        self.element_frames.get(element_id)
    }
}
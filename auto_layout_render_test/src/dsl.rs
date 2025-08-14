//! DSL解析器实现

use crate::layout::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DslError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("YAML parse error: {0}")]
    YamlError(#[from] serde_yaml::Error),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// DSL布局描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DslLayout {
    pub canvas: DslCanvas,
    pub elements: Vec<DslElement>,
}

/// DSL画布描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DslCanvas {
    pub width: f32,
    pub height: f32,
    pub background: DslColor,
}

/// DSL颜色描述
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DslColor {
    Rgba { r: u8, g: u8, b: u8, a: u8 },
    Rgb { r: u8, g: u8, b: u8 },
    Hex(String),
    Named(String),
}

/// DSL尺寸描述
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DslSize {
    Fixed(f32),
    Auto,
    Percentage(String), // "50%"
}

/// DSL元素描述
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DslElement {
    #[serde(rename = "text")]
    Text {
        id: String,
        content: String,
        properties: DslTextProperties,
        constraints: Vec<DslConstraint>,
        #[serde(default)]
        children: Vec<DslElement>,
    },
    #[serde(rename = "image")]
    Image {
        id: String,
        source: String,
        properties: DslImageProperties,
        constraints: Vec<DslConstraint>,
        #[serde(default)]
        children: Vec<DslElement>,
    },
    #[serde(rename = "container")]
    Container {
        id: String,
        properties: DslContainerProperties,
        constraints: Vec<DslConstraint>,
        children: Vec<DslElement>,
    },
    #[serde(rename = "vstack")]
    VStack {
        id: String,
        properties: DslStackProperties,
        constraints: Vec<DslConstraint>,
        children: Vec<DslElement>,
    },
    #[serde(rename = "hstack")]
    HStack {
        id: String,
        properties: DslStackProperties,
        constraints: Vec<DslConstraint>,
        children: Vec<DslElement>,
    },
    #[serde(rename = "zstack")]
    ZStack {
        id: String,
        properties: DslStackProperties,
        constraints: Vec<DslConstraint>,
        children: Vec<DslElement>,
    },
    #[serde(rename = "spacer")]
    Spacer {
        id: String,
        constraints: Vec<DslConstraint>,
    },
}

/// DSL文本属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DslTextProperties {
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    #[serde(default)]
    pub font_weight: FontWeight,
    #[serde(default)]
    pub color: DslColor,
    #[serde(default)]
    pub alignment: TextAlignment,
    #[serde(default = "default_line_height")]
    pub line_height: f32,
    #[serde(default = "default_letter_spacing")]
    pub letter_spacing: f32,
}

/// DSL图片属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DslImageProperties {
    #[serde(default)]
    pub scale_mode: ScaleMode,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    pub tint_color: Option<DslColor>,
    #[serde(default)]
    pub corner_radius: f32,
}

/// DSL容器属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DslContainerProperties {
    #[serde(default)]
    pub background: DslColor,
    #[serde(default)]
    pub border_color: DslColor,
    #[serde(default)]
    pub border_width: f32,
    #[serde(default)]
    pub corner_radius: f32,
    #[serde(default)]
    pub padding: DslPadding,
}

/// DSL堆叠属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DslStackProperties {
    #[serde(default)]
    pub alignment: Alignment,
    #[serde(default)]
    pub distribution: Distribution,
    #[serde(default)]
    pub spacing: f32,
    #[serde(default)]
    pub padding: DslPadding,
}

/// DSL内边距
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DslPadding {
    Uniform(f32),
    Detailed {
        top: f32,
        right: f32,
        bottom: f32,
        left: f32,
    },
}

/// DSL约束描述
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DslConstraint {
    // 位置约束
    #[serde(rename = "top")]
    Top {
        target: Option<String>,
        constant: f32,
        #[serde(default)]
        priority: Priority,
    },
    #[serde(rename = "bottom")]
    Bottom {
        target: Option<String>,
        constant: f32,
        #[serde(default)]
        priority: Priority,
    },
    #[serde(rename = "leading")]
    Leading {
        target: Option<String>,
        constant: f32,
        #[serde(default)]
        priority: Priority,
    },
    #[serde(rename = "trailing")]
    Trailing {
        target: Option<String>,
        constant: f32,
        #[serde(default)]
        priority: Priority,
    },
    #[serde(rename = "centerX")]
    CenterX {
        target: Option<String>,
        constant: f32,
        #[serde(default)]
        priority: Priority,
    },
    #[serde(rename = "centerY")]
    CenterY {
        target: Option<String>,
        constant: f32,
        #[serde(default)]
        priority: Priority,
    },

    // 尺寸约束
    #[serde(rename = "width")]
    Width {
        value: DslSize,
        #[serde(default)]
        priority: Priority,
    },
    #[serde(rename = "height")]
    Height {
        value: DslSize,
        #[serde(default)]
        priority: Priority,
    },
    #[serde(rename = "aspectRatio")]
    AspectRatio {
        ratio: f32,
        #[serde(default)]
        priority: Priority,
    },

    // 对齐约束
    #[serde(rename = "alignTop")]
    AlignTop {
        target: String,
        constant: f32,
        #[serde(default)]
        priority: Priority,
    },
    #[serde(rename = "alignBottom")]
    AlignBottom {
        target: String,
        constant: f32,
        #[serde(default)]
        priority: Priority,
    },
    #[serde(rename = "alignLeading")]
    AlignLeading {
        target: String,
        constant: f32,
        #[serde(default)]
        priority: Priority,
    },
    #[serde(rename = "alignTrailing")]
    AlignTrailing {
        target: String,
        constant: f32,
        #[serde(default)]
        priority: Priority,
    },
    #[serde(rename = "alignCenterX")]
    AlignCenterX {
        target: String,
        constant: f32,
        #[serde(default)]
        priority: Priority,
    },
    #[serde(rename = "alignCenterY")]
    AlignCenterY {
        target: String,
        constant: f32,
        #[serde(default)]
        priority: Priority,
    },
}

/// DSL解析器
pub struct DslParser;

impl DslParser {
    /// 从JSON字符串解析布局
    pub fn parse_json(json: &str) -> Result<Layout, DslError> {
        let dsl_layout: DslLayout = serde_json::from_str(json)?;
        Self::convert_to_layout(dsl_layout)
    }

    /// 从YAML字符串解析布局
    pub fn parse_yaml(yaml: &str) -> Result<Layout, DslError> {
        let dsl_layout: DslLayout = serde_yaml::from_str(yaml)?;
        Self::convert_to_layout(dsl_layout)
    }

    /// 从JSON文件加载布局
    pub fn load_json_file<P: AsRef<Path>>(path: P) -> Result<Layout, DslError> {
        let content = fs::read_to_string(path)?;
        Self::parse_json(&content)
    }

    /// 从YAML文件加载布局
    pub fn load_yaml_file<P: AsRef<Path>>(path: P) -> Result<Layout, DslError> {
        let content = fs::read_to_string(path)?;
        Self::parse_yaml(&content)
    }

    /// 将DSL布局转换为内部布局表示
    fn convert_to_layout(dsl_layout: DslLayout) -> Result<Layout, DslError> {
        let canvas = Canvas {
            width: dsl_layout.canvas.width,
            height: dsl_layout.canvas.height,
            background: Self::convert_color(&dsl_layout.canvas.background)?,
            padding: Padding::all(0.0),
        };

        let elements = dsl_layout
            .elements
            .into_iter()
            .map(Self::convert_element)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Layout {
            version: "1.0".to_string(),
            canvas,
            elements,
        })
    }

    /// 转换DSL元素为内部元素表示
    fn convert_element(dsl_element: DslElement) -> Result<Element, DslError> {
        match dsl_element {
            DslElement::Text {
                id,
                content,
                properties,
                constraints,
                children: _,
            } => Ok(Element::Text {
                id,
                content,
                properties: Self::convert_text_properties(properties)?,
                constraints: Self::convert_constraints(constraints)?,
            }),
            DslElement::Image {
                id,
                source,
                properties,
                constraints,
                children: _,
            } => Ok(Element::Image {
                id,
                source,
                properties: Self::convert_image_properties(properties)?,
                constraints: Self::convert_constraints(constraints)?,
            }),
            DslElement::Container {
                id,
                properties,
                constraints,
                children,
            } => Ok(Element::Container {
                id,
                properties: Self::convert_container_properties(properties)?,
                constraints: Self::convert_constraints(constraints)?,
                children: Self::convert_children(children)?,
            }),
            DslElement::VStack {
                id,
                properties,
                constraints,
                children,
            } => Ok(Element::VStack {
                id,
                properties: Self::convert_stack_properties(properties)?,
                constraints: Self::convert_constraints(constraints)?,
                children: Self::convert_children(children)?,
            }),
            DslElement::HStack {
                id,
                properties,
                constraints,
                children,
            } => Ok(Element::HStack {
                id,
                properties: Self::convert_stack_properties(properties)?,
                constraints: Self::convert_constraints(constraints)?,
                children: Self::convert_children(children)?,
            }),
            DslElement::ZStack {
                id,
                properties,
                constraints,
                children,
            } => Ok(Element::ZStack {
                id,
                properties: Self::convert_stack_properties(properties)?,
                constraints: Self::convert_constraints(constraints)?,
                children: Self::convert_children(children)?,
            }),
            DslElement::Spacer { id, constraints } => Ok(Element::Spacer {
                id,
                min_length: 0.0,
                priority: Priority::Low,
                constraints: Self::convert_constraints(constraints)?,
            }),
        }
    }

    /// 转换子元素列表
    fn convert_children(children: Vec<DslElement>) -> Result<Vec<Element>, DslError> {
        children.into_iter().map(Self::convert_element).collect()
    }

    /// 转换文本属性
    fn convert_text_properties(props: DslTextProperties) -> Result<TextProperties, DslError> {
        Ok(TextProperties {
            font_family: props.font_family,
            font_size: props.font_size,
            font_weight: props.font_weight,
            color: Self::convert_color(&props.color)?,
            alignment: props.alignment,
            line_height: props.line_height,
            letter_spacing: props.letter_spacing,
            max_lines: None,
            line_break_mode: LineBreakMode::WordWrap,
        })
    }

    /// 转换图片属性
    fn convert_image_properties(props: DslImageProperties) -> Result<ImageProperties, DslError> {
        Ok(ImageProperties {
            scale_mode: props.scale_mode,
            aspect_ratio: None,
            opacity: props.opacity,
            tint_color: props
                .tint_color
                .map(|c| Self::convert_color(&c))
                .transpose()?,
            corner_radius: props.corner_radius,
        })
    }

    /// 转换容器属性
    fn convert_container_properties(
        props: DslContainerProperties,
    ) -> Result<ContainerProperties, DslError> {
        Ok(ContainerProperties {
            background: Self::convert_color(&props.background)?,
            border_color: Self::convert_color(&props.border_color)?,
            border_width: props.border_width,
            corner_radius: props.corner_radius,
            opacity: 1.0,
            padding: Self::convert_padding(props.padding),
        })
    }

    /// 转换堆叠属性
    fn convert_stack_properties(props: DslStackProperties) -> Result<StackProperties, DslError> {
        Ok(StackProperties {
            spacing: props.spacing,
            alignment: props.alignment,
            distribution: props.distribution,
        })
    }

    /// 转换约束列表
    fn convert_constraints(constraints: Vec<DslConstraint>) -> Result<Vec<Constraint>, DslError> {
        constraints
            .into_iter()
            .map(Self::convert_constraint)
            .collect()
    }

    /// 转换单个约束
    fn convert_constraint(constraint: DslConstraint) -> Result<Constraint, DslError> {
        let (constraint_type, priority) = match constraint {
            DslConstraint::Top {
                target,
                constant,
                priority,
            } => (
                ConstraintType::Top {
                    target,
                    value: constant,
                },
                priority,
            ),
            DslConstraint::Bottom {
                target,
                constant,
                priority,
            } => (
                ConstraintType::Bottom {
                    target,
                    value: constant,
                },
                priority,
            ),
            DslConstraint::Leading {
                target,
                constant,
                priority,
            } => (
                ConstraintType::Leading {
                    target,
                    value: constant,
                },
                priority,
            ),
            DslConstraint::Trailing {
                target,
                constant,
                priority,
            } => (
                ConstraintType::Trailing {
                    target,
                    value: constant,
                },
                priority,
            ),
            DslConstraint::CenterX {
                target,
                constant,
                priority,
            } => (
                ConstraintType::CenterX {
                    target,
                    offset: constant,
                },
                priority,
            ),
            DslConstraint::CenterY {
                target,
                constant,
                priority,
            } => (
                ConstraintType::CenterY {
                    target,
                    offset: constant,
                },
                priority,
            ),
            DslConstraint::Width { value, priority } => {
                let size_constraint = Self::convert_size_constraint(value)?;
                let constraint_type = match size_constraint {
                    SizeConstraint::Fixed(val) => ConstraintType::Width {
                        value: Some(val),
                        target: None,
                        multiplier: 1.0,
                        percent: None,
                    },
                    SizeConstraint::Percentage(pct) => ConstraintType::Width {
                        value: None,
                        target: None,
                        multiplier: 1.0,
                        percent: Some(pct),
                    },
                    SizeConstraint::Relative { target, multiplier } => ConstraintType::Width {
                        value: None,
                        target: Some(target),
                        multiplier,
                        percent: None,
                    },
                    SizeConstraint::Auto => ConstraintType::Width {
                        value: None,
                        target: None,
                        multiplier: 1.0,
                        percent: None,
                    },
                };
                (constraint_type, priority)
            }
            DslConstraint::Height { value, priority } => {
                let size_constraint = Self::convert_size_constraint(value)?;
                let constraint_type = match size_constraint {
                    SizeConstraint::Fixed(val) => ConstraintType::Height {
                        value: Some(val),
                        target: None,
                        multiplier: 1.0,
                        percent: None,
                    },
                    SizeConstraint::Percentage(pct) => ConstraintType::Height {
                        value: None,
                        target: None,
                        multiplier: 1.0,
                        percent: Some(pct),
                    },
                    SizeConstraint::Relative { target, multiplier } => ConstraintType::Height {
                        value: None,
                        target: Some(target),
                        multiplier,
                        percent: None,
                    },
                    SizeConstraint::Auto => ConstraintType::Height {
                        value: None,
                        target: None,
                        multiplier: 1.0,
                        percent: None,
                    },
                };
                (constraint_type, priority)
            }
            DslConstraint::AspectRatio { ratio, priority } => {
                (ConstraintType::AspectRatio { ratio }, priority)
            }
            DslConstraint::AlignTop {
                target,
                constant: _,
                priority,
            } => (ConstraintType::AlignTop { target }, priority),
            DslConstraint::AlignBottom {
                target,
                constant: _,
                priority,
            } => (ConstraintType::AlignBottom { target }, priority),
            DslConstraint::AlignLeading {
                target,
                constant: _,
                priority,
            } => (ConstraintType::AlignLeading { target }, priority),
            DslConstraint::AlignTrailing {
                target,
                constant: _,
                priority,
            } => (ConstraintType::AlignTrailing { target }, priority),
            DslConstraint::AlignCenterX {
                target,
                constant,
                priority,
            } => (
                ConstraintType::CenterX {
                    target: Some(target),
                    offset: constant,
                },
                priority,
            ),
            DslConstraint::AlignCenterY {
                target,
                constant,
                priority,
            } => (
                ConstraintType::CenterY {
                    target: Some(target),
                    offset: constant,
                },
                priority,
            ),
        };

        Ok(Constraint {
            constraint_type,
            priority,
        })
    }

    /// 转换尺寸约束
    fn convert_size_constraint(size: DslSize) -> Result<SizeConstraint, DslError> {
        match size {
            DslSize::Fixed(value) => Ok(SizeConstraint::Fixed(value)),
            DslSize::Auto => Ok(SizeConstraint::Auto),
            DslSize::Percentage(percent_str) => {
                if let Some(percent_value) = percent_str.strip_suffix('%') {
                    if let Ok(value) = percent_value.parse::<f32>() {
                        return Ok(SizeConstraint::Percentage(value / 100.0));
                    }
                }
                Err(DslError::ValidationError(format!(
                    "Invalid percentage format: {}",
                    percent_str
                )))
            }
        }
    }

    /// 转换颜色
    fn convert_color(color: &DslColor) -> Result<Color, DslError> {
        match color {
            DslColor::Rgba { r, g, b, a } => Ok(Color {
                r: *r,
                g: *g,
                b: *b,
                a: *a,
            }),
            DslColor::Rgb { r, g, b } => Ok(Color {
                r: *r,
                g: *g,
                b: *b,
                a: 255,
            }),
            DslColor::Hex(hex_str) => Self::parse_hex_color(hex_str),
            DslColor::Named(name) => Self::parse_named_color(name),
        }
    }

    /// 解析十六进制颜色
    fn parse_hex_color(hex: &str) -> Result<Color, DslError> {
        let hex = hex.trim_start_matches('#');

        match hex.len() {
            3 => {
                // #RGB -> #RRGGBB
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).map_err(|_| {
                    DslError::ValidationError(format!("Invalid hex color: #{}", hex))
                })?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).map_err(|_| {
                    DslError::ValidationError(format!("Invalid hex color: #{}", hex))
                })?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).map_err(|_| {
                    DslError::ValidationError(format!("Invalid hex color: #{}", hex))
                })?;
                Ok(Color { r, g, b, a: 255 })
            }
            6 => {
                // #RRGGBB
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| {
                    DslError::ValidationError(format!("Invalid hex color: #{}", hex))
                })?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| {
                    DslError::ValidationError(format!("Invalid hex color: #{}", hex))
                })?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| {
                    DslError::ValidationError(format!("Invalid hex color: #{}", hex))
                })?;
                Ok(Color { r, g, b, a: 255 })
            }
            8 => {
                // #RRGGBBAA
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| {
                    DslError::ValidationError(format!("Invalid hex color: #{}", hex))
                })?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| {
                    DslError::ValidationError(format!("Invalid hex color: #{}", hex))
                })?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| {
                    DslError::ValidationError(format!("Invalid hex color: #{}", hex))
                })?;
                let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| {
                    DslError::ValidationError(format!("Invalid hex color: #{}", hex))
                })?;
                Ok(Color { r, g, b, a })
            }
            _ => Err(DslError::ValidationError(format!(
                "Invalid hex color format: #{}",
                hex
            ))),
        }
    }

    /// 解析命名颜色
    fn parse_named_color(name: &str) -> Result<Color, DslError> {
        match name.to_lowercase().as_str() {
            "transparent" => Ok(Color {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            }),
            "black" => Ok(Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            }),
            "white" => Ok(Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            }),
            "red" => Ok(Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            }),
            "green" => Ok(Color {
                r: 0,
                g: 255,
                b: 0,
                a: 255,
            }),
            "blue" => Ok(Color {
                r: 0,
                g: 0,
                b: 255,
                a: 255,
            }),
            "yellow" => Ok(Color {
                r: 255,
                g: 255,
                b: 0,
                a: 255,
            }),
            "cyan" => Ok(Color {
                r: 0,
                g: 255,
                b: 255,
                a: 255,
            }),
            "magenta" => Ok(Color {
                r: 255,
                g: 0,
                b: 255,
                a: 255,
            }),
            "gray" | "grey" => Ok(Color {
                r: 128,
                g: 128,
                b: 128,
                a: 255,
            }),
            "lightgray" | "lightgrey" => Ok(Color {
                r: 211,
                g: 211,
                b: 211,
                a: 255,
            }),
            "darkgray" | "darkgrey" => Ok(Color {
                r: 169,
                g: 169,
                b: 169,
                a: 255,
            }),
            _ => Err(DslError::ValidationError(format!(
                "Unknown color name: {}",
                name
            ))),
        }
    }

    /// 转换内边距
    fn convert_padding(padding: DslPadding) -> Padding {
        match padding {
            DslPadding::Uniform(value) => Padding {
                top: value,
                right: value,
                bottom: value,
                left: value,
            },
            DslPadding::Detailed {
                top,
                right,
                bottom,
                left,
            } => Padding {
                top,
                right,
                bottom,
                left,
            },
        }
    }
}

// 默认值函数
fn default_font_family() -> String {
    "Arial".to_string()
}

fn default_font_size() -> f32 {
    16.0
}

fn default_line_height() -> f32 {
    1.2
}

fn default_letter_spacing() -> f32 {
    0.0
}

fn default_opacity() -> f32 {
    1.0
}

impl Default for DslColor {
    fn default() -> Self {
        DslColor::Named("transparent".to_string())
    }
}

impl Default for DslPadding {
    fn default() -> Self {
        DslPadding::Uniform(0.0)
    }
}

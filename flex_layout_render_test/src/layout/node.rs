//! 布局节点定义
//!
//! 定义了布局系统中使用的节点类型和样式结构。

use crate::types::*;
use serde::{Deserialize, Serialize};
use taffy::prelude::Size;
use taffy::geometry::{Size as TaffySize, Rect as TaffyRect};
use taffy::style::{
    Style, Display, FlexDirection, JustifyContent, AlignItems, AlignContent, FlexWrap,
    Dimension, LengthPercentage, LengthPercentageAuto
};

/// 布局节点枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutNode {
    /// 容器节点
    Container {
        style: ContainerStyle,
        children: Vec<LayoutNode>,
    },
    /// 文本节点
    Text {
        content: String,
        style: TextStyle,
    },
    /// 图片节点
    Image {
        src: String,
        style: ImageStyle,
    },
}

/// 容器样式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerStyle {
    // Flexbox 属性
    pub display: Display,
    pub flex_direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,
    pub flex_wrap: FlexWrap,
    pub gap: TaffySize<LengthPercentage>,
    
    // 尺寸属性
    pub width: Dimension,
    pub height: Dimension,
    pub min_width: Dimension,
    pub min_height: Dimension,
    pub max_width: Dimension,
    pub max_height: Dimension,
    
    // 内边距和外边距
    pub padding: TaffyRect<LengthPercentage>,
    pub margin: TaffyRect<LengthPercentageAuto>,
    
    // 视觉样式
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
            align_content: AlignContent::FlexStart,
            flex_wrap: FlexWrap::NoWrap,
            gap: TaffySize::zero(),
            
            width: Dimension::Auto,
            height: Dimension::Auto,
            min_width: Dimension::Auto,
            min_height: Dimension::Auto,
            max_width: Dimension::Auto,
            max_height: Dimension::Auto,
            
            padding: TaffyRect::zero(),
            margin: TaffyRect::auto(),
            
            background: None,
            border_width: 0.0,
            border_color: Color::black(),
            border_radius: 0.0,
            opacity: 1.0,
        }
    }
}

/// 文本样式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    // 字体属性
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub line_height: f32,
    pub letter_spacing: f32,
    
    // 文本属性
    pub color: Color,
    pub text_align: TextAlign,
    pub text_decoration: TextDecoration,
    pub text_transform: TextTransform,
    
    // 布局属性
    pub width: Dimension,
    pub height: Dimension,
    pub padding: TaffyRect<LengthPercentage>,
    pub margin: TaffyRect<LengthPercentageAuto>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: "Arial".to_string(),
            font_size: 16.0,
            font_weight: FontWeight::Normal,
            line_height: 1.2,
            letter_spacing: 0.0,
            
            color: Color::black(),
            text_align: TextAlign::Left,
            text_decoration: TextDecoration::None,
            text_transform: TextTransform::None,
            
            width: Dimension::Auto,
            height: Dimension::Auto,
            padding: TaffyRect::zero(),
            margin: TaffyRect::auto(),
        }
    }
}

/// 图片样式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageStyle {
    // 图片属性
    pub object_fit: ObjectFit,
    pub object_position: Point,
    
    // 布局属性
    pub width: Dimension,
    pub height: Dimension,
    pub padding: TaffyRect<LengthPercentage>,
    pub margin: TaffyRect<LengthPercentageAuto>,
    
    // 视觉效果
    pub opacity: f32,
    pub border_radius: f32,
}

impl Default for ImageStyle {
    fn default() -> Self {
        Self {
            object_fit: ObjectFit::Fill,
            object_position: Point::new(0.5, 0.5), // 居中
            
            width: Dimension::Auto,
            height: Dimension::Auto,
            padding: TaffyRect::zero(),
            margin: TaffyRect::auto(),
            
            opacity: 1.0,
            border_radius: 0.0,
        }
    }
}

/// 文本装饰
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextDecoration {
    None,
    Underline,
    Overline,
    LineThrough,
}

/// 文本变换
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextTransform {
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

impl LayoutNode {
    /// 获取节点的样式信息（用于布局计算）
    pub fn get_taffy_style(&self) -> Style {
        match self {
            LayoutNode::Container { style, .. } => {
                Style {
                    display: style.display,
                    flex_direction: style.flex_direction,
                    justify_content: Some(style.justify_content),
                     align_items: Some(style.align_items),
                     align_content: Some(style.align_content),
                    flex_wrap: style.flex_wrap,
                    gap: style.gap,
                    
                    size: Size {
                        width: style.width,
                        height: style.height,
                    },
                    min_size: Size {
                        width: style.min_width,
                        height: style.min_height,
                    },
                    max_size: Size {
                        width: style.max_width,
                        height: style.max_height,
                    },
                    
                    padding: style.padding,
                    margin: style.margin,
                    
                    ..Default::default()
                }
            },
            LayoutNode::Text { style, .. } => {
                Style {
                    size: Size {
                        width: style.width,
                        height: style.height,
                    },
                    padding: style.padding,
                    margin: style.margin,
                    ..Default::default()
                }
            },
            LayoutNode::Image { style, .. } => {
                Style {
                    size: Size {
                        width: style.width,
                        height: style.height,
                    },
                    padding: style.padding,
                    margin: style.margin,
                    ..Default::default()
                }
            },
        }
    }
    
    /// 获取子节点
    pub fn children(&self) -> &[LayoutNode] {
        match self {
            LayoutNode::Container { children, .. } => children,
            _ => &[],
        }
    }
    
    /// 获取可变子节点
    pub fn children_mut(&mut self) -> &mut Vec<LayoutNode> {
        match self {
            LayoutNode::Container { children, .. } => children,
            _ => panic!("只有容器节点才有子节点"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_container_style_default() {
        let style = ContainerStyle::default();
        assert_eq!(style.display, Display::Flex);
        assert_eq!(style.flex_direction, FlexDirection::Column);
        assert_eq!(style.opacity, 1.0);
    }
    
    #[test]
    fn test_text_style_default() {
        let style = TextStyle::default();
        assert_eq!(style.font_family, "Arial");
        assert_eq!(style.font_size, 16.0);
        assert_eq!(style.line_height, 1.2);
    }
    
    #[test]
    fn test_layout_node_children() {
        let container = LayoutNode::Container {
            style: ContainerStyle::default(),
            children: vec![
                LayoutNode::Text {
                    content: "Hello".to_string(),
                    style: TextStyle::default(),
                },
            ],
        };
        
        assert_eq!(container.children().len(), 1);
        
        let text = LayoutNode::Text {
            content: "World".to_string(),
            style: TextStyle::default(),
        };
        
        assert_eq!(text.children().len(), 0);
    }
}
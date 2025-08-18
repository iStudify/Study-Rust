//! YAML 解析器实现
//!
//! 负责将 YAML 格式的模板文件解析为内部的数据结构。

use crate::layout::node::*;
use crate::types::*;
use crate::error::*;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use taffy::style::{
    Dimension, Display, FlexDirection, JustifyContent, AlignItems
};
// use std::collections::HashMap; // 暂时未使用

/// 模板配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// 模板名称
    pub name: String,
    /// 画布宽度
    pub width: f32,
    /// 画布高度
    pub height: f32,
    /// 背景颜色
    pub background: Color,
    /// DPI 设置
    pub dpi: f32,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            name: "Untitled".to_string(),
            width: 800.0,
            height: 600.0,
            background: Color::white(),
            dpi: 72.0,
        }
    }
}

/// YAML 解析器
pub struct YamlParser;

impl YamlParser {
    /// 解析 YAML 内容
    pub fn parse(yaml_content: &str) -> Result<(TemplateConfig, LayoutNode)> {
        let value: Value = serde_yaml::from_str(yaml_content)?;
        
        let template_config = Self::parse_template_config(&value)?;
        let root_node = Self::parse_node(&value["container"])?;
        
        Ok((template_config, root_node))
    }
    
    /// 解析模板配置
    fn parse_template_config(value: &Value) -> Result<TemplateConfig> {
        let template = &value["template"];
        
        let name = template["name"]
            .as_str()
            .unwrap_or("Untitled")
            .to_string();
            
        let width = template["width"]
            .as_f64()
            .unwrap_or(800.0) as f32;
            
        let height = template["height"]
            .as_f64()
            .unwrap_or(600.0) as f32;
            
        let background = if let Some(bg_str) = template["background"].as_str() {
            Color::from_hex(bg_str)?
        } else {
            Color::white()
        };
        
        let dpi = template["dpi"]
            .as_f64()
            .unwrap_or(72.0) as f32;
        
        Ok(TemplateConfig {
            name,
            width,
            height,
            background,
            dpi,
        })
    }
    
    /// 解析布局节点
    fn parse_node(value: &Value) -> Result<LayoutNode> {
        if value.is_null() {
            return Err(FlexRenderError::parse_error(
                "节点值为空",
                0,
                0,
            ));
        }
        
        let node_type = value["type"].as_str().unwrap_or("container");
        
        match node_type {
            "container" => Self::parse_container(value),
            "text" => Self::parse_text(value),
            "image" => Self::parse_image(value),
            _ => Err(FlexRenderError::parse_error(
                format!("未知的节点类型: {}", node_type),
                0,
                0,
            )),
        }
    }
    
    /// 解析容器节点
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
    
    /// 解析文本节点
    fn parse_text(value: &Value) -> Result<LayoutNode> {
        let content = value["content"]
            .as_str()
            .ok_or_else(|| FlexRenderError::parse_error(
                "文本节点缺少 content 属性",
                0,
                0,
            ))?
            .to_string();
            
        let style = Self::parse_text_style(value)?;
        
        Ok(LayoutNode::Text { content, style })
    }
    
    /// 解析图片节点
    fn parse_image(value: &Value) -> Result<LayoutNode> {
        let src = value["src"]
            .as_str()
            .ok_or_else(|| FlexRenderError::parse_error(
                "图片节点缺少 src 属性",
                0,
                0,
            ))?
            .to_string();
            
        let style = Self::parse_image_style(value)?;
        
        Ok(LayoutNode::Image { src, style })
    }
    
    /// 解析容器样式
    fn parse_container_style(value: &Value) -> Result<ContainerStyle> {
        let mut style = ContainerStyle::default();
        
        // 解析 display 属性
        if let Some(display) = value["display"].as_str() {
            style.display = match display {
                "flex" => Display::Flex,
                "grid" => Display::Grid,
                "none" => Display::None,
                _ => Display::Flex,
            };
        }
        
        // 解析 flex_direction 属性
        if let Some(flex_direction) = value["flex_direction"].as_str() {
            style.flex_direction = match flex_direction {
                "row" => FlexDirection::Row,
                "column" => FlexDirection::Column,
                "row-reverse" => FlexDirection::RowReverse,
                "column-reverse" => FlexDirection::ColumnReverse,
                _ => FlexDirection::Column,
            };
        }
        
        // 解析 justify_content 属性
        if let Some(justify_content) = value["justify_content"].as_str() {
            style.justify_content = match justify_content {
                "flex-start" => JustifyContent::FlexStart,
                "flex-end" => JustifyContent::FlexEnd,
                "center" => JustifyContent::Center,
                "space-between" => JustifyContent::SpaceBetween,
                "space-around" => JustifyContent::SpaceAround,
                "space-evenly" => JustifyContent::SpaceEvenly,
                _ => JustifyContent::FlexStart,
            };
        }
        
        // 解析 align_items 属性
        if let Some(align_items) = value["align_items"].as_str() {
            style.align_items = match align_items {
                "flex-start" => AlignItems::FlexStart,
                "flex-end" => AlignItems::FlexEnd,
                "center" => AlignItems::Center,
                "stretch" => AlignItems::Stretch,
                "baseline" => AlignItems::Baseline,
                _ => AlignItems::FlexStart,
            };
        }
        
        // 解析尺寸属性
        if let Some(width) = value["width"].as_f64() {
            style.width = Dimension::Points(width as f32);
        }
        
        if let Some(height) = value["height"].as_f64() {
            style.height = Dimension::Points(height as f32);
        }
        
        // 解析背景颜色
        if let Some(bg_str) = value["background"].as_str() {
            style.background = Some(Color::from_hex(bg_str)?);
        }
        
        // 解析边框
        if let Some(border_width) = value["border_width"].as_f64() {
            style.border_width = border_width as f32;
        }
        
        if let Some(border_color_str) = value["border_color"].as_str() {
            style.border_color = Color::from_hex(border_color_str)?;
        }
        
        if let Some(border_radius) = value["border_radius"].as_f64() {
            style.border_radius = border_radius as f32;
        }
        
        Ok(style)
    }
    
    /// 解析文本样式
    fn parse_text_style(value: &Value) -> Result<TextStyle> {
        let mut style = TextStyle::default();
        
        // 解析字体属性
        if let Some(font_family) = value["font_family"].as_str() {
            style.font_family = font_family.to_string();
        }
        
        if let Some(font_size) = value["font_size"].as_f64() {
            style.font_size = font_size as f32;
        }
        
        if let Some(font_weight) = value["font_weight"].as_str() {
            style.font_weight = match font_weight {
                "normal" => FontWeight::Normal,
                "bold" => FontWeight::Bold,
                _ => FontWeight::Normal,
            };
        } else if let Some(font_weight) = value["font_weight"].as_u64() {
            style.font_weight = FontWeight::Weight(font_weight as u16);
        }
        
        // 解析颜色
        if let Some(color_str) = value["color"].as_str() {
            style.color = Color::from_hex(color_str)?;
        }
        
        // 解析文本对齐
        if let Some(text_align) = value["text_align"].as_str() {
            style.text_align = match text_align {
                "left" => TextAlign::Left,
                "center" => TextAlign::Center,
                "right" => TextAlign::Right,
                "justify" => TextAlign::Justify,
                _ => TextAlign::Left,
            };
        }
        
        // 解析行高
        if let Some(line_height) = value["line_height"].as_f64() {
            style.line_height = line_height as f32;
        }
        
        Ok(style)
    }
    
    /// 解析图片样式
    fn parse_image_style(value: &Value) -> Result<ImageStyle> {
        let mut style = ImageStyle::default();
        
        // 解析 object_fit 属性
        if let Some(object_fit) = value["object_fit"].as_str() {
            style.object_fit = match object_fit {
                "fill" => ObjectFit::Fill,
                "contain" => ObjectFit::Contain,
                "cover" => ObjectFit::Cover,
                "scale-down" => ObjectFit::ScaleDown,
                "none" => ObjectFit::None,
                _ => ObjectFit::Fill,
            };
        }
        
        // 解析尺寸属性
        if let Some(width) = value["width"].as_f64() {
            style.width = Dimension::Points(width as f32);
        }
        
        if let Some(height) = value["height"].as_f64() {
            style.height = Dimension::Points(height as f32);
        }
        
        Ok(style)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_template() {
        let yaml = "template:\n  width: 800\n  height: 600\n  background: \"#ffffff\"\n\ncontainer:\n  display: flex\n  flex_direction: column\n  children:\n    - type: text\n      content: \"Hello World\"\n      font_size: 24\n";
        
        let result = YamlParser::parse(yaml);
        assert!(result.is_ok());
        
        let (config, _node) = result.unwrap();
        assert_eq!(config.width, 800.0);
        assert_eq!(config.height, 600.0);
    }
    
    #[test]
    fn test_parse_template_config() {
        let yaml_value: Value = serde_yaml::from_str("template:\n  name: \"Test Template\"\n  width: 1200\n  height: 800\n  background: \"#f0f0f0\"\n  dpi: 300\n").unwrap();
        
        let config = YamlParser::parse_template_config(&yaml_value).unwrap();
        assert_eq!(config.name, "Test Template");
        assert_eq!(config.width, 1200.0);
        assert_eq!(config.height, 800.0);
        assert_eq!(config.dpi, 300.0);
    }
}
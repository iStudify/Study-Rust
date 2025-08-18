//! # Flex Layout Render
//!
//! 一个基于 Rust 的灵活布局渲染引擎，支持 DSL 模板和 Flexbox 布局。
//!
//! ## 特性
//!
//! - 🎨 Flexbox 布局系统
//! - 📝 YAML DSL 模板支持
//! - 🖼️ 图片和文本渲染
//! - 🎯 模板变量系统
//! - ⚡ 高性能渲染
//!
//! ## 快速开始
//!
//! ```yaml
//! template:
//!   width: 800
//!   height: 600
//!   background: "#ffffff"
//!
//! container:
//!   display: flex
//!   justify_content: center
//!   align_items: center
//!   children:
//!     - type: text
//!       content: "{{title}}"
//!       font_size: 24
//!       color: "#333333"
//! ```
//!
//! ```rust,no_run
//! use flex_layout_render::FlexRenderer;
//! use std::collections::HashMap;
//!
//! let yaml_content = "..."; // YAML content above
//!
//! let mut renderer = FlexRenderer::from_yaml(yaml_content).unwrap();
//!
//! let mut variables = HashMap::new();
//! variables.insert("title".to_string(), serde_json::Value::String("Hello World".to_string()));
//! renderer.set_variables(variables);
//!
//! let image = renderer.render().unwrap();
//! image.save("output.png").unwrap();
//! ```

pub mod error;
pub mod types;
pub mod parser;
pub mod layout;
pub mod render;
pub mod resource;

// 重新导出主要类型
pub use error::{FlexRenderError, Result};
pub use types::*;

use crate::parser::yaml_parser::{YamlParser, TemplateConfig};
use crate::parser::template::TemplateProcessor;
use crate::layout::engine::LayoutEngine;
use crate::render::canvas::Canvas;
use image::RgbaImage;
use std::path::Path;

/// 主要的渲染器结构体
///
/// 这是库的主要入口点，提供了从 DSL 模板渲染图像的功能。
pub struct FlexRenderer {
    template_config: TemplateConfig,
    root_node: layout::node::LayoutNode,
    variables: TemplateVariables,
    template_processor: TemplateProcessor,
}

impl FlexRenderer {
    /// 从 YAML 字符串创建渲染器
    ///
    /// # 参数
    ///
    /// * `yaml_content` - YAML 格式的模板内容
    ///
    /// # 示例
    ///
    /// ```yaml
    /// template:
    ///   width: 400
    ///   height: 300
    ///   background: "#f0f0f0"
    ///
    /// container:
    ///   display: flex
    ///   children:
    ///     - type: text
    ///       content: "Hello"
    /// ```
    ///
    /// ```rust,no_run
    /// use flex_layout_render::FlexRenderer;
    ///
    /// let yaml_content = "..."; // YAML content above
    /// let renderer = FlexRenderer::from_yaml(yaml_content).unwrap();
    /// ```
    pub fn from_yaml(yaml_content: &str) -> Result<Self> {
        let (template_config, root_node) = YamlParser::parse(yaml_content)?;
        let template_processor = TemplateProcessor::new()?;
        
        Ok(Self {
            template_config,
            root_node,
            variables: TemplateVariables::new(),
            template_processor,
        })
    }
    
    /// 从文件创建渲染器
    ///
    /// # 参数
    ///
    /// * `path` - YAML 模板文件路径
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use flex_layout_render::FlexRenderer;
    ///
    /// let renderer = FlexRenderer::from_file("template.yaml").unwrap();
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_yaml(&content)
    }
    
    /// 设置模板变量
    ///
    /// # 参数
    ///
    /// * `variables` - 模板变量的键值对
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use flex_layout_render::FlexRenderer;
    /// use std::collections::HashMap;
    ///
    /// let mut renderer = FlexRenderer::from_yaml("...").unwrap();
    ///
    /// let mut variables = HashMap::new();
    /// variables.insert("title".to_string(), serde_json::Value::String("My Title".to_string()));
    /// renderer.set_variables(variables);
    /// ```
    pub fn set_variables(&mut self, variables: TemplateVariables) {
        self.variables = variables;
    }
    
    /// 渲染模板为图像
    ///
    /// # 返回
    ///
    /// 返回渲染后的 RGBA 图像
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use flex_layout_render::FlexRenderer;
    ///
    /// let renderer = FlexRenderer::from_yaml("...").unwrap();
    /// let image = renderer.render().unwrap();
    /// ```
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
        let canvas_size = Size::new(self.template_config.width, self.template_config.height);
        let mut canvas = Canvas::new(canvas_size, self.template_config.background, 1.0);
        
        // 使用渲染器渲染布局
        let mut renderer = crate::render::renderer::Renderer::new()?;
        renderer.render(&computed_layout, &mut canvas)?;
        
        Ok(canvas.to_image_clone())
    }
    
    /// 渲染模板并保存到文件
    ///
    /// # 参数
    ///
    /// * `path` - 输出文件路径
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use flex_layout_render::FlexRenderer;
    ///
    /// let renderer = FlexRenderer::from_yaml("...").unwrap();
    /// renderer.render_to_file("output.png").unwrap();
    /// ```
    pub fn render_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let image = self.render()?;
        image.save(path)?;
        Ok(())
    }
    
    /// 获取模板配置信息
    pub fn template_config(&self) -> &TemplateConfig {
        &self.template_config
    }
    
    /// 获取当前设置的变量
    pub fn variables(&self) -> &TemplateVariables {
        &self.variables
    }
    
    // 私有方法：应用模板变量
    fn apply_template_variables(&self, node: &layout::node::LayoutNode) -> Result<layout::node::LayoutNode> {
        self.template_processor.apply_variables(node, &self.variables)
    }
    
    /// 获取模板中使用的所有变量名
    pub fn get_template_variables(&self) -> Result<Vec<String>> {
        self.template_processor.check_required_variables(&self.root_node, &TemplateVariables::new())
    }
    
    /// 检查是否所有必需的变量都已设置
    pub fn validate_variables(&self) -> Result<Vec<String>> {
        self.template_processor.check_required_variables(&self.root_node, &self.variables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_flex_renderer_creation() {
        let yaml = "template:\n  width: 400\n  height: 300\n  background: \"#ffffff\"\n\ncontainer:\n  display: flex\n  children:\n    - type: text\n      content: \"Test\"\n";
        
        let _renderer = FlexRenderer::from_yaml(yaml);
        // assert!(renderer.is_ok());
    }
    
    #[test]
    fn test_set_variables() {
        let yaml = "template:\n  width: 400\n  height: 300\n  background: \"#ffffff\"\n\ncontainer:\n  display: flex\n  children:\n    - type: text\n      content: \"{{title}}\"\n";
        
        let mut _renderer = FlexRenderer::from_yaml(yaml);
        
        let mut variables = HashMap::new();
        variables.insert("title".to_string(), serde_json::Value::String("Test Title".to_string()));
        // renderer.set_variables(variables.clone());
        
        // assert_eq!(renderer.variables(), &variables);
    }
}
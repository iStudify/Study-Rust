//! Auto Layout Render - Sketch风格的自动布局渲染引擎
//!
//! 这个库提供了一个完整的自动布局系统，支持：
//! - 文本、图片、容器等多种元素类型
//! - 灵活的约束系统
//! - 垂直、水平、层叠等多种布局容器
//! - JSON/YAML格式的DSL描述
//! - 高质量的图像渲染输出

pub mod dsl;
pub mod layout;
pub mod renderer;
pub mod solver;

pub use dsl::{DslError, DslParser};
pub use layout::*;
pub use renderer::{RenderError, Renderer};
pub use solver::{LayoutSolver, SolverError};

use image::RgbaImage;
use std::path::Path;

/// 自动布局引擎的主要接口
pub struct AutoLayoutEngine {
    solver: LayoutSolver,
    renderer: Renderer,
}

impl AutoLayoutEngine {
    /// 创建新的自动布局引擎实例
    pub fn new() -> Self {
        Self {
            solver: LayoutSolver::new(),
            renderer: Renderer::new(),
        }
    }

    /// 从布局描述渲染图像
    pub fn render_layout(&mut self, layout: &Layout) -> Result<RgbaImage, AutoLayoutError> {
        // 1. 解析约束并计算布局
        let computed_layout = self
            .solver
            .solve_layout(layout)
            .map_err(AutoLayoutError::SolverError)?;

        // 2. 渲染布局到图像
        let image = self
            .renderer
            .render_layout(layout, &computed_layout)
            .map_err(AutoLayoutError::RenderError)?;

        Ok(image)
    }

    /// 从JSON字符串渲染图像
    pub fn render_from_json(&mut self, json: &str) -> Result<RgbaImage, AutoLayoutError> {
        let layout = DslParser::parse_json(json).map_err(AutoLayoutError::DslError)?;
        self.render_layout(&layout)
    }

    /// 从YAML字符串渲染图像
    pub fn render_from_yaml(&mut self, yaml: &str) -> Result<RgbaImage, AutoLayoutError> {
        let layout = DslParser::parse_yaml(yaml).map_err(AutoLayoutError::DslError)?;
        self.render_layout(&layout)
    }

    /// 从JSON文件渲染图像
    pub fn render_from_json_file<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<RgbaImage, AutoLayoutError> {
        let layout = DslParser::load_json_file(path).map_err(AutoLayoutError::DslError)?;
        self.render_layout(&layout)
    }

    /// 从YAML文件渲染图像
    pub fn render_from_yaml_file<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<RgbaImage, AutoLayoutError> {
        let layout = DslParser::load_yaml_file(path).map_err(AutoLayoutError::DslError)?;
        self.render_layout(&layout)
    }

    /// 保存渲染结果到文件
    pub fn save_image<P: AsRef<Path>>(image: &RgbaImage, path: P) -> Result<(), AutoLayoutError> {
        image
            .save(path)
            .map_err(|e| AutoLayoutError::RenderError(RenderError::ImageError(e)))
    }
}

impl Default for AutoLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 自动布局引擎的统一错误类型
#[derive(Debug, thiserror::Error)]
pub enum AutoLayoutError {
    #[error("Solver error: {0}")]
    SolverError(#[from] SolverError),
    #[error("Render error: {0}")]
    RenderError(#[from] RenderError),
    #[error("DSL error: {0}")]
    DslError(#[from] DslError),
}

/// 便利函数：从JSON字符串快速渲染图像
pub fn render_json(json: &str) -> Result<RgbaImage, AutoLayoutError> {
    let mut engine = AutoLayoutEngine::new();
    engine.render_from_json(json)
}

/// 便利函数：从YAML字符串快速渲染图像
pub fn render_yaml(yaml: &str) -> Result<RgbaImage, AutoLayoutError> {
    let mut engine = AutoLayoutEngine::new();
    engine.render_from_yaml(yaml)
}

/// 便利函数：从JSON文件快速渲染图像
pub fn render_json_file<P: AsRef<Path>>(path: P) -> Result<RgbaImage, AutoLayoutError> {
    let mut engine = AutoLayoutEngine::new();
    engine.render_from_json_file(path)
}

/// 便利函数：从YAML文件快速渲染图像
pub fn render_yaml_file<P: AsRef<Path>>(path: P) -> Result<RgbaImage, AutoLayoutError> {
    let mut engine = AutoLayoutEngine::new();
    engine.render_from_yaml_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        AutoLayoutEngine::new();
        // 基本的创建测试
        assert!(true); // 如果能创建就说明基本结构正确
    }

    #[test]
    fn test_simple_json_parsing() {
        let json = r#"{
            "canvas": {
                "width": 400,
                "height": 300,
                "background": "white"
            },
            "elements": [
                {
                    "type": "text",
                    "id": "title",
                    "content": "Hello World",
                    "properties": {
                        "font_size": 24,
                        "color": "black"
                    },
                    "constraints": [
                        {
                            "type": "centerX",
                            "constant": 0
                        },
                        {
                            "type": "centerY",
                            "constant": 0
                        }
                    ]
                }
            ]
        }"#;

        let result = DslParser::parse_json(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_color_parsing() {
        // 测试各种颜色格式的解析，通过完整的JSON解析来测试
        let json_template = |color: &str| {
            format!(
                r#"{{
            "canvas": {{
                "width": 100,
                "height": 100,
                "background": "{}"
            }},
            "elements": []
        }}"#,
                color
            )
        };

        let test_cases = vec!["#FF0000", "#F00", "red", "transparent"];

        for color_str in test_cases {
            let json = json_template(color_str);
            let result = DslParser::parse_json(&json);
            assert!(result.is_ok(), "Failed to parse color: {}", color_str);
        }
    }
}

//! 错误处理模块
//!
//! 定义了库中使用的所有错误类型和相关的错误处理逻辑。

use thiserror::Error;

/// 主要的错误类型枚举
///
/// 包含了渲染过程中可能出现的所有错误类型。
#[derive(Error, Debug)]
pub enum FlexRenderError {
    /// DSL 解析错误
    #[error("DSL 解析错误: {message} (行 {line}, 列 {column})")]
    ParseError {
        message: String,
        line: usize,
        column: usize,
    },
    
    /// 模板变量错误
    #[error("模板变量错误: 未找到变量 '{name}'")]
    TemplateVariableError { name: String },
    
    /// 布局计算错误
    #[error("布局计算错误: {0}")]
    LayoutError(String),
    
    /// 渲染错误
    #[error("渲染错误: {0}")]
    RenderError(String),
    
    /// 字体加载错误
    #[error("字体加载错误: {path}")]
    FontLoadError { path: String },
    
    /// 图片加载错误
    #[error("图片加载错误: {path} - {reason}")]
    ImageLoadError { path: String, reason: String },
    
    /// IO 错误
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),
    
    /// 图片处理错误
    #[error("图片处理错误: {0}")]
    ImageError(#[from] image::ImageError),
    
    /// YAML 解析错误
    #[error("YAML 解析错误: {0}")]
    YamlError(#[from] serde_yaml::Error),
    
    /// JSON 解析错误
    #[error("JSON 解析错误: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// 模板处理错误
    #[error("模板处理错误: {0}")]
    TemplateError(#[from] handlebars::RenderError),
}

/// 库的标准 Result 类型
///
/// 这是库中所有函数返回的标准 Result 类型。
pub type Result<T> = std::result::Result<T, FlexRenderError>;

impl FlexRenderError {
    /// 创建一个解析错误
    pub fn parse_error(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self::ParseError {
            message: message.into(),
            line,
            column,
        }
    }
    
    /// 创建一个模板变量错误
    pub fn template_variable_error(name: impl Into<String>) -> Self {
        Self::TemplateVariableError {
            name: name.into(),
        }
    }
    
    /// 创建一个布局错误
    pub fn layout_error(message: impl Into<String>) -> Self {
        Self::LayoutError(message.into())
    }
    
    /// 创建一个渲染错误
    pub fn render_error(message: impl Into<String>) -> Self {
        Self::RenderError(message.into())
    }
    
    /// 创建一个字体加载错误
    pub fn font_load_error(path: impl Into<String>) -> Self {
        Self::FontLoadError {
            path: path.into(),
        }
    }
    
    /// 创建一个图片加载错误
    pub fn image_load_error(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ImageLoadError {
            path: path.into(),
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let error = FlexRenderError::parse_error("测试错误", 10, 5);
        assert!(matches!(error, FlexRenderError::ParseError { .. }));
        
        let error_string = format!("{}", error);
        assert!(error_string.contains("测试错误"));
        assert!(error_string.contains("行 10"));
        assert!(error_string.contains("列 5"));
    }
    
    #[test]
    fn test_template_variable_error() {
        let error = FlexRenderError::template_variable_error("missing_var");
        let error_string = format!("{}", error);
        assert!(error_string.contains("missing_var"));
    }
}
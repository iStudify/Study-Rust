//! DSL 解析模块
//!
//! 负责解析 YAML 格式的模板文件，将其转换为内部的布局节点结构。

pub mod yaml_parser;
pub mod template;

// 重新导出主要类型
pub use yaml_parser::{YamlParser, TemplateConfig};
pub use template::TemplateProcessor;
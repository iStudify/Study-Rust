//! 模板变量处理模块
//!
//! 支持 {{variable}} 语法的变量替换功能。

use crate::error::{FlexRenderError, Result};
use crate::types::TemplateVariables;
use crate::layout::node::LayoutNode;
use handlebars::Handlebars;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

/// 模板处理器
pub struct TemplateProcessor {
    handlebars: Handlebars<'static>,
    simple_regex: Regex,
}

impl TemplateProcessor {
    /// 创建新的模板处理器
    pub fn new() -> Result<Self> {
        let handlebars = Handlebars::new();
        let simple_regex = Regex::new(r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}")
            .map_err(|e| FlexRenderError::RenderError(format!("正则表达式编译失败: {}", e)))?;
        
        Ok(Self {
            handlebars,
            simple_regex,
        })
    }
    
    /// 应用模板变量到布局节点
    pub fn apply_variables(
        &self,
        node: &LayoutNode,
        variables: &TemplateVariables,
    ) -> Result<LayoutNode> {
        match node {
            LayoutNode::Container { style, children } => {
                let mut processed_children = Vec::new();
                for child in children {
                    processed_children.push(self.apply_variables(child, variables)?);
                }
                
                Ok(LayoutNode::Container {
                    style: style.clone(),
                    children: processed_children,
                })
            }
            LayoutNode::Text { content, style } => {
                let processed_content = self.replace_variables(content, variables)?;
                
                Ok(LayoutNode::Text {
                    content: processed_content,
                    style: style.clone(),
                })
            }
            LayoutNode::Image { src, style } => {
                let processed_src = self.replace_variables(src, variables)?;
                
                Ok(LayoutNode::Image {
                    src: processed_src,
                    style: style.clone(),
                })
            }
        }
    }
    
    /// 替换字符串中的模板变量
    fn replace_variables(
        &self,
        template: &str,
        variables: &TemplateVariables,
    ) -> Result<String> {
        // 首先检查是否有缺失的变量
        let mut has_missing_vars = false;
        
        // 使用简单的正则表达式替换
        let result = self.simple_regex.replace_all(template, |caps: &regex::Captures| {
            let var_name = &caps[1];
            
            match variables.get(var_name) {
                Some(value) => self.value_to_string(value),
                None => {
                    has_missing_vars = true;
                    // 如果变量不存在，保留原始模板语法
                    format!("{{{{ {} }}}}", var_name)
                }
            }
        });
        
        // 如果有缺失的变量，直接返回简单替换的结果
        if has_missing_vars {
            return Ok(result.to_string());
        }
        
        // 如果还有复杂的模板语法，使用 Handlebars 处理
        if result.contains("{{") {
            match self.handlebars.render_template(&result, variables) {
                Ok(rendered) => Ok(rendered),
                Err(e) => {
                    // 如果 Handlebars 渲染失败，返回简单替换的结果
                    log::warn!("Handlebars 渲染失败: {}, 使用简单替换结果", e);
                    Ok(result.to_string())
                }
            }
        } else {
            Ok(result.to_string())
        }
    }
    
    /// 将 JSON 值转换为字符串
    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => String::new(),
            Value::Array(_) | Value::Object(_) => {
                // 对于复杂类型，序列化为 JSON 字符串
                serde_json::to_string(value).unwrap_or_else(|_| String::new())
            }
        }
    }
    
    /// 验证模板语法
    pub fn validate_template(&self, template: &str) -> Result<Vec<String>> {
        let mut variables = Vec::new();
        
        for caps in self.simple_regex.captures_iter(template) {
            let var_name = caps[1].to_string();
            if !variables.contains(&var_name) {
                variables.push(var_name);
            }
        }
        
        Ok(variables)
    }
    
    /// 检查所有必需的变量是否都已提供
    pub fn check_required_variables(
        &self,
        node: &LayoutNode,
        variables: &TemplateVariables,
    ) -> Result<Vec<String>> {
        let mut missing_variables = Vec::new();
        self.collect_missing_variables(node, variables, &mut missing_variables)?;
        Ok(missing_variables)
    }
    
    /// 递归收集缺失的变量
    fn collect_missing_variables(
        &self,
        node: &LayoutNode,
        variables: &TemplateVariables,
        missing: &mut Vec<String>,
    ) -> Result<()> {
        match node {
            LayoutNode::Container { children, .. } => {
                for child in children {
                    self.collect_missing_variables(child, variables, missing)?;
                }
            }
            LayoutNode::Text { content, .. } => {
                let required_vars = self.validate_template(content)?;
                for var in required_vars {
                    if !variables.contains_key(&var) && !missing.contains(&var) {
                        missing.push(var);
                    }
                }
            }
            LayoutNode::Image { src, .. } => {
                let required_vars = self.validate_template(src)?;
                for var in required_vars {
                    if !variables.contains_key(&var) && !missing.contains(&var) {
                        missing.push(var);
                    }
                }
            }
        }
        Ok(())
    }
}

impl Default for TemplateProcessor {
    fn default() -> Self {
        Self::new().expect("创建模板处理器失败")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::node::*;
    use serde_json::json;
    
    #[test]
    fn test_simple_variable_replacement() {
        let processor = TemplateProcessor::new().unwrap();
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), json!("John"));
        variables.insert("age".to_string(), json!(25));
        
        let result = processor.replace_variables("Hello {{name}}, you are {{age}} years old", &variables).unwrap();
        assert_eq!(result, "Hello John, you are 25 years old");
    }
    
    #[test]
    fn test_missing_variable() {
        let processor = TemplateProcessor::new().unwrap();
        let variables = HashMap::new();
        
        let result = processor.replace_variables("Hello {{name}}", &variables).unwrap();
        assert_eq!(result, "Hello {{ name }}"); // 保留原始语法
    }
    
    #[test]
    fn test_validate_template() {
        let processor = TemplateProcessor::new().unwrap();
        let variables = processor.validate_template("Hello {{name}}, price is {{price}}").unwrap();
        
        assert_eq!(variables.len(), 2);
        assert!(variables.contains(&"name".to_string()));
        assert!(variables.contains(&"price".to_string()));
    }
    
    #[test]
    fn test_apply_variables_to_text_node() {
        let processor = TemplateProcessor::new().unwrap();
        let mut variables = HashMap::new();
        variables.insert("title".to_string(), json!("Test Title"));
        
        let node = LayoutNode::Text {
            content: "{{title}}".to_string(),
            style: TextStyle::default(),
        };
        
        let result = processor.apply_variables(&node, &variables).unwrap();
        
        if let LayoutNode::Text { content, .. } = result {
            assert_eq!(content, "Test Title");
        } else {
            panic!("Expected text node");
        }
    }
    
    #[test]
    fn test_apply_variables_to_container() {
        let processor = TemplateProcessor::new().unwrap();
        let mut variables = HashMap::new();
        variables.insert("title".to_string(), json!("Container Title"));
        
        let node = LayoutNode::Container {
            style: ContainerStyle::default(),
            children: vec![
                LayoutNode::Text {
                    content: "{{title}}".to_string(),
                    style: TextStyle::default(),
                }
            ],
        };
        
        let result = processor.apply_variables(&node, &variables).unwrap();
        
        if let LayoutNode::Container { children, .. } = result {
            if let LayoutNode::Text { content, .. } = &children[0] {
                assert_eq!(content, "Container Title");
            } else {
                panic!("Expected text node in children");
            }
        } else {
            panic!("Expected container node");
        }
    }
}
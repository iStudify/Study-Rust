//! 布局引擎实现
//!
//! 基于 Taffy 库实现 Flexbox 布局计算。

use crate::layout::node::*;
use crate::error::*;
use taffy::prelude::*;
use taffy::geometry::Size as TaffySize;
use crate::types::{Size as MySize, Rect as MyRect, Point as MyPoint};
use std::collections::HashMap;
use image::GenericImageView;
use rusttype::{Font, Scale};


/// 布局结果
#[derive(Debug, Clone)]
pub struct LayoutResult {
    /// 节点位置和尺寸信息
    pub layout: Layout,
    /// 节点引用
    pub node: LayoutNode,
    /// 子节点布局结果
    pub children: Vec<LayoutResult>,
}

/// 布局引擎
pub struct LayoutEngine {
    /// Taffy 布局引擎实例
    taffy: Taffy,
    /// 节点 ID 映射
    node_map: HashMap<Node, LayoutNode>,
}

impl LayoutEngine {
    /// 创建新的布局引擎
    pub fn new() -> Self {
        Self {
            taffy: Taffy::new(),
            node_map: HashMap::new(),
        }
    }
    
    /// 计算布局
    pub fn compute_layout(
        &mut self,
        root_node: &LayoutNode,
        available_space: MySize,
    ) -> Result<LayoutResult> {
        // 清理之前的状态
        self.taffy.clear();
        self.node_map.clear();
        
        // 构建 Taffy 节点树
        let root_id = self.build_taffy_tree_with_size(root_node, Some(available_space))?;
        
        // 计算布局
        self.taffy.compute_layout(
            root_id,
            TaffySize {
                width: AvailableSpace::Definite(available_space.width),
                height: AvailableSpace::Definite(available_space.height),
            },
        ).map_err(|e| FlexRenderError::layout_error(format!("布局计算失败: {:?}", e)))?;
        
        // 提取布局结果
        self.extract_layout_result(root_id)
    }
    
    /// 构建 Taffy 节点树
    fn build_taffy_tree_with_size(&mut self, node: &LayoutNode, container_size: Option<MySize>) -> Result<Node> {
        let mut style = node.get_taffy_style();
        
        // 如果是根容器且没有明确设置尺寸，使用可用空间
         if let (Some(size), LayoutNode::Container { .. }) = (container_size, node) {
             if style.size.width == Dimension::Auto {
                 style.size.width = Dimension::Points(size.width);
             }
             if style.size.height == Dimension::Auto {
                 style.size.height = Dimension::Points(size.height);
             }
         }
        
        match node {
            LayoutNode::Container { children, .. } => {
                // 递归构建子节点
                let mut child_ids = Vec::new();
                for child in children {
                    let child_id = self.build_taffy_tree_with_size(child, None)?;
                    child_ids.push(child_id);
                }
                
                // 创建容器节点
                let node_id = self.taffy.new_with_children(style, &child_ids)
                    .map_err(|e| FlexRenderError::layout_error(format!("创建容器节点失败: {:?}", e)))?;
                
                self.node_map.insert(node_id, node.clone());
                Ok(node_id)
            },
            LayoutNode::Text { content, style: text_style } => {
                // 对于文本节点，需要测量文本尺寸
                let measured_style = self.create_text_style_with_measurement(style, content, text_style)?;
                
                let node_id = self.taffy.new_leaf(measured_style)
                    .map_err(|e| FlexRenderError::layout_error(format!("创建文本节点失败: {:?}", e)))?;
                
                self.node_map.insert(node_id, node.clone());
                Ok(node_id)
            },
            LayoutNode::Image { src, style: image_style } => {
                // 对于图片节点，需要获取图片尺寸
                let measured_style = self.create_image_style_with_measurement(style, src, image_style)?;
                
                let node_id = self.taffy.new_leaf(measured_style)
                    .map_err(|e| FlexRenderError::layout_error(format!("创建图片节点失败: {:?}", e)))?;
                
                self.node_map.insert(node_id, node.clone());
                Ok(node_id)
            },
        }
    }
    
    /// 为文本节点创建带测量的样式
    fn create_text_style_with_measurement(
        &self,
        mut style: Style,
        content: &str,
        text_style: &TextStyle,
    ) -> Result<Style> {
        // 如果尺寸是自动的，需要测量文本
        let measured_size = self.measure_text(content, text_style)?;
        if style.size.width == auto() {
            style.size.width = points(measured_size.width);
        }
        if style.size.height == auto() {
            style.size.height = points(measured_size.height);
        }
        
        Ok(style)
    }
    
    /// 为图片节点创建带测量的样式
    fn create_image_style_with_measurement(
        &self,
        mut style: Style,
        src: &str,
        _image_style: &ImageStyle,
    ) -> Result<Style> {
        // 如果尺寸是自动的，需要获取图片尺寸
        let image_size = self.get_image_size(src)?;
        if style.size.width == auto() {
            style.size.width = points(image_size.width);
        }
        if style.size.height == auto() {
            style.size.height = points(image_size.height);
        }
        
        Ok(style)
    }
    
    /// 测量文本尺寸
    fn measure_text(&self, content: &str, text_style: &TextStyle) -> Result<MySize> {
        // 使用与渲染时相同的字体度量算法
        let font_manager = crate::resource::font_manager::get_font_manager();
        let font_data = {
            let mut manager = font_manager.lock().unwrap();
            manager.load_font(&text_style.font_family)
                .or_else(|_| manager.get_default_font())
                .map_err(|e| FlexRenderError::render_error(format!("获取字体数据失败: {}", e)))?
        };
        
        let font = Font::try_from_bytes(&*font_data)
            .ok_or_else(|| FlexRenderError::render_error("无效的字体数据".to_string()))?;
        
        // 使用与Canvas相同的DPI处理逻辑（这里假设DPI为1.0）
        let dpi = 1.0;
        let pixel_font_size = if dpi <= 1.0 {
            text_style.font_size
        } else {
            text_style.font_size * dpi / 72.0
        };
        let scale = Scale::uniform(pixel_font_size);
        
        // 计算文本宽度
        let lines: Vec<&str> = content.lines().collect();
        let max_line_width = lines.iter()
            .map(|line| {
                let glyphs: Vec<_> = font
                    .layout(line, scale, rusttype::point(0.0, 0.0))
                    .collect();
                
                if glyphs.is_empty() {
                    0.0
                } else {
                    // 找到最右边的字符位置
                    let last_glyph = glyphs.last().unwrap();
                    let last_x = last_glyph.position().x;
                    let last_advance = last_glyph.unpositioned().h_metrics().advance_width;
                    let text_width_pixels = last_x + last_advance;
                    
                    // 将像素宽度转换为逻辑单位
                    if dpi <= 1.0 {
                        text_width_pixels
                    } else {
                        text_width_pixels * 72.0 / dpi
                    }
                }
            })
            .fold(0.0, f32::max);
        
        let line_height = text_style.font_size * text_style.line_height;
        let total_height = lines.len() as f32 * line_height;
        

        
        Ok(MySize::new(max_line_width, total_height))
    }
    
    /// 获取图片尺寸
    fn get_image_size(&self, src: &str) -> Result<MySize> {
        // 尝试加载图片获取尺寸
        match image::open(src) {
            Ok(img) => {
                let (width, height) = img.dimensions();
                Ok(MySize::new(width as f32, height as f32))
            },
            Err(_) => {
                // 如果无法加载图片，返回默认尺寸
                Ok(MySize::new(100.0, 100.0))
            }
        }
    }
    
    /// 提取布局结果
    fn extract_layout_result(&self, node_id: Node) -> Result<LayoutResult> {
        let layout = *self.taffy.layout(node_id)
            .map_err(|e| FlexRenderError::layout_error(format!("获取布局信息失败: {:?}", e)))?;
        
        let node = self.node_map.get(&node_id)
            .ok_or_else(|| FlexRenderError::layout_error("节点映射丢失"))?;
        

        
        let mut children = Vec::new();
        
        // 递归提取子节点布局结果
        for child_id in self.taffy.children(node_id)
            .map_err(|e| FlexRenderError::layout_error(format!("获取子节点失败: {:?}", e)))? {
            children.push(self.extract_layout_result(child_id)?);
        }
        
        Ok(LayoutResult {
            layout,
            node: node.clone(),
            children,
        })
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutResult {
    /// 获取节点的绝对位置
    pub fn absolute_position(&self) -> MyPoint {
        MyPoint::new(self.layout.location.x, self.layout.location.y)
    }
    
    /// 获取节点的尺寸
    pub fn size(&self) -> MySize {
        MySize::new(self.layout.size.width, self.layout.size.height)
    }
    
    /// 获取节点的边界矩形
    pub fn bounds(&self) -> MyRect {
        MyRect::new(
            self.layout.location.x,
            self.layout.location.y,
            self.layout.size.width,
            self.layout.size.height,
        )
    }
    
    /// 递归查找指定位置的节点
    pub fn find_node_at_position(&self, position: MyPoint) -> Option<&LayoutResult> {
        if !self.bounds().contains_point(position) {
            return None;
        }
        
        // 先检查子节点（从后往前，因为后面的节点在上层）
        for child in self.children.iter().rev() {
            if let Some(found) = child.find_node_at_position(position) {
                return Some(found);
            }
        }
        
        // 如果没有子节点包含该位置，返回当前节点
        Some(self)
    }
    
    /// 递归遍历所有节点
    pub fn traverse<F>(&self, mut callback: F) 
    where 
        F: FnMut(&LayoutResult),
    {
        callback(self);
        for child in &self.children {
            child.traverse(&mut callback);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_layout_engine_creation() {
        let engine = LayoutEngine::new();
        assert_eq!(engine.node_map.len(), 0);
    }
    
    #[test]
    fn test_simple_layout() {
        let mut engine = LayoutEngine::new();
        
        let root = LayoutNode::Container {
            style: ContainerStyle {
                width: points(200.0),
                height: points(100.0),
                ..Default::default()
            },
            children: vec![
                LayoutNode::Text {
                    content: "Hello".to_string(),
                    style: TextStyle::default(),
                },
            ],
        };
        
        let result = engine.compute_layout(&root, MySize::new(400.0, 300.0));
        assert!(result.is_ok());
        
        let layout_result = result.unwrap();
        assert_eq!(layout_result.size().width, 200.0);
        assert_eq!(layout_result.size().height, 100.0);
        assert_eq!(layout_result.children.len(), 1);
    }
    
    #[test]
    fn test_text_measurement() {
        let engine = LayoutEngine::new();
        let text_style = TextStyle {
            font_size: 16.0,
            line_height: 1.2,
            ..Default::default()
        };
        
        let size = engine.measure_text("Hello World", &text_style).unwrap();
        assert!(size.width > 0.0);
        assert!(size.height > 0.0);
    }
}
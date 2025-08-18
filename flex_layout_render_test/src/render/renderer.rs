//! 渲染器实现
//!
//! 负责将布局结果渲染到画布上。

use crate::render::canvas::Canvas;
use crate::layout::{LayoutResult, LayoutNode};
use crate::types::*;
use crate::error::*;
use std::collections::HashMap;

/// 渲染器
pub struct Renderer {
    /// 字体缓存
    font_cache: HashMap<String, Vec<u8>>,
    /// 默认字体数据
    default_font: Vec<u8>,
}

impl Renderer {
    /// 创建新的渲染器
    pub fn new() -> Result<Self> {
        // 加载默认字体（这里使用一个简单的实现）
        let default_font = Self::load_default_font()?;
        
        Ok(Self {
            font_cache: HashMap::new(),
            default_font,
        })
    }
    
    /// 渲染布局结果到画布
    pub fn render(
        &mut self,
        layout_result: &LayoutResult,
        canvas: &mut Canvas,
    ) -> Result<()> {
        self.render_node(layout_result, canvas, Point::new(0.0, 0.0))
    }
    
    /// 递归渲染节点
    fn render_node(
        &mut self,
        layout_result: &LayoutResult,
        canvas: &mut Canvas,
        parent_offset: Point,
    ) -> Result<()> {
        let absolute_position = Point::new(
            parent_offset.x + layout_result.layout.location.x,
            parent_offset.y + layout_result.layout.location.y,
        );
        
        let size = Size::new(
            layout_result.layout.size.width,
            layout_result.layout.size.height,
        );
        
        let bounds = Rect::new(
            absolute_position.x,
            absolute_position.y,
            size.width,
            size.height,
        );
        
        // 根据节点类型进行渲染
        match &layout_result.node {
            LayoutNode::Container { style, .. } => {
                self.render_container(canvas, bounds, style)?;
            },
            LayoutNode::Text { content, style } => {
                self.render_text(canvas, bounds, content, style)?;
            },
            LayoutNode::Image { src, style } => {
                self.render_image(canvas, bounds, src, style)?;
            },
        }
        
        // 递归渲染子节点
        for child in &layout_result.children {
            self.render_node(child, canvas, absolute_position)?;
        }
        
        Ok(())
    }
    
    /// 渲染容器
    fn render_container(
        &self,
        canvas: &mut Canvas,
        bounds: Rect,
        style: &crate::layout::ContainerStyle,
    ) -> Result<()> {
        // 输出容器调试信息
        println!("[DEBUG] 容器渲染调试信息:");
        println!("  边界: x={}, y={}, width={}, height={}", bounds.x, bounds.y, bounds.width, bounds.height);
        println!("  背景色: {:?}", style.background);
        println!("  边框颜色: {:?}", style.border_color);
        println!("  边框宽度: {}", style.border_width);
        println!("  边框圆角: {}", style.border_radius);
        
        // 绘制背景
        if let Some(background) = style.background {
            canvas.fill_rect(bounds, background);
        }
        
        // 绘制边框
        if style.border_width > 0.0 {
            canvas.stroke_rect(bounds, style.border_color, style.border_width);
        }
        
        // 绘制容器调试边界（蓝色实心边框）- 放在最后确保可见
        let container_border_color = crate::types::Color::new(0, 0, 255, 255); // 不透明蓝色
        let border_width = 2.0;
        // 上边框
        canvas.fill_rect(crate::types::Rect::new(bounds.x, bounds.y, bounds.width, border_width), container_border_color);
        // 下边框
        canvas.fill_rect(crate::types::Rect::new(bounds.x, bounds.y + bounds.height - border_width, bounds.width, border_width), container_border_color);
        // 左边框
        canvas.fill_rect(crate::types::Rect::new(bounds.x, bounds.y, border_width, bounds.height), container_border_color);
        // 右边框
        canvas.fill_rect(crate::types::Rect::new(bounds.x + bounds.width - border_width, bounds.y, border_width, bounds.height), container_border_color);
        
        // TODO: 实现圆角边框
        if style.border_radius > 0.0 {
            // 圆角边框的实现比较复杂，这里先跳过
        }
        
        Ok(())
    }
    
    /// 渲染文本
    fn render_text(
        &mut self,
        canvas: &mut Canvas,
        bounds: Rect,
        content: &str,
        style: &crate::layout::TextStyle,
    ) -> Result<()> {
        // 输出调试信息
        println!("[DEBUG] 文本渲染调试信息:");
        println!("  内容: '{}'", content);
        println!("  边界: x={}, y={}, width={}, height={}", bounds.x, bounds.y, bounds.width, bounds.height);
        println!("  字体大小: {}", style.font_size);
        println!("  文本对齐: {:?}", style.text_align);
        println!("  字体家族: {}", style.font_family);
        println!("  颜色: {:?}", style.color);
        
        // 获取字体数据
        let font_data = self.get_font_data(&style.font_family)?.to_vec();
        
        // 使用新的对齐绘制方法
        canvas.draw_text_aligned(
            content,
            bounds,
            &font_data,
            style.font_size,
            style.color,
            style.text_align,
        )?;
        
        Ok(())
    }
    
    /// 渲染图片
    fn render_image(
        &self,
        canvas: &mut Canvas,
        bounds: Rect,
        src: &str,
        style: &crate::layout::ImageStyle,
    ) -> Result<()> {
        canvas.draw_image(src, bounds, style.object_fit)?;
        Ok(())
    }
    
    /// 获取字体数据
    fn get_font_data(&mut self, font_family: &str) -> Result<&[u8]> {
        if !self.font_cache.contains_key(font_family) {
            // 使用与布局引擎相同的字体管理器
            let font_manager = crate::resource::font_manager::get_font_manager();
            let font_data = {
                let mut manager = font_manager.lock().unwrap();
                manager.load_font(font_family)
                    .or_else(|_| manager.get_default_font())
            };
            
            match font_data {
                Ok(data) => {
                     self.font_cache.insert(font_family.to_string(), (*data).clone());
                 },
                Err(_) => {
                    // 如果FontManager也失败，使用默认字体
                    return Ok(&self.default_font);
                }
            }
        }
        
        Ok(self.font_cache.get(font_family).unwrap_or(&self.default_font))
    }
    
    /// 加载字体文件
    fn load_font(&self, font_family: &str) -> Result<Vec<u8>> {
        // 常见字体路径映射
        let font_paths = [
            format!("fonts/{}.ttf", font_family),
            format!("fonts/{}.otf", font_family),
            format!("/System/Library/Fonts/{}.ttf", font_family),
            format!("/System/Library/Fonts/{}.otf", font_family),
            format!("/Library/Fonts/{}.ttf", font_family),
            format!("/Library/Fonts/{}.otf", font_family),
        ];
        
        for path in &font_paths {
            if let Ok(data) = std::fs::read(path) {
                return Ok(data);
            }
        }
        
        // 如果找不到指定字体，尝试加载默认字体
        Self::load_default_font()
    }
    
    /// 加载默认字体
    fn load_default_font() -> Result<Vec<u8>> {
        // 尝试加载项目字体和系统默认字体
        let default_paths = [
            "fonts/DejaVuSans.ttf",
            "fonts/SourceHanSansSC-Regular.otf",
            "/System/Library/Fonts/Arial.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
            "/Library/Fonts/Arial.ttf",
            "fonts/default.ttf",
        ];
        
        for path in &default_paths {
            if let Ok(data) = std::fs::read(path) {
                return Ok(data);
            }
        }
        
        // 如果都找不到，创建一个最小的字体数据占位符
        // 在实际项目中，应该嵌入一个真实的字体文件
        Ok(vec![0; 1024]) // 占位符数据
    }
    

    
    /// 预加载字体
    pub fn preload_font(&mut self, font_family: &str) -> Result<()> {
        if !self.font_cache.contains_key(font_family) {
            let font_data = self.load_font(font_family)?;
            self.font_cache.insert(font_family.to_string(), font_data);
        }
        Ok(())
    }
    
    /// 清理字体缓存
    pub fn clear_font_cache(&mut self) {
        self.font_cache.clear();
    }
    
    /// 获取缓存的字体数量
    pub fn cached_font_count(&self) -> usize {
        self.font_cache.len()
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            font_cache: HashMap::new(),
            default_font: vec![0; 1024], // 占位符
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{ContainerStyle, TextStyle};
    
    #[test]
    fn test_renderer_creation() {
        let renderer = Renderer::new();
        assert!(renderer.is_ok());
        
        let renderer = renderer.unwrap();
        assert_eq!(renderer.cached_font_count(), 0);
        assert!(!renderer.default_font.is_empty());
    }
    

    
    #[test]
    fn test_font_cache() {
        let mut renderer = Renderer::default();
        assert_eq!(renderer.cached_font_count(), 0);
        
        // 尝试预加载一个不存在的字体（应该不会崩溃）
        let _ = renderer.preload_font("NonExistentFont");
        
        renderer.clear_font_cache();
        assert_eq!(renderer.cached_font_count(), 0);
    }
}
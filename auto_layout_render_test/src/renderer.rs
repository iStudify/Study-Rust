//! 渲染引擎实现

use crate::layout::*;
use image::{ImageBuffer, Rgba, RgbaImage, DynamicImage};
use fontdue::{Font, FontSettings};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("Font error: {0}")]
    FontError(String),
    #[error("Element not found: {0}")]
    ElementNotFound(String),
}

/// 渲染上下文
struct RenderContext {
    fonts: HashMap<String, Font>,
    images: HashMap<String, DynamicImage>,
}

impl RenderContext {
    fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            images: HashMap::new(),
        }
    }
    
    /// 加载字体
    fn load_font(&mut self, font_family: &str) -> Result<(), RenderError> {
        if self.fonts.contains_key(font_family) {
            return Ok(());
        }
        
        // 使用默认的 DejaVu Sans 字体
        let font_data = include_bytes!("../assets/fonts/DejaVuSans.ttf");
        let font = Font::from_bytes(font_data as &[u8], FontSettings::default())
            .map_err(|e| RenderError::FontError(format!("Failed to load DejaVu Sans font: {}", e)))?;
        
        self.fonts.insert(font_family.to_string(), font);
        println!("✅ 成功加载字体: {} (使用 DejaVu Sans)", font_family);
        Ok(())
    }
    
    /// 创建占位符字体数据
    fn create_placeholder_font_data(&self) -> Vec<u8> {
        // 返回空向量，跳过字体加载
        vec![]
    }
    
    /// 加载图片
    fn load_image(&mut self, source: &str) -> Result<(), RenderError> {
        if self.images.contains_key(source) {
            return Ok(());
        }
        
        let img = if source.starts_with("http") {
            // 网络图片加载（简化实现）
            return Err(RenderError::ImageError(image::ImageError::Unsupported(
                image::error::UnsupportedError::from_format_and_kind(
                    image::error::ImageFormatHint::Unknown,
                    image::error::UnsupportedErrorKind::GenericFeature("Network images not supported".to_string())
                )
            )));
        } else {
            // 本地图片加载
            image::open(source)?
        };
        
        self.images.insert(source.to_string(), img);
        Ok(())
    }
}

/// 渲染引擎
pub struct Renderer {
    context: RenderContext,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            context: RenderContext::new(),
        }
    }
    
    /// 渲染布局到图像
    pub fn render_layout(
        &mut self,
        layout: &Layout,
        computed_layout: &ComputedLayout,
    ) -> Result<RgbaImage, RenderError> {
        // 创建画布
        let canvas_width = computed_layout.canvas_size.width as u32;
        let canvas_height = computed_layout.canvas_size.height as u32;
        let mut image = ImageBuffer::new(canvas_width, canvas_height);
        
        // 填充背景色
        let bg_color = color_to_rgba(&layout.canvas.background);
        for pixel in image.pixels_mut() {
            *pixel = bg_color;
        }
        
        // 预加载资源
        self.preload_resources(&layout.elements)?;
        
        // 渲染所有元素
        self.render_elements(&layout.elements, computed_layout, &mut image)?;
        
        Ok(image)
    }
    
    /// 预加载所有需要的资源
    fn preload_resources(&mut self, elements: &[Element]) -> Result<(), RenderError> {
        for element in elements {
            match element {
                Element::Text { properties, .. } => {
                    self.context.load_font(&properties.font_family)?;
                }
                Element::Image { source, .. } => {
                    self.context.load_image(source)?;
                }
                _ => {}
            }
            
            // 递归处理子元素
            if let Some(children) = element.children() {
                self.preload_resources(children)?;
            }
        }
        Ok(())
    }
    
    /// 渲染元素列表
    fn render_elements(
        &mut self,
        elements: &[Element],
        computed_layout: &ComputedLayout,
        image: &mut RgbaImage,
    ) -> Result<(), RenderError> {
        for element in elements {
            self.render_element(element, computed_layout, image)?;
            
            // 递归渲染子元素
            if let Some(children) = element.children() {
                self.render_elements(children, computed_layout, image)?;
            }
        }
        Ok(())
    }
    
    /// 渲染单个元素
    fn render_element(
        &mut self,
        element: &Element,
        computed_layout: &ComputedLayout,
        image: &mut RgbaImage,
    ) -> Result<(), RenderError> {
        let frame = computed_layout.get_frame(element.id())
            .ok_or_else(|| RenderError::ElementNotFound(element.id().clone()))?;
        
        match element {
            Element::Text { content, properties, .. } => {
                self.render_text(content, properties, frame, image)?;
            }
            Element::Image { source, properties, .. } => {
                self.render_image(source, properties, frame, image)?;
            }
            Element::Container { properties, .. } => {
                self.render_container(properties, frame, image)?;
            }
            Element::VStack { .. } | Element::HStack { .. } | Element::ZStack { .. } => {
                // 堆叠容器本身不需要渲染，只渲染子元素
            }
            Element::Spacer { .. } => {
                // Spacer不需要渲染
            }
        }
        
        Ok(())
    }
    
    /// 渲染文本（简化版本，绘制文本框占位符）
    fn render_text(
        &mut self,
        content: &str,
        properties: &TextProperties,
        frame: &Rect,
        image: &mut RgbaImage,
    ) -> Result<(), RenderError> {
        let font = self.context.fonts.get(&properties.font_family)
            .ok_or_else(|| RenderError::FontError(format!("Font not loaded: {}", properties.font_family)))?;
        
        let scale = properties.font_size;
        let color = color_to_rgba(&properties.color);
        
        // 计算文本位置
        let mut x = frame.origin.x;
        // 简化基线计算：将文本基线设置在frame底部向上偏移一定距离
        // 这样可以确保文本在frame内正确显示
        let baseline_offset = properties.font_size * 0.2; // 字体大小的20%作为底部边距
        let y = frame.origin.y + frame.size.height - baseline_offset;
        
        // 根据对齐方式调整x位置
        match properties.alignment {
            TextAlignment::Leading => {
                // x已经是正确的
            }
            TextAlignment::Center => {
                let text_width = self.measure_text_width(content, font, scale);
                x = frame.origin.x + (frame.size.width - text_width) / 2.0;
            }
            TextAlignment::Trailing => {
                let text_width = self.measure_text_width(content, font, scale);
                x = frame.origin.x + frame.size.width - text_width;
            }
            TextAlignment::Justified => {
                // 简化实现，当作左对齐处理
            }
        }
        
        // 渲染每个字符
        for ch in content.chars() {
            let (metrics, bitmap) = font.rasterize(ch, scale);
            
            // 绘制字符位图
            if metrics.width > 0 {
                for (bitmap_y, row) in bitmap.chunks(metrics.width).enumerate() {
                for (bitmap_x, &alpha) in row.iter().enumerate() {
                    if alpha > 0 {
                        let pixel_x = (x + bitmap_x as f32 + metrics.xmin as f32) as u32;
                        // 修正基线对齐：y是基线位置，bitmap从上到下，需要正确处理垂直偏移
                        let pixel_y = (y + bitmap_y as f32 + metrics.ymin as f32) as u32;
                        
                        if pixel_x < image.width() && pixel_y < image.height() {
                            let existing_pixel = image.get_pixel(pixel_x, pixel_y);
                            let blended = blend_colors(*existing_pixel, color, alpha);
                            image.put_pixel(pixel_x, pixel_y, blended);
                        }
                    }
                 }
             }
             }
             
             x += metrics.advance_width;
         }
        
        println!("📝 渲染文本: '{}' 在位置 ({}, {}) 尺寸 {}x{}", 
                content, frame.origin.x, frame.origin.y, frame.size.width, frame.size.height);
        
        Ok(())
    }
    
    /// 渲染图片
    fn render_image(
        &mut self,
        source: &str,
        properties: &ImageProperties,
        frame: &Rect,
        image: &mut RgbaImage,
    ) -> Result<(), RenderError> {
        let src_image = self.context.images.get(source)
            .ok_or_else(|| RenderError::ImageError(image::ImageError::Unsupported(
                image::error::UnsupportedError::from_format_and_kind(
                    image::error::ImageFormatHint::Unknown,
                    image::error::UnsupportedErrorKind::GenericFeature(format!("Image not loaded: {}", source))
                )
            )))?;
        
        // 转换为RGBA格式
        let src_rgba = src_image.to_rgba8();
        
        // 计算缩放后的尺寸
        let (scaled_width, scaled_height) = self.calculate_scaled_size(
            src_rgba.width(),
            src_rgba.height(),
            frame.size.width as u32,
            frame.size.height as u32,
            properties.scale_mode,
        );
        
        // 缩放图片
        let scaled_image = image::imageops::resize(
            &src_rgba,
            scaled_width,
            scaled_height,
            image::imageops::FilterType::Lanczos3,
        );
        
        // 计算绘制位置（居中）
        let draw_x = frame.origin.x + (frame.size.width - scaled_width as f32) / 2.0;
        let draw_y = frame.origin.y + (frame.size.height - scaled_height as f32) / 2.0;
        
        // 绘制图片
        for (src_x, src_y, src_pixel) in scaled_image.enumerate_pixels() {
            let dst_x = (draw_x + src_x as f32) as u32;
            let dst_y = (draw_y + src_y as f32) as u32;
            
            if dst_x < image.width() && dst_y < image.height() {
                let mut pixel = *src_pixel;
                
                // 应用透明度
                if properties.opacity < 1.0 {
                    pixel[3] = (pixel[3] as f32 * properties.opacity) as u8;
                }
                
                // 应用着色
                if let Some(tint) = &properties.tint_color {
                    pixel[0] = ((pixel[0] as f32 * tint.r as f32) / 255.0) as u8;
                    pixel[1] = ((pixel[1] as f32 * tint.g as f32) / 255.0) as u8;
                    pixel[2] = ((pixel[2] as f32 * tint.b as f32) / 255.0) as u8;
                }
                
                // Alpha混合
                let existing_pixel = image.get_pixel(dst_x, dst_y);
                let blended = alpha_blend(*existing_pixel, pixel);
                image.put_pixel(dst_x, dst_y, blended);
            }
        }
        
        Ok(())
    }
    
    /// 渲染容器
    fn render_container(
        &mut self,
        properties: &ContainerProperties,
        frame: &Rect,
        image: &mut RgbaImage,
    ) -> Result<(), RenderError> {
        // 绘制背景
        if properties.background.a > 0 {
            let bg_color = color_to_rgba(&properties.background);
            self.fill_rect(image, frame, bg_color);
        }
        
        // 绘制边框
        if properties.border_width > 0.0 {
            let border_color = color_to_rgba(&properties.border_color);
            self.draw_border(image, frame, properties.border_width, border_color);
        }
        
        Ok(())
    }
    
    /// 填充矩形
    fn fill_rect(&self, image: &mut RgbaImage, rect: &Rect, color: Rgba<u8>) {
        let x1 = rect.origin.x as u32;
        let y1 = rect.origin.y as u32;
        let x2 = (rect.origin.x + rect.size.width) as u32;
        let y2 = (rect.origin.y + rect.size.height) as u32;
        
        for y in y1..y2.min(image.height()) {
            for x in x1..x2.min(image.width()) {
                image.put_pixel(x, y, color);
            }
        }
    }
    
    /// 绘制边框
    fn draw_border(&self, image: &mut RgbaImage, rect: &Rect, width: f32, color: Rgba<u8>) {
        let border_width = width as u32;
        let x1 = rect.origin.x as u32;
        let y1 = rect.origin.y as u32;
        let x2 = (rect.origin.x + rect.size.width) as u32;
        let y2 = (rect.origin.y + rect.size.height) as u32;
        
        // 上边框
        for y in y1..y1.saturating_add(border_width).min(image.height()) {
            for x in x1..x2.min(image.width()) {
                image.put_pixel(x, y, color);
            }
        }
        
        // 下边框
        for y in y2.saturating_sub(border_width)..y2.min(image.height()) {
            for x in x1..x2.min(image.width()) {
                image.put_pixel(x, y, color);
            }
        }
        
        // 左边框
        for x in x1..x1.saturating_add(border_width).min(image.width()) {
            for y in y1..y2.min(image.height()) {
                image.put_pixel(x, y, color);
            }
        }
        
        // 右边框
        for x in x2.saturating_sub(border_width)..x2.min(image.width()) {
            for y in y1..y2.min(image.height()) {
                image.put_pixel(x, y, color);
            }
        }
    }
    
    /// 测量文本宽度
    fn measure_text_width(&self, text: &str, font: &Font, scale: f32) -> f32 {
        let mut width = 0.0;
        for ch in text.chars() {
            let metrics = font.metrics(ch, scale);
            width += metrics.advance_width;
        }
        width
    }
    
    /// 计算缩放后的尺寸
    fn calculate_scaled_size(
        &self,
        src_width: u32,
        src_height: u32,
        dst_width: u32,
        dst_height: u32,
        scale_mode: ScaleMode,
    ) -> (u32, u32) {
        match scale_mode {
            ScaleMode::Fit => {
                let scale_x = dst_width as f32 / src_width as f32;
                let scale_y = dst_height as f32 / src_height as f32;
                let scale = scale_x.min(scale_y);
                
                ((src_width as f32 * scale) as u32, (src_height as f32 * scale) as u32)
            }
            ScaleMode::Fill => {
                let scale_x = dst_width as f32 / src_width as f32;
                let scale_y = dst_height as f32 / src_height as f32;
                let scale = scale_x.max(scale_y);
                
                ((src_width as f32 * scale) as u32, (src_height as f32 * scale) as u32)
            }
            ScaleMode::Stretch => {
                (dst_width, dst_height)
            }
            ScaleMode::Center => {
                (src_width, src_height)
            }
        }
    }
}

/// 将Color转换为Rgba<u8>
fn color_to_rgba(color: &Color) -> Rgba<u8> {
    Rgba([color.r, color.g, color.b, color.a])
}

/// Alpha混合两个颜色
fn alpha_blend(background: Rgba<u8>, foreground: Rgba<u8>) -> Rgba<u8> {
    let bg = background.0;
    let fg = foreground.0;
    
    let alpha_fg = fg[3] as f32 / 255.0;
    let alpha_bg = bg[3] as f32 / 255.0;
    let alpha_out = alpha_fg + alpha_bg * (1.0 - alpha_fg);
    
    if alpha_out == 0.0 {
        return Rgba([0, 0, 0, 0]);
    }
    
    let r = ((fg[0] as f32 * alpha_fg + bg[0] as f32 * alpha_bg * (1.0 - alpha_fg)) / alpha_out) as u8;
    let g = ((fg[1] as f32 * alpha_fg + bg[1] as f32 * alpha_bg * (1.0 - alpha_fg)) / alpha_out) as u8;
    let b = ((fg[2] as f32 * alpha_fg + bg[2] as f32 * alpha_bg * (1.0 - alpha_fg)) / alpha_out) as u8;
    let a = (alpha_out * 255.0) as u8;
    
    Rgba([r, g, b, a])
}

/// 颜色混合（用于文本渲染）
fn blend_colors(background: Rgba<u8>, foreground: Rgba<u8>, alpha: u8) -> Rgba<u8> {
    let alpha_f = alpha as f32 / 255.0;
    let inv_alpha = 1.0 - alpha_f;
    
    let r = (foreground.0[0] as f32 * alpha_f + background.0[0] as f32 * inv_alpha) as u8;
    let g = (foreground.0[1] as f32 * alpha_f + background.0[1] as f32 * inv_alpha) as u8;
    let b = (foreground.0[2] as f32 * alpha_f + background.0[2] as f32 * inv_alpha) as u8;
    let a = (foreground.0[3] as f32 * alpha_f + background.0[3] as f32 * inv_alpha) as u8;
    
    Rgba([r, g, b, a])
}
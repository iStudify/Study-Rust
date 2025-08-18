//! 渲染画布实现
//!
//! 提供基于 image 库的画布渲染功能。

use crate::error::*;
use crate::types::*;
use image::{ImageBuffer, Rgba, RgbaImage};
// use ab_glyph::{FontRef, PxScale, point}; // 暂时未使用
use imageproc::drawing::{draw_filled_rect_mut, draw_hollow_rect_mut};
use imageproc::rect::Rect as ImageRect;
use rusttype::{Font, Scale};
use std::path::Path;

/// 渲染画布
pub struct Canvas {
    /// 图像缓冲区
    image: RgbaImage,
    /// 画布尺寸
    size: Size,
    /// DPI 设置
    dpi: f32,
}

impl Canvas {
    /// 创建新的画布
    pub fn new(size: Size, background: Color, dpi: f32) -> Self {
        // 对于标准显示，直接使用指定的尺寸
        // DPI缩放应该只在高DPI显示时应用
        let width = if dpi <= 1.0 {
            size.width as u32
        } else {
            (size.width * dpi / 72.0) as u32
        };
        let height = if dpi <= 1.0 {
            size.height as u32
        } else {
            (size.height * dpi / 72.0) as u32
        };

        let mut image = ImageBuffer::new(width, height);

        // 填充背景色
        let bg_color = Rgba([background.r, background.g, background.b, background.a]);

        for pixel in image.pixels_mut() {
            *pixel = bg_color;
        }

        Self { image, size, dpi }
    }

    /// 获取画布尺寸
    pub fn size(&self) -> Size {
        self.size
    }

    /// 获取 DPI
    pub fn dpi(&self) -> f32 {
        self.dpi
    }

    /// 将逻辑坐标转换为像素坐标
    fn to_pixel_coords(&self, point: Point) -> (u32, u32) {
        // 对于标准显示（DPI <= 1.0），直接使用逻辑坐标
        // 对于高DPI显示，进行缩放
        if self.dpi <= 1.0 {
            (point.x as u32, point.y as u32)
        } else {
            let scale = self.dpi / 72.0;
            ((point.x * scale) as u32, (point.y * scale) as u32)
        }
    }

    /// 将逻辑尺寸转换为像素尺寸
    fn to_pixel_size(&self, size: Size) -> (u32, u32) {
        // 对于标准显示（DPI <= 1.0），直接使用逻辑尺寸
        // 对于高DPI显示，进行缩放
        if self.dpi <= 1.0 {
            (size.width as u32, size.height as u32)
        } else {
            let scale = self.dpi / 72.0;
            ((size.width * scale) as u32, (size.height * scale) as u32)
        }
    }

    /// 绘制填充矩形
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let (x, y) = self.to_pixel_coords(Point::new(rect.x, rect.y));
        let (width, height) = self.to_pixel_size(Size::new(rect.width, rect.height));

        let rgba = Rgba([color.r, color.g, color.b, color.a]);

        let image_rect = ImageRect::at(x as i32, y as i32).of_size(width, height);
        draw_filled_rect_mut(&mut self.image, image_rect, rgba);
    }

    /// 绘制矩形边框
    pub fn stroke_rect(&mut self, rect: Rect, color: Color, width: f32) {
        if width <= 0.0 {
            return;
        }

        let (x, y) = self.to_pixel_coords(Point::new(rect.x, rect.y));
        let (rect_width, rect_height) = self.to_pixel_size(Size::new(rect.width, rect.height));

        let rgba = Rgba([color.r, color.g, color.b, color.a]);

        let stroke_width = (width * self.dpi / 72.0) as u32;

        // 绘制多层边框来模拟边框宽度
        for i in 0..stroke_width {
            let image_rect = ImageRect::at((x + i) as i32, (y + i) as i32).of_size(
                rect_width.saturating_sub(i * 2),
                rect_height.saturating_sub(i * 2),
            );

            if image_rect.width() > 0 && image_rect.height() > 0 {
                draw_hollow_rect_mut(&mut self.image, image_rect, rgba);
            }
        }
    }

    /// 绘制文本
    pub fn draw_text(
        &mut self,
        text: &str,
        position: Point,
        font_data: &[u8],
        font_size: f32,
        color: Color,
    ) -> Result<()> {
        let font = Font::try_from_bytes(font_data)
            .ok_or_else(|| FlexRenderError::render_error("字体加载失败".to_string()))?;

        // 对于标准显示（DPI <= 1），直接使用字体大小
        // 对于高DPI显示，才进行DPI缩放
        let pixel_font_size = if self.dpi <= 1.0 {
            font_size
        } else {
            font_size * self.dpi / 72.0
        };
        let scale = Scale::uniform(pixel_font_size);
        let (x, y) = self.to_pixel_coords(position);

        let rgba = Rgba([color.r, color.g, color.b, color.a]);

        // 使用 rusttype 和 imageproc 绘制文本
        imageproc::drawing::draw_text_mut(
            &mut self.image,
            rgba,
            x as i32,
            y as i32,
            scale,
            &font,
            text,
        );
        Ok(())
    }

    /// 测试用：直接绘制文本到指定位置
    pub fn draw_text_direct(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        font_data: &[u8],
        font_size: f32,
        color: Color,
    ) -> Result<()> {
        let font = Font::try_from_bytes(font_data)
            .ok_or_else(|| FlexRenderError::render_error("字体加载失败".to_string()))?;

        let pixel_font_size = if self.dpi <= 1.0 {
            font_size
        } else {
            font_size * self.dpi / 72.0
        };
        let scale = Scale::uniform(pixel_font_size);

        let (pixel_x, pixel_y) = self.to_pixel_coords(Point::new(x, y));

        let rgba = Rgba([color.r, color.g, color.b, color.a]);

        println!("[DEBUG] 直接绘制文本:");
        println!("  文本: '{}'", text);
        println!("  逻辑位置: x={}, y={}", x, y);
        println!("  像素位置: x={}, y={}", pixel_x, pixel_y);
        println!(
            "  字体大小: {} -> 像素字体大小: {}",
            font_size, pixel_font_size
        );

        // 使用 rusttype 和 imageproc 绘制文本
        imageproc::drawing::draw_text_mut(
            &mut self.image,
            rgba,
            pixel_x as i32,
            pixel_y as i32,
            scale,
            &font,
            text,
        );
        Ok(())
    }

    /// 绘制带对齐的文本
    pub fn draw_text_aligned(
        &mut self,
        text: &str,
        bounds: Rect,
        font_data: &[u8],
        font_size: f32,
        color: Color,
        text_align: TextAlign,
    ) -> Result<()> {
        let font = Font::try_from_bytes(font_data)
            .ok_or_else(|| FlexRenderError::render_error("字体加载失败".to_string()))?;

        let pixel_font_size = if self.dpi <= 1.0 {
            font_size
        } else {
            font_size * self.dpi / 72.0
        };
        let scale = Scale::uniform(pixel_font_size);

        // 使用更准确的方法计算文本宽度
        let glyphs: Vec<_> = font
            .layout(text, scale, rusttype::point(0.0, 0.0))
            .collect();

        // 计算文本的实际边界框
        let text_width_pixels = if glyphs.is_empty() {
            0.0
        } else {
            // 找到最右边的字符位置
            let last_glyph = glyphs.last().unwrap();
            let last_x = last_glyph.position().x;
            let last_advance = last_glyph.unpositioned().h_metrics().advance_width;
            last_x + last_advance
        };

        // 将像素宽度转换为逻辑单位
        let text_width_logical = if self.dpi <= 1.0 {
            text_width_pixels
        } else {
            text_width_pixels * 72.0 / self.dpi
        };

        // 根据对齐方式计算x坐标
        let x = match text_align {
            TextAlign::Left => bounds.x,
            TextAlign::Center => bounds.x + (bounds.width - text_width_logical) / 2.0,
            TextAlign::Right => bounds.x + bounds.width - text_width_logical,
            TextAlign::Justify => bounds.x, // Justify按左对齐处理
        };

        // 基线位置 - 简单的垂直居中，确保在边界框内，暂不实现
        let y = bounds.y;

        let (pixel_x, pixel_y) = self.to_pixel_coords(Point::new(x, y));

        let rgba = Rgba([color.r, color.g, color.b, color.a]);

        println!("[DEBUG] Canvas绘制文本:");
        println!("  文本: '{}'", text);
        println!(
            "  边界: x={}, y={}, width={}, height={}",
            bounds.x, bounds.y, bounds.width, bounds.height
        );
        println!(
            "  计算出的文本宽度: {} (像素: {})",
            text_width_logical, text_width_pixels
        );
        println!("  对齐方式: {:?}", text_align);
        println!("  最终绘制位置: x={}, y={}", x, y);
        println!("  像素坐标: x={}, y={}", pixel_x, pixel_y);
        println!(
            "  字体大小: {} -> 像素字体大小: {}",
            font_size, pixel_font_size
        );
        println!("  DPI: {}", self.dpi);

        // 使用 rusttype 和 imageproc 绘制文本
        imageproc::drawing::draw_text_mut(
            &mut self.image,
            rgba,
            pixel_x as i32,
            pixel_y as i32,
            scale,
            &font,
            text,
        );

        // 绘制文本区域调试边界（红色实心边框）- 放在最后确保可见
        let border_color = Color::new(255, 0, 0, 255); // 不透明红色
        let border_width = 2.0;
        // 上边框
        self.fill_rect(
            Rect::new(bounds.x, bounds.y, bounds.width, border_width),
            border_color,
        );
        // 下边框
        self.fill_rect(
            Rect::new(
                bounds.x,
                bounds.y + bounds.height - border_width,
                bounds.width,
                border_width,
            ),
            border_color,
        );
        // 左边框
        self.fill_rect(
            Rect::new(bounds.x, bounds.y, border_width, bounds.height),
            border_color,
        );
        // 右边框
        self.fill_rect(
            Rect::new(
                bounds.x + bounds.width - border_width,
                bounds.y,
                border_width,
                bounds.height,
            ),
            border_color,
        );

        Ok(())
    }

    /// 绘制图片
    pub fn draw_image(
        &mut self,
        image_path: &str,
        dest_rect: Rect,
        object_fit: ObjectFit,
    ) -> Result<()> {
        // 加载图片
        let source_image = image::open(image_path)
            .map_err(|e| FlexRenderError::render_error(format!("图片加载失败: {:?}", e)))?;

        let source_rgba = source_image.to_rgba8();
        let (src_width, src_height) = source_rgba.dimensions();

        // 计算目标区域
        let (dest_x, dest_y) = self.to_pixel_coords(Point::new(dest_rect.x, dest_rect.y));
        let (dest_width, dest_height) =
            self.to_pixel_size(Size::new(dest_rect.width, dest_rect.height));

        // 根据 object_fit 计算实际绘制区域和源区域
        let (draw_rect, _src_rect) = self.calculate_image_rects(
            Size::new(src_width as f32, src_height as f32),
            Rect::new(
                dest_x as f32,
                dest_y as f32,
                dest_width as f32,
                dest_height as f32,
            ),
            object_fit,
        );

        // 缩放并绘制图片
        let resized_image = image::imageops::resize(
            &source_rgba,
            draw_rect.width as u32,
            draw_rect.height as u32,
            image::imageops::FilterType::Lanczos3,
        );

        // 将缩放后的图片绘制到画布上
        self.blend_image(&resized_image, Point::new(draw_rect.x, draw_rect.y));

        Ok(())
    }

    /// 计算图片绘制区域
    fn calculate_image_rects(
        &self,
        src_size: Size,
        dest_rect: Rect,
        object_fit: ObjectFit,
    ) -> (Rect, Rect) {
        match object_fit {
            ObjectFit::Fill => {
                // 拉伸填充整个区域
                (
                    dest_rect,
                    Rect::new(0.0, 0.0, src_size.width, src_size.height),
                )
            }
            ObjectFit::Contain => {
                // 保持比例，完全显示在区域内
                let scale =
                    (dest_rect.width / src_size.width).min(dest_rect.height / src_size.height);

                let scaled_width = src_size.width * scale;
                let scaled_height = src_size.height * scale;

                let x = dest_rect.x + (dest_rect.width - scaled_width) / 2.0;
                let y = dest_rect.y + (dest_rect.height - scaled_height) / 2.0;

                (
                    Rect::new(x, y, scaled_width, scaled_height),
                    Rect::new(0.0, 0.0, src_size.width, src_size.height),
                )
            }
            ObjectFit::Cover => {
                // 保持比例，覆盖整个区域
                let scale =
                    (dest_rect.width / src_size.width).max(dest_rect.height / src_size.height);

                let scaled_width = src_size.width * scale;
                let scaled_height = src_size.height * scale;

                (
                    dest_rect,
                    Rect::new(
                        (scaled_width - dest_rect.width) / 2.0 / scale,
                        (scaled_height - dest_rect.height) / 2.0 / scale,
                        dest_rect.width / scale,
                        dest_rect.height / scale,
                    ),
                )
            }
            ObjectFit::ScaleDown => {
                // 如果图片比容器大，则按 contain 处理，否则保持原尺寸
                if src_size.width > dest_rect.width || src_size.height > dest_rect.height {
                    self.calculate_image_rects(src_size, dest_rect, ObjectFit::Contain)
                } else {
                    let x = dest_rect.x + (dest_rect.width - src_size.width) / 2.0;
                    let y = dest_rect.y + (dest_rect.height - src_size.height) / 2.0;

                    (
                        Rect::new(x, y, src_size.width, src_size.height),
                        Rect::new(0.0, 0.0, src_size.width, src_size.height),
                    )
                }
            }
            ObjectFit::None => {
                // 保持原尺寸，居中显示
                let x = dest_rect.x + (dest_rect.width - src_size.width) / 2.0;
                let y = dest_rect.y + (dest_rect.height - src_size.height) / 2.0;

                (
                    Rect::new(x, y, src_size.width, src_size.height),
                    Rect::new(0.0, 0.0, src_size.width, src_size.height),
                )
            }
        }
    }

    /// 混合图片到画布
    fn blend_image(&mut self, source: &RgbaImage, position: Point) {
        let (start_x, start_y) = self.to_pixel_coords(position);
        let (canvas_width, canvas_height) = self.image.dimensions();

        for (x, y, pixel) in source.enumerate_pixels() {
            let dest_x = start_x + x;
            let dest_y = start_y + y;

            if dest_x < canvas_width && dest_y < canvas_height {
                // 简单的 alpha 混合
                let dest_pixel = self.image.get_pixel_mut(dest_x, dest_y);
                let alpha = pixel[3] as f32 / 255.0;
                let inv_alpha = 1.0 - alpha;

                dest_pixel[0] =
                    ((pixel[0] as f32 * alpha) + (dest_pixel[0] as f32 * inv_alpha)) as u8;
                dest_pixel[1] =
                    ((pixel[1] as f32 * alpha) + (dest_pixel[1] as f32 * inv_alpha)) as u8;
                dest_pixel[2] =
                    ((pixel[2] as f32 * alpha) + (dest_pixel[2] as f32 * inv_alpha)) as u8;
                dest_pixel[3] = ((pixel[3] as f32) + (dest_pixel[3] as f32 * inv_alpha)) as u8;
            }
        }
    }

    /// 保存画布到文件
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.image
            .save(path)
            .map_err(|e| FlexRenderError::render_error(format!("保存图片失败: {:?}", e)))?;
        Ok(())
    }

    /// 获取画布图像数据
    pub fn to_image(&self) -> &RgbaImage {
        &self.image
    }

    /// 获取画布图像的克隆
    pub fn to_image_clone(&self) -> RgbaImage {
        self.image.clone()
    }

    /// 获取画布图像数据的可变引用
    pub fn to_image_mut(&mut self) -> &mut RgbaImage {
        &mut self.image
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_creation() {
        let canvas = Canvas::new(Size::new(100.0, 100.0), Color::white(), 72.0);

        assert_eq!(canvas.size().width, 100.0);
        assert_eq!(canvas.size().height, 100.0);
        assert_eq!(canvas.dpi(), 72.0);
    }

    #[test]
    fn test_pixel_conversion() {
        let canvas = Canvas::new(
            Size::new(100.0, 100.0),
            Color::white(),
            144.0, // 2x DPI
        );

        let (x, y) = canvas.to_pixel_coords(Point::new(10.0, 20.0));
        assert_eq!(x, 20); // 10 * 144 / 72 = 20
        assert_eq!(y, 40); // 20 * 144 / 72 = 40
    }

    #[test]
    fn test_fill_rect() {
        let mut canvas = Canvas::new(Size::new(100.0, 100.0), Color::white(), 72.0);

        canvas.fill_rect(Rect::new(10.0, 10.0, 50.0, 30.0), Color::red());

        // 验证矩形区域的颜色
        let pixel = canvas.image.get_pixel(20, 20);
        assert_eq!(pixel[0], 255); // 红色
        assert_eq!(pixel[1], 0); // 绿色
        assert_eq!(pixel[2], 0); // 蓝色
    }
}

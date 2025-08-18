//! 基础类型定义模块
//!
//! 包含了库中使用的所有基础数据类型和枚举。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RGBA 颜色类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// 创建新的颜色
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    
    /// 从浮点数创建颜色 (0.0-1.0 范围)
    pub fn from_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: (r.clamp(0.0, 1.0) * 255.0).round() as u8,
            g: (g.clamp(0.0, 1.0) * 255.0).round() as u8,
            b: (b.clamp(0.0, 1.0) * 255.0).round() as u8,
            a: (a.clamp(0.0, 1.0) * 255.0).round() as u8,
        }
    }
    
    /// 从十六进制字符串创建颜色
    pub fn from_hex(hex: &str) -> crate::Result<Self> {
        let color = csscolorparser::parse(hex)
            .map_err(|e| crate::error::FlexRenderError::parse_error(
                format!("无效的颜色值: {}", e),
                0,
                0,
            ))?;
        
        Ok(Self::from_f32(color.r as f32, color.g as f32, color.b as f32, color.a as f32))
    }
    
    /// 转换为浮点数格式 (0.0-1.0 范围)
    pub fn to_f32(&self) -> (f32, f32, f32, f32) {
        (
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        )
    }
    
    /// 透明颜色
    pub fn transparent() -> Self {
        Self::new(0, 0, 0, 0)
    }
    
    /// 白色
    pub fn white() -> Self {
        Self::new(255, 255, 255, 255)
    }
    
    /// 黑色
    pub fn black() -> Self {
        Self::new(0, 0, 0, 255)
    }
    
    /// 红色
    pub fn red() -> Self {
        Self::new(255, 0, 0, 255)
    }
    
    /// 绿色
    pub fn green() -> Self {
        Self::new(0, 255, 0, 255)
    }
    
    /// 蓝色
    pub fn blue() -> Self {
        Self::new(0, 0, 255, 255)
    }
}

/// 2D 点坐标
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }
    
    /// 计算到另一个点的距离
    pub fn distance_to(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
    
    /// 点的平移
    pub fn translate(&self, dx: f32, dy: f32) -> Point {
        Point::new(self.x + dx, self.y + dy)
    }
}

/// 2D 尺寸
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width: width.max(0.0),
            height: height.max(0.0),
        }
    }
    
    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }
    
    pub fn area(&self) -> f32 {
        self.width * self.height
    }
    
    /// 检查是否为空（宽度或高度为0）
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }
    
    /// 按比例缩放
    pub fn scale(&self, factor: f32) -> Size {
        Size::new(self.width * factor, self.height * factor)
    }
    
    /// 等比例缩放以适应指定尺寸
    pub fn fit_within(&self, container: &Size) -> Size {
        if self.is_empty() || container.is_empty() {
            return Size::zero();
        }

        let scale_x = container.width / self.width;
        let scale_y = container.height / self.height;
        let scale = scale_x.min(scale_y);
        
        self.scale(scale)
    }
}

/// 矩形区域
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }
    
    pub fn from_size(size: Size) -> Self {
        Self::new(0.0, 0.0, size.width, size.height)
    }
    
    pub fn from_point_size(point: Point, size: Size) -> Self {
        Self::new(point.x, point.y, size.width, size.height)
    }
    
    /// 获取矩形的各个边界
    pub fn left(&self) -> f32 { self.x }
    pub fn top(&self) -> f32 { self.y }
    pub fn right(&self) -> f32 { self.x + self.width }
    pub fn bottom(&self) -> f32 { self.y + self.height }
    
    /// 获取矩形的尺寸
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
    
    /// 获取矩形的原点
    pub fn origin(&self) -> Point {
        Point::new(self.x, self.y)
    }
    
    pub fn contains_point(&self, point: Point) -> bool {
        point.x >= self.x && point.x <= self.x + self.width &&
        point.y >= self.y && point.y <= self.y + self.height
    }
    
    pub fn center(&self) -> Point {
        Point::new(
            self.x + self.width / 2.0,
            self.y + self.height / 2.0,
        )
    }
    
    /// 检查是否与另一个矩形相交
    pub fn intersects(&self, other: &Rect) -> bool {
        self.left() < other.right() && self.right() > other.left() &&
        self.top() < other.bottom() && self.bottom() > other.top()
    }
    
    /// 计算与另一个矩形的交集
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        if !self.intersects(other) {
            return None;
        }

        let left = self.left().max(other.left());
        let top = self.top().max(other.top());
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        Some(Rect::new(left, top, right - left, bottom - top))
    }
    
    /// 计算包含两个矩形的最小矩形
    pub fn union(&self, other: &Rect) -> Rect {
        let left = self.left().min(other.left());
        let top = self.top().min(other.top());
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());

        Rect::new(left, top, right - left, bottom - top)
    }
    
    /// 矩形平移
    pub fn translate(&self, dx: f32, dy: f32) -> Rect {
        Rect::new(self.x + dx, self.y + dy, self.width, self.height)
    }
    
    /// 检查矩形是否为空
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }
}

/// 模板变量类型
pub type TemplateVariables = HashMap<String, serde_json::Value>;

/// 尺寸单位
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Unit {
    /// 像素值
    Px(f32),
    /// 百分比值
    Percent(f32),
    /// 自动
    Auto,
}

impl Unit {
    /// 解析为像素值
    pub fn to_pixels(&self, container_size: f32) -> f32 {
        match self {
            Unit::Px(px) => *px,
            Unit::Percent(percent) => container_size * percent / 100.0,
            Unit::Auto => 0.0, // 自动值需要在布局计算中处理
        }
    }
}

/// 字体粗细
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FontWeight {
    /// 正常粗细
    Normal,
    /// 粗体
    Bold,
    /// 自定义粗细值 (100-900)
    Weight(u16),
}

impl FontWeight {
    /// 转换为数值
    pub fn to_number(&self) -> u16 {
        match self {
            FontWeight::Normal => 400,
            FontWeight::Bold => 700,
            FontWeight::Weight(weight) => *weight,
        }
    }
}

/// 文本对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TextAlign {
    /// 左对齐
    Left,
    /// 居中对齐
    Center,
    /// 右对齐
    Right,
    /// 两端对齐
    Justify,
}

/// 图片适应方式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ObjectFit {
    /// 填充整个容器，可能会拉伸
    Fill,
    /// 保持宽高比，完全显示在容器内
    Contain,
    /// 保持宽高比，覆盖整个容器
    Cover,
    /// 保持宽高比，缩小到容器内
    ScaleDown,
    /// 不缩放
    None,
}

/// 边框样式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BorderStyle {
    /// 实线
    Solid,
    /// 虚线
    Dashed,
    /// 点线
    Dotted,
    /// 无边框
    None,
}

/// 渐变类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Gradient {
    /// 线性渐变
    Linear {
        /// 角度（度）
        angle: f32,
        /// 颜色停止点
        stops: Vec<ColorStop>,
    },
    /// 径向渐变
    Radial {
        /// 中心点
        center: Point,
        /// 半径
        radius: f32,
        /// 颜色停止点
        stops: Vec<ColorStop>,
    },
}

/// 渐变色停止点
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorStop {
    /// 位置 (0.0 到 1.0)
    pub position: f32,
    /// 颜色
    pub color: Color,
}

/// 边距/内边距
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    /// 创建新的边距
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }

    /// 创建统一边距
    pub fn all(value: f32) -> Self {
        Self::new(value, value, value, value)
    }

    /// 创建对称边距
    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self::new(vertical, horizontal, vertical, horizontal)
    }

    /// 零边距
    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    /// 计算水平边距总和
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// 计算垂直边距总和
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }

    /// 应用边距到矩形（缩小矩形）
    pub fn apply_to_rect(&self, rect: &Rect) -> Rect {
        let new_x = rect.x + self.left;
        let new_y = rect.y + self.top;
        let new_width = (rect.width - self.horizontal()).max(0.0);
        let new_height = (rect.height - self.vertical()).max(0.0);
        
        Rect::new(new_x, new_y, new_width, new_height)
    }

    /// 扩展矩形（添加边距）
    pub fn expand_rect(&self, rect: &Rect) -> Rect {
        let new_x = rect.x - self.left;
        let new_y = rect.y - self.top;
        let new_width = rect.width + self.horizontal();
        let new_height = rect.height + self.vertical();
        
        Rect::new(new_x, new_y, new_width, new_height)
    }
}

impl Default for EdgeInsets {
    fn default() -> Self {
        Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_color_creation() {
        let color = Color::new(255, 128, 64, 255);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
        assert_eq!(color.a, 255);
    }
    
    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("#ff8040").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
        assert_eq!(color.a, 255);
    }
    
    #[test]
    fn test_rect_contains_point() {
        let rect = Rect::new(10.0, 10.0, 100.0, 50.0);
        let point_inside = Point::new(50.0, 30.0);
        let point_outside = Point::new(5.0, 5.0);
        
        assert!(rect.contains_point(point_inside));
        assert!(!rect.contains_point(point_outside));
    }
    
    #[test]
    fn test_unit_to_pixels() {
        let px_unit = Unit::Px(100.0);
        let percent_unit = Unit::Percent(50.0);
        
        assert_eq!(px_unit.to_pixels(200.0), 100.0);
        assert_eq!(percent_unit.to_pixels(200.0), 100.0);
    }
    
    #[test]
    fn test_font_weight_to_number() {
        assert_eq!(FontWeight::Normal.to_number(), 400);
        assert_eq!(FontWeight::Bold.to_number(), 700);
        assert_eq!(FontWeight::Weight(500).to_number(), 500);
    }
    
    #[test]
    fn test_color_f32_conversion() {
        let color = Color::from_f32(0.5, 0.7, 0.9, 0.8);
        let (r, g, b, a) = color.to_f32();
        assert!((r - 0.5).abs() < 0.01);
        assert!((g - 0.7).abs() < 0.01);
        assert!((b - 0.9).abs() < 0.01);
        assert!((a - 0.8).abs() < 0.01);
    }
    
    #[test]
    fn test_point_operations() {
        let p1 = Point::new(1.0, 2.0);
        let p2 = Point::new(4.0, 6.0);
        
        assert_eq!(p1.distance_to(&p2), 5.0);
        assert_eq!(p1.translate(3.0, 4.0), p2);
    }
    
    #[test]
    fn test_size_operations() {
        let size = Size::new(10.0, 20.0);
        assert_eq!(size.area(), 200.0);
        assert!(!size.is_empty());
        
        let scaled = size.scale(0.5);
        assert_eq!(scaled, Size::new(5.0, 10.0));
    }
    
    #[test]
    fn test_rect_intersection() {
        let rect1 = Rect::new(0.0, 0.0, 10.0, 10.0);
        let rect2 = Rect::new(5.0, 5.0, 10.0, 10.0);
        
        assert!(rect1.intersects(&rect2));
        let intersection = rect1.intersection(&rect2).unwrap();
        assert_eq!(intersection, Rect::new(5.0, 5.0, 5.0, 5.0));
    }
    
    #[test]
    fn test_edge_insets() {
        let insets = EdgeInsets::all(5.0);
        let rect = Rect::new(0.0, 0.0, 20.0, 20.0);
        
        let inner_rect = insets.apply_to_rect(&rect);
        assert_eq!(inner_rect, Rect::new(5.0, 5.0, 10.0, 10.0));
        
        let expanded_rect = insets.expand_rect(&rect);
        assert_eq!(expanded_rect, Rect::new(-5.0, -5.0, 30.0, 30.0));
    }
}
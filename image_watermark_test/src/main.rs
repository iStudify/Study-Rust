use clap::{Arg, Command};
use image::{DynamicImage, Rgba};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};

#[derive(Debug)]
pub enum WatermarkPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
    Custom(u32, u32),
}

impl WatermarkPosition {
    fn calculate_position(
        &self,
        img_width: u32,
        img_height: u32,
        text_width: u32,
        text_height: u32,
    ) -> (u32, u32) {
        match self {
            WatermarkPosition::TopLeft => (10, 10),
            WatermarkPosition::TopRight => (img_width.saturating_sub(text_width + 10), 10),
            WatermarkPosition::BottomLeft => (10, img_height.saturating_sub(text_height + 10)),
            WatermarkPosition::BottomRight => (
                img_width.saturating_sub(text_width + 10),
                img_height.saturating_sub(text_height + 10),
            ),
            WatermarkPosition::Center => (
                (img_width.saturating_sub(text_width)) / 2,
                (img_height.saturating_sub(text_height)) / 2,
            ),
            WatermarkPosition::Custom(x, y) => (*x, *y),
        }
    }
}

pub struct WatermarkConfig {
    pub text: String,
    pub position: WatermarkPosition,
    pub font_size: f32,
    pub color: Rgba<u8>,
    pub opacity: f32,
}

impl Default for WatermarkConfig {
    fn default() -> Self {
        Self {
            text: "Watermark".to_string(),
            position: WatermarkPosition::BottomRight,
            font_size: 48.0,
            color: Rgba([255, 255, 255, 255]), // 白色
            opacity: 0.7,
        }
    }
}

pub fn add_text_watermark(
    img: &mut DynamicImage,
    config: &WatermarkConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // 使用内置字体数据
    let font_data = include_bytes!("../assets/DejaVuSans.ttf");
    let font = Font::try_from_bytes(font_data as &[u8])
        .ok_or("Failed to load font. Please ensure DejaVuSans.ttf exists in assets/ directory")?;

    let scale = Scale::uniform(config.font_size);

    // 估算文字尺寸
    let v_metrics = font.v_metrics(scale);
    let text_width = font
        .layout(&config.text, scale, rusttype::point(0.0, 0.0))
        .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
        .last()
        .unwrap_or(0.0) as u32;
    let text_height = (v_metrics.ascent - v_metrics.descent) as u32;

    // 计算水印位置
    let (x, y) =
        config
            .position
            .calculate_position(img.width(), img.height(), text_width, text_height);

    // 应用透明度
    let mut color = config.color;
    color.0[3] = (color.0[3] as f32 * config.opacity) as u8;

    // 转换为RGBA格式
    let mut rgba_img = img.to_rgba8();

    // 绘制文字
    draw_text_mut(
        &mut rgba_img,
        color,
        x as i32,
        y as i32,
        scale,
        &font,
        &config.text,
    );

    // 更新原图像
    *img = DynamicImage::ImageRgba8(rgba_img);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Image Watermark Tool")
        .version("1.0")
        .about("给图片添加文字水印")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("输入图片路径")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("输出图片路径")
                .required(true),
        )
        .arg(
            Arg::new("text")
                .short('t')
                .long("text")
                .value_name("TEXT")
                .help("水印文字")
                .default_value("Watermark"),
        )
        .arg(
            Arg::new("position")
                .short('p')
                .long("position")
                .value_name("POSITION")
                .help(
                    "水印位置: top-left, top-right, bottom-left, bottom-right, center, 或 x,y 坐标",
                )
                .default_value("bottom-right"),
        )
        .arg(
            Arg::new("size")
                .short('s')
                .long("size")
                .value_name("SIZE")
                .help("字体大小")
                .default_value("48"),
        )
        .arg(
            Arg::new("color")
                .short('c')
                .long("color")
                .value_name("COLOR")
                .help("文字颜色 (格式: r,g,b 或 r,g,b,a)")
                .default_value("255,255,255,255"),
        )
        .arg(
            Arg::new("opacity")
                .long("opacity")
                .value_name("OPACITY")
                .help("透明度 (0.0-1.0)")
                .default_value("1.0"),
        )
        .get_matches();

    let input_path = matches.get_one::<String>("input").unwrap();
    let output_path = matches.get_one::<String>("output").unwrap();
    let text = matches.get_one::<String>("text").unwrap();
    let position_str = matches.get_one::<String>("position").unwrap();
    let font_size: f32 = matches.get_one::<String>("size").unwrap().parse()?;
    let color_str = matches.get_one::<String>("color").unwrap();
    let opacity: f32 = matches.get_one::<String>("opacity").unwrap().parse()?;

    // 解析位置
    let position = match position_str.as_str() {
        "top-left" => WatermarkPosition::TopLeft,
        "top-right" => WatermarkPosition::TopRight,
        "bottom-left" => WatermarkPosition::BottomLeft,
        "bottom-right" => WatermarkPosition::BottomRight,
        "center" => WatermarkPosition::Center,
        custom => {
            let coords: Vec<&str> = custom.split(',').collect();
            if coords.len() == 2 {
                let x: u32 = coords[0].parse()?;
                let y: u32 = coords[1].parse()?;
                WatermarkPosition::Custom(x, y)
            } else {
                return Err("无效的位置格式，请使用 x,y 格式".into());
            }
        }
    };

    // 解析颜色
    let color_parts: Vec<&str> = color_str.split(',').collect();
    let color = match color_parts.len() {
        3 => {
            let r: u8 = color_parts[0].parse()?;
            let g: u8 = color_parts[1].parse()?;
            let b: u8 = color_parts[2].parse()?;
            Rgba([r, g, b, 255])
        }
        4 => {
            let r: u8 = color_parts[0].parse()?;
            let g: u8 = color_parts[1].parse()?;
            let b: u8 = color_parts[2].parse()?;
            let a: u8 = color_parts[3].parse()?;
            Rgba([r, g, b, a])
        }
        _ => return Err("无效的颜色格式，请使用 r,g,b 或 r,g,b,a 格式".into()),
    };

    // 加载图片
    let mut img = image::open(input_path)?;

    println!("原图尺寸: {}x{}", img.width(), img.height());

    // 创建水印配置
    let config = WatermarkConfig {
        text: text.clone(),
        position,
        font_size,
        color,
        opacity,
    };

    // 添加水印
    add_text_watermark(&mut img, &config)?;

    // 保存图片
    img.save(output_path)?;

    println!("水印添加完成！输出文件: {}", output_path);
    println!("水印配置:");
    println!("  文字: {}", config.text);
    println!("  位置: {:?}", config.position);
    println!("  字体大小: {}", config.font_size);
    println!("  颜色: {:?}", config.color);
    println!("  透明度: {}", config.opacity);

    Ok(())
}

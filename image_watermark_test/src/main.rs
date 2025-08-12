use clap::{Arg, Command};
use image::{DynamicImage, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use resvg::usvg;
use tiny_skia::Pixmap;

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

pub struct SvgWatermarkConfig {
    pub svg_path: String,
    pub position: WatermarkPosition,
    pub width: u32,
    pub height: u32,
    pub opacity: f32,
}

impl Default for SvgWatermarkConfig {
    fn default() -> Self {
        Self {
            svg_path: "assets/sample.svg".to_string(),
            position: WatermarkPosition::BottomRight,
            width: 100,
            height: 100,
            opacity: 0.7,
        }
    }
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

pub fn add_svg_watermark(
    img: &mut DynamicImage,
    config: &SvgWatermarkConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // 读取 SVG 文件
    let svg_data = std::fs::read_to_string(&config.svg_path)?;

    // 解析 SVG
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_str(&svg_data, &options)?;

    // 获取 SVG 的原始尺寸
    let svg_size = tree.size();

    // 计算缩放比例以适应指定的水印尺寸（center crop）
    let scale_x = config.width as f32 / svg_size.width();
    let scale_y = config.height as f32 / svg_size.height();
    let scale = scale_x.max(scale_y); // 使用较大的缩放比例实现 center crop

    // 计算实际渲染尺寸
    let render_width = (svg_size.width() * scale) as u32;
    let render_height = (svg_size.height() * scale) as u32;

    // 创建 pixmap 用于渲染 SVG
    let mut pixmap = Pixmap::new(render_width, render_height).ok_or("Failed to create pixmap")?;

    // 渲染 SVG
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );

    // 裁剪到指定尺寸（center crop）
    let crop_x = if render_width > config.width {
        (render_width - config.width) / 2
    } else {
        0
    };
    let crop_y = if render_height > config.height {
        (render_height - config.height) / 2
    } else {
        0
    };

    // 创建裁剪后的图像
    let mut watermark_img = RgbaImage::new(config.width, config.height);

    for y in 0..config.height {
        for x in 0..config.width {
            let src_x = x + crop_x;
            let src_y = y + crop_y;

            if src_x < render_width && src_y < render_height {
                let pixel_idx = ((src_y * render_width + src_x) * 4) as usize;
                let pixel_data = pixmap.data();

                if pixel_idx + 3 < pixel_data.len() {
                    let r = pixel_data[pixel_idx];
                    let g = pixel_data[pixel_idx + 1];
                    let b = pixel_data[pixel_idx + 2];
                    let mut a = pixel_data[pixel_idx + 3];

                    // 应用透明度
                    a = (a as f32 * config.opacity) as u8;

                    watermark_img.put_pixel(x, y, Rgba([r, g, b, a]));
                }
            }
        }
    }

    // 计算水印在原图上的位置
    let (x, y) =
        config
            .position
            .calculate_position(img.width(), img.height(), config.width, config.height);

    // 将水印叠加到原图上
    let mut rgba_img = img.to_rgba8();

    for dy in 0..config.height {
        for dx in 0..config.width {
            let target_x = x + dx;
            let target_y = y + dy;

            if target_x < img.width() && target_y < img.height() {
                let watermark_pixel = watermark_img.get_pixel(dx, dy);
                let base_pixel = rgba_img.get_pixel(target_x, target_y);

                // Alpha 混合
                let alpha = watermark_pixel[3] as f32 / 255.0;
                let inv_alpha = 1.0 - alpha;

                let blended_pixel = Rgba([
                    (watermark_pixel[0] as f32 * alpha + base_pixel[0] as f32 * inv_alpha) as u8,
                    (watermark_pixel[1] as f32 * alpha + base_pixel[1] as f32 * inv_alpha) as u8,
                    (watermark_pixel[2] as f32 * alpha + base_pixel[2] as f32 * inv_alpha) as u8,
                    255, // 保持原图的不透明度
                ]);

                rgba_img.put_pixel(target_x, target_y, blended_pixel);
            }
        }
    }

    // 更新原图像
    *img = DynamicImage::ImageRgba8(rgba_img);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Image Watermark Tool")
        .version("1.0")
        .about("给图片添加文字或SVG水印")
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
            Arg::new("type")
                .long("type")
                .value_name("TYPE")
                .help("水印类型: text 或 svg")
                .default_value("text"),
        )
        .arg(
            Arg::new("text")
                .short('t')
                .long("text")
                .value_name("TEXT")
                .help("水印文字（仅用于文字水印）")
                .default_value("Watermark"),
        )
        .arg(
            Arg::new("svg")
                .long("svg")
                .value_name("SVG_FILE")
                .help("SVG文件路径（仅用于SVG水印）")
                .default_value("assets/sample.svg"),
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
                .help("字体大小（文字水印）")
                .default_value("48"),
        )
        .arg(
            Arg::new("width")
                .short('w')
                .long("width")
                .value_name("WIDTH")
                .help("SVG水印宽度")
                .default_value("100"),
        )
        .arg(
            Arg::new("height")
                .long("height")
                .value_name("HEIGHT")
                .help("SVG水印高度")
                .default_value("100"),
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
    let watermark_type = matches.get_one::<String>("type").unwrap();
    let text = matches.get_one::<String>("text").unwrap();
    let svg_path = matches.get_one::<String>("svg").unwrap();
    let position_str = matches.get_one::<String>("position").unwrap();
    let font_size: f32 = matches.get_one::<String>("size").unwrap().parse()?;
    let svg_width: u32 = matches.get_one::<String>("width").unwrap().parse()?;
    let svg_height: u32 = matches.get_one::<String>("height").unwrap().parse()?;
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

    // 根据水印类型添加相应的水印
    match watermark_type.as_str() {
        "text" => {
            // 创建文字水印配置
            let config = WatermarkConfig {
                text: text.clone(),
                position,
                font_size,
                color,
                opacity,
            };

            // 添加文字水印
            add_text_watermark(&mut img, &config)?;

            println!("文字水印添加完成！");
            println!("水印配置:");
            println!("  文字: {}", config.text);
            println!("  位置: {:?}", config.position);
            println!("  字体大小: {}", config.font_size);
            println!("  颜色: {:?}", config.color);
            println!("  透明度: {}", config.opacity);
        }
        "svg" => {
            // 创建SVG水印配置
            let svg_config = SvgWatermarkConfig {
                svg_path: svg_path.clone(),
                position,
                width: svg_width,
                height: svg_height,
                opacity,
            };

            // 添加SVG水印
            add_svg_watermark(&mut img, &svg_config)?;

            println!("SVG水印添加完成！");
            println!("水印配置:");
            println!("  SVG文件: {}", svg_config.svg_path);
            println!("  位置: {:?}", svg_config.position);
            println!("  尺寸: {}x{}", svg_config.width, svg_config.height);
            println!("  透明度: {}", svg_config.opacity);
        }
        _ => {
            return Err("无效的水印类型，请使用 'text' 或 'svg'".into());
        }
    }

    // 保存图片
    img.save(output_path)?;

    println!("输出文件: {}", output_path);

    Ok(())
}

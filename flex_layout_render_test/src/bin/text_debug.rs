use flex_layout_render::render::canvas::Canvas;
use flex_layout_render::types::*;
use flex_layout_render::error::Result;

fn main() -> Result<()> {
    // 创建一个简单的画布
    let mut canvas = Canvas::new(
        Size::new(400.0, 300.0),
        Color::new(255, 255, 255, 255), // 白色背景
        1.0 // DPI
    );
    
    // 加载默认字体
    let font_data = include_bytes!("../../fonts/DejaVuSans.ttf");
    
    // 在不同位置绘制文本进行测试
    canvas.draw_text_direct(
        "测试文本 (0,20)",
        0.0,
        20.0,
        font_data,
        24.0,
        Color::new(255, 0, 0, 255), // 红色
    )?;
    
    canvas.draw_text_direct(
        "测试文本 (50,60)",
        50.0,
        60.0,
        font_data,
        24.0,
        Color::new(0, 255, 0, 255), // 绿色
    )?;
    
    canvas.draw_text_direct(
        "测试文本 (100,100)",
        100.0,
        100.0,
        font_data,
        24.0,
        Color::new(0, 0, 255, 255), // 蓝色
    )?;
    
    // 绘制一些参考线
    canvas.fill_rect(Rect::new(0.0, 20.0, 400.0, 1.0), Color::new(255, 0, 0, 128));
    canvas.fill_rect(Rect::new(50.0, 60.0, 350.0, 1.0), Color::new(0, 255, 0, 128));
    canvas.fill_rect(Rect::new(100.0, 100.0, 300.0, 1.0), Color::new(0, 0, 255, 128));
    
    // 保存图片
    canvas.save("examples/text_debug_direct.png")?;
    println!("✓ 直接文本绘制测试完成，保存到 examples/text_debug_direct.png");
    
    Ok(())
}
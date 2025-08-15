//! 自动布局渲染系统示例程序

use auto_layout_render_test::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Auto Layout Render Test");
    println!("==========================\n");

    // 测试不同的布局示例
    test_simple_layout()?;

    // 测试复杂布局
    test_complex_layout()?;

    // 测试Debug模式
    test_debug_mode()?;

    println!("✅ 所有测试完成！");
    Ok(())
}

/// 测试Debug模式
fn test_debug_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试Debug模式（显示元素边界框）...");

    let mut engine = AutoLayoutEngine::new();

    // 启用debug模式
    engine.set_debug(true);

    // 使用专门的debug演示文件
    let image_debug = engine.render_from_json_file("examples/debug_demo.json")?;
    AutoLayoutEngine::save_image(&image_debug, "output/debug_demo.png")?;
    println!("✅ Debug模式演示完成 -> output/debug_demo.png");

    // 也测试图片尺寸变体的debug效果
    let image_variants_debug = engine.render_from_json_file("examples/image_size_variants.json")?;
    AutoLayoutEngine::save_image(&image_variants_debug, "output/debug_image_variants.png")?;
    println!("✅ Debug模式（图片变体）完成 -> output/debug_image_variants.png");

    println!("   🎨 Debug边框颜色说明：");
    println!("   🔴 红色边框：文本元素");
    println!("   🟢 绿色边框：图片元素");
    println!("   🔵 蓝色边框：容器元素");
    println!("   🟡 黄色边框：垂直堆叠");
    println!("   🟣 紫色边框：水平堆叠");
    println!("   🔵 青色边框：层叠");
    println!("   ⚪ 灰色边框：间隔器\n");

    Ok(())
}

/// 测试简单布局
fn test_simple_layout() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = AutoLayoutEngine::new();

    println!("🧪 测试简单布局...");
    let image = engine.render_from_json_file("examples/simple.json")?;
    AutoLayoutEngine::save_image(&image, "output/simple.png")?;
    println!("✅ 简单布局渲染完成 -> output/simple.png\n");

    // 测试图片自动尺寸功能
    println!("🧪 测试图片自动尺寸...");
    let image_auto = engine.render_from_json_file("examples/auto_image_size.json")?;
    AutoLayoutEngine::save_image(&image_auto, "output/auto_image_size.png")?;
    println!("✅ 图片自动尺寸测试完成 -> output/auto_image_size.png\n");

    // 测试图片尺寸变体
    println!("🧪 测试图片尺寸变体（完全自动、固定宽度、固定高度）...");
    let image_variants = engine.render_from_json_file("examples/image_size_variants.json")?;
    AutoLayoutEngine::save_image(&image_variants, "output/image_size_variants.png")?;
    println!("✅ 图片尺寸变体测试完成 -> output/image_size_variants.png\n");

    Ok(())
}

/// 测试复杂布局
fn test_complex_layout() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试复杂布局（通过代码构建）...");

    // 通过代码构建复杂布局
    let layout = Layout {
        version: "1.0".to_string(),
        canvas: Canvas {
            width: 500.0,
            height: 300.0,
            background: Color {
                r: 245,
                g: 245,
                b: 245,
                a: 255,
            },
            padding: Padding::all(0.0),
        },
        elements: vec![Element::Container {
            id: "main_container".to_string(),
            properties: ContainerProperties {
                background: Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255,
                },
                border_color: Color {
                    r: 200,
                    g: 200,
                    b: 200,
                    a: 255,
                },
                border_width: 1.0,
                corner_radius: 8.0,
                opacity: 1.0,
                padding: Padding {
                    top: 20.0,
                    right: 20.0,
                    bottom: 20.0,
                    left: 20.0,
                },
            },
            constraints: vec![
                Constraint::new(ConstraintType::Top {
                    target: None,
                    value: 20.0,
                }),
                Constraint::new(ConstraintType::Leading {
                    target: None,
                    value: 20.0,
                }),
                Constraint::new(ConstraintType::Trailing {
                    target: None,
                    value: -20.0,
                }),
                Constraint::new(ConstraintType::Bottom {
                    target: None,
                    value: -20.0,
                }),
            ],
            children: vec![
                Element::Text {
                    id: "header".to_string(),
                    content: "复杂布局示例".to_string(),
                    properties: TextProperties {
                        font_family: "Arial".to_string(),
                        font_size: 24.0,
                        font_weight: FontWeight::Bold,
                        color: Color {
                            r: 51,
                            g: 51,
                            b: 51,
                            a: 255,
                        },
                        alignment: TextAlignment::Center,
                        line_height: 1.2,
                        letter_spacing: 0.0,
                        max_lines: None,
                        line_break_mode: LineBreakMode::WordWrap,
                    },
                    constraints: vec![
                        Constraint::new(ConstraintType::Top {
                            target: None,
                            value: 0.0,
                        }),
                        Constraint::new(ConstraintType::CenterX {
                            target: None,
                            offset: 0.0,
                        }),
                    ],
                },
                Element::Text {
                    id: "content".to_string(),
                    content: "这是一个使用Rust实现的自动布局系统示例".to_string(),
                    properties: TextProperties {
                        font_family: "Arial".to_string(),
                        font_size: 16.0,
                        font_weight: FontWeight::Normal,
                        color: Color {
                            r: 102,
                            g: 102,
                            b: 102,
                            a: 255,
                        },
                        alignment: TextAlignment::Center,
                        line_height: 1.4,
                        letter_spacing: 0.0,
                        max_lines: None,
                        line_break_mode: LineBreakMode::WordWrap,
                    },
                    constraints: vec![
                        Constraint::new(ConstraintType::Top {
                            target: Some("header".to_string()),
                            value: 30.0,
                        }),
                        Constraint::new(ConstraintType::CenterX {
                            target: None,
                            offset: 0.0,
                        }),
                    ],
                },
            ],
        }],
    };

    let mut engine = AutoLayoutEngine::new();
    let image = engine.render_layout(&layout)?;

    AutoLayoutEngine::save_image(&image, "output/complex_layout.png")?;
    println!("✅ 复杂布局渲染完成 -> output/complex_layout.png\n");

    Ok(())
}

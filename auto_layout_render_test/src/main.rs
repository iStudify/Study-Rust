//! 自动布局渲染系统示例程序

use auto_layout_render_test::*;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Auto Layout Render Test");
    println!("==========================\n");

    // 创建示例布局描述
    create_example_layouts()?;

    // 测试不同的布局示例
    test_simple_layout()?;

    test_complex_layout()?;

    println!("✅ 所有测试完成！");
    Ok(())
}

/// 创建示例布局文件
fn create_example_layouts() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 创建示例布局文件...");

    // 创建输出目录
    fs::create_dir_all("examples")?;
    fs::create_dir_all("output")?;

    // 简单布局示例
    let simple_json = r##"{
  "canvas": {
    "width": 400,
    "height": 300,
    "background": "#f0f0f0"
  },
  "elements": [
    {
      "type": "text",
      "id": "title",
      "content": "Hello Auto Layout",
      "properties": {
        "font_size": 24,
        "color": "#333333"
      },
      "constraints": [
        {
          "type": "centerX",
          "constant": 0
        },
        {
          "type": "centerY",
          "constant": 0
        },
        {
          "type": "width",
          "value": 200
        },
        {
          "type": "height",
          "value": 30
        }
      ]
    }
  ]
}"##;

    fs::write("examples/simple.json", simple_json)?;

    println!("✅ 示例文件创建完成\n");
    Ok(())
}

/// 测试简单布局
fn test_simple_layout() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试简单布局...");

    let mut engine = AutoLayoutEngine::new();
    let image = engine.render_from_json_file("examples/simple.json")?;

    AutoLayoutEngine::save_image(&image, "output/simple.png")?;
    println!("✅ 简单布局渲染完成 -> output/simple.png\n");

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
                        Constraint::new(ConstraintType::Width {
                            value: Some(200.0),
                            target: None,
                            multiplier: 1.0,
                            percent: None,
                        }),
                        Constraint::new(ConstraintType::Height {
                            value: Some(30.0),
                            target: None,
                            multiplier: 1.0,
                            percent: None,
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
                        Constraint::new(ConstraintType::Width {
                            value: Some(300.0),
                            target: None,
                            multiplier: 1.0,
                            percent: None,
                        }),
                        Constraint::new(ConstraintType::Height {
                            value: Some(50.0),
                            target: None,
                            multiplier: 1.0,
                            percent: None,
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

use clap::{Arg, Command};
use flex_layout_render::{FlexRenderer, TemplateVariables};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::process;

fn main() {
    env_logger::init();
    
    let matches = Command::new("flex-render")
        .version("0.1.0")
        .author("Gavin <gavin@example.com>")
        .about("A flexible layout rendering engine with DSL support")
        .arg(
            Arg::new("input")
                .short('i')
                .long("input")
                .value_name("FILE")
                .help("Input YAML template file")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output image file")
                .required(true),
        )
        .arg(
            Arg::new("variables")
                .short('v')
                .long("variables")
                .value_name("JSON")
                .help("Template variables as JSON string")
                .required(false),
        )
        .arg(
            Arg::new("var-file")
                .long("var-file")
                .value_name("FILE")
                .help("Template variables from JSON file")
                .required(false),
        )
        .arg(
            Arg::new("validate")
                .long("validate")
                .help("Only validate the template without rendering")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("list-vars")
                .long("list-vars")
                .help("List all template variables")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let input_file = matches.get_one::<String>("input").unwrap();
    let output_file = matches.get_one::<String>("output").unwrap();
    
    // 读取模板文件
    let yaml_content = match fs::read_to_string(input_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading input file '{}': {}", input_file, e);
            process::exit(1);
        }
    };
    
    // 创建渲染器
    let mut renderer = match FlexRenderer::from_yaml(&yaml_content) {
        Ok(renderer) => renderer,
        Err(e) => {
            eprintln!("Error parsing YAML template: {}", e);
            process::exit(1);
        }
    };
    
    // 处理变量
    let variables = load_variables(&matches);
    if let Some(vars) = variables {
        renderer.set_variables(vars);
    }
    
    // 列出变量
    if matches.get_flag("list-vars") {
        match renderer.get_template_variables() {
            Ok(vars) => {
                if vars.is_empty() {
                    println!("No template variables found.");
                } else {
                    println!("Template variables:");
                    for var in vars {
                        println!("  - {}", var);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error getting template variables: {}", e);
                process::exit(1);
            }
        }
        return;
    }
    
    // 验证模式
    if matches.get_flag("validate") {
        match renderer.validate_variables() {
            Ok(missing) => {
                if missing.is_empty() {
                    println!("✓ Template validation passed. All variables are set.");
                } else {
                    println!("⚠ Missing variables:");
                    for var in missing {
                        println!("  - {}", var);
                    }
                    process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Error validating template: {}", e);
                process::exit(1);
            }
        }
        return;
    }
    
    // 渲染图像
    println!("Rendering template '{}' to '{}'...", input_file, output_file);
    
    match renderer.render_to_file(output_file) {
        Ok(()) => {
            println!("✓ Successfully rendered to '{}'", output_file);
        }
        Err(e) => {
            eprintln!("Error rendering image: {}", e);
            process::exit(1);
        }
    }
}

fn load_variables(matches: &clap::ArgMatches) -> Option<TemplateVariables> {
    let mut variables = HashMap::new();
    
    // 从命令行 JSON 字符串加载变量
    if let Some(json_str) = matches.get_one::<String>("variables") {
        match serde_json::from_str::<HashMap<String, Value>>(json_str) {
            Ok(vars) => {
                variables.extend(vars);
            }
            Err(e) => {
                eprintln!("Error parsing variables JSON: {}", e);
                process::exit(1);
            }
        }
    }
    
    // 从文件加载变量
    if let Some(var_file) = matches.get_one::<String>("var-file") {
        let content = match fs::read_to_string(var_file) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading variables file '{}': {}", var_file, e);
                process::exit(1);
            }
        };
        
        match serde_json::from_str::<HashMap<String, Value>>(&content) {
            Ok(vars) => {
                variables.extend(vars);
            }
            Err(e) => {
                eprintln!("Error parsing variables file JSON: {}", e);
                process::exit(1);
            }
        }
    }
    
    if variables.is_empty() {
        None
    } else {
        Some(variables)
    }
}

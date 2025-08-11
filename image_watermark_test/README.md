# 图片文字水印工具

这是一个用 Rust 编写的图片文字水印添加工具，支持自定义水印位置、字体大小、颜色和透明度。

## 功能特性

- 支持多种图片格式（PNG、JPEG、GIF、BMP等）
- 可自定义水印位置：
  - 预设位置：左上角、右上角、左下角、右下角、居中
  - 自定义坐标位置
- 可调节字体大小
- 可自定义文字颜色（RGB/RGBA）
- 可调节透明度
- 命令行界面，使用简单

## 安装依赖

确保你已经安装了 Rust 环境，然后运行：

```bash
cargo build --release
```

## 使用方法

### 基本用法

```bash
cargo run -- -i input.jpg -o output.jpg -t "我的水印"
```

### 完整参数说明

```bash
cargo run -- \
  --input input.jpg \           # 输入图片路径
  --output output.jpg \         # 输出图片路径
  --text "我的水印" \           # 水印文字
  --position bottom-right \     # 水印位置
  --size 48 \                   # 字体大小
  --color 255,255,255,255 \     # 文字颜色 (R,G,B,A)
  --opacity 0.7                 # 透明度 (0.0-1.0)
```

### 位置参数

- `top-left`: 左上角
- `top-right`: 右上角  
- `bottom-left`: 左下角
- `bottom-right`: 右下角
- `center`: 居中
- `x,y`: 自定义坐标（如 `100,200`）

### 颜色参数

- RGB格式：`255,0,0` (红色)
- RGBA格式：`255,0,0,128` (半透明红色)

## 示例

### 在右下角添加白色水印
```bash
cargo run -- -i photo.jpg -o watermarked.jpg -t "© 2024 我的版权"
```

### 在中心位置添加大号红色水印
```bash
cargo run -- -i photo.jpg -o watermarked.jpg -t "SAMPLE" -p center -s 72 -c 255,0,0,200
```

### 在自定义位置添加水印
```bash
cargo run -- -i photo.jpg -o watermarked.jpg -t "Custom Position" -p 50,100
```

## 支持的图片格式

- JPEG (.jpg, .jpeg)
- PNG (.png)
- GIF (.gif)
- BMP (.bmp)
- TIFF (.tiff, .tif)
- WebP (.webp)

## 注意事项

- 输出图片将保存为与输入格式相同的格式
- 字体文件位于 `assets/DejaVuSans.ttf`
- 透明度值范围为 0.0（完全透明）到 1.0（完全不透明）
- 坐标系原点在图片左上角
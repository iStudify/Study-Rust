# 图片水印工具

这是一个用 Rust 编写的图片水印添加工具，支持**文字水印**和**SVG 水印**两种模式，可自定义水印位置、大小、颜色和透明度。

## 功能特性

- 支持多种图片格式（PNG、JPEG、GIF、BMP 等）
- **双水印模式**：
  - **文字水印**：添加自定义文字水印
  - **SVG 水印**：添加矢量图形水印（支持 center crop 缩放）
- 可自定义水印位置：
  - 预设位置：左上角、右上角、左下角、右下角、居中
  - 自定义坐标位置
- 文字水印功能：
  - 可调节字体大小
  - 可自定义文字颜色（RGB/RGBA）
- SVG 水印功能：
  - 可设置水印尺寸（宽度和高度）
  - 自动 center crop 适配指定尺寸
  - 支持透明度和 Alpha 混合
- 可调节透明度
- 命令行界面，使用简单

## 安装依赖

确保你已经安装了 Rust 环境，然后运行：

```bash
cargo build --release
```

## 使用方法

### 命令行参数说明

```bash
cargo run -- [OPTIONS] --input <FILE> --output <FILE>

选项:
  -i, --input <FILE>         输入图片路径
  -o, --output <FILE>        输出图片路径
      --type <TYPE>          水印类型: text 或 svg [默认: text]
  -t, --text <TEXT>          水印文字（仅用于文字水印） [默认: Watermark]
      --svg <SVG_FILE>       SVG文件路径（仅用于SVG水印） [默认: assets/sample.svg]
  -p, --position <POSITION>  水印位置: top-left, top-right, bottom-left, bottom-right, center, 或 x,y 坐标 [默认: bottom-right]
  -s, --size <SIZE>          字体大小（文字水印） [默认: 48]
  -w, --width <WIDTH>        SVG水印宽度 [默认: 100]
      --height <HEIGHT>      SVG水印高度 [默认: 100]
  -c, --color <COLOR>        文字颜色 (格式: r,g,b 或 r,g,b,a) [默认: 255,255,255,255]
      --opacity <OPACITY>    透明度 (0.0-1.0) [默认: 1.0]
  -h, --help                 显示帮助信息
  -V, --version              显示版本信息
```

## 文字水印使用示例

### 基本文字水印

```bash
cargo run -- -i input.jpg -o output.jpg -t "我的水印"
```

### 完整文字水印参数

```bash
cargo run -- \
  --input input.jpg \           # 输入图片路径
  --output output.jpg \         # 输出图片路径
  --type text \                 # 指定文字水印模式
  --text "我的水印" \           # 水印文字
  --position bottom-right \     # 水印位置
  --size 48 \                   # 字体大小
  --color 255,255,255,255 \     # 文字颜色 (R,G,B,A)
  --opacity 0.7                 # 透明度 (0.0-1.0)
```

### 文字水印示例

#### 在右下角添加白色水印

```bash
cargo run -- -i photo.jpg -o watermarked.jpg -t "© 2024 我的版权"
```

#### 在中心位置添加大号红色水印

```bash
cargo run -- -i photo.jpg -o watermarked.jpg -t "SAMPLE" -p center -s 72 -c 255,0,0,200
```

#### 在自定义位置添加水印

```bash
cargo run -- -i photo.jpg -o watermarked.jpg -t "Custom Position" -p 50,100
```

## SVG 水印使用示例

### 基本 SVG 水印

```bash
cargo run -- -i input.jpg -o output.jpg --type svg
```

### 完整 SVG 水印参数

```bash
cargo run -- \
  --input input.jpg \           # 输入图片路径
  --output output.jpg \         # 输出图片路径
  --type svg \                  # 指定SVG水印模式
  --svg assets/logo.svg \       # SVG文件路径
  --position center \           # 水印位置
  --width 150 \                 # SVG水印宽度
  --height 150 \                # SVG水印高度
  --opacity 0.8                 # 透明度 (0.0-1.0)
```

### SVG 水印示例

#### 在右下角添加默认 SVG 水印

```bash
cargo run -- -i photo.jpg -o watermarked.jpg --type svg
```

#### 在中心位置添加大尺寸半透明 SVG 水印

```bash
cargo run -- -i photo.jpg -o watermarked.jpg --type svg -p center --width 200 --height 200 --opacity 0.5
```

#### 使用自定义 SVG 文件和位置

```bash
cargo run -- -i photo.jpg -o watermarked.jpg --type svg --svg my_logo.svg -p top-left --width 120 --height 80
```

#### 在指定坐标添加 SVG 水印

```bash
cargo run -- -i photo.jpg -o watermarked.jpg --type svg -p 100,50 --width 100 --height 100 --opacity 0.7
```

## 参数说明

### 位置参数 (--position)

- `top-left`: 左上角
- `top-right`: 右上角
- `bottom-left`: 左下角
- `bottom-right`: 右下角
- `center`: 居中
- `x,y`: 自定义坐标（如 `100,200`）

### 颜色参数 (--color，仅文字水印)

- RGB 格式：`255,0,0` (红色)
- RGBA 格式：`255,0,0,128` (半透明红色)

### SVG 水印特性

- **Center Crop 缩放**：SVG 会按照指定的宽度和高度进行缩放，使用较大的缩放比例确保完全填充指定区域
- **透明度支持**：SVG 水印支持透明度设置，与原图进行 Alpha 混合
- **矢量渲染**：使用 resvg 库进行高质量的 SVG 渲染

## 支持的图片格式

- JPEG (.jpg, .jpeg)
- PNG (.png)
- GIF (.gif)
- BMP (.bmp)
- TIFF (.tiff, .tif)
- WebP (.webp)

## 注意事项

- 输出图片将保存为与输入格式相同的格式
- 文字水印字体文件位于 `assets/DejaVuSans.ttf`
- 默认 SVG 文件位于 `assets/sample.svg`
- 透明度值范围为 0.0（完全透明）到 1.0（完全不透明）
- 坐标系原点在图片左上角
- SVG 水印会自动进行 center crop 缩放以适配指定尺寸

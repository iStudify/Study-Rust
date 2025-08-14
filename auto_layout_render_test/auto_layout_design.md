# Auto Layout 图文排版系统设计方案

## 1. 系统概述

本系统旨在实现类似 Sketch Auto Layout 的图文排版功能，通过约束系统自动计算元素的位置和尺寸，支持响应式布局和复杂的图文组合。

### 1.1 核心特性

- 声明式布局描述
- 自动约束求解
- 响应式尺寸计算
- 支持嵌套容器
- 灵活的对齐方式
- DSL 配置支持

## 2. 元素类型定义

### 2.1 基础元素

#### Text（文本元素）

**属性：**

- `content`: 文本内容
- `fontSize`: 字体大小
- `fontWeight`: 字体粗细（normal, bold, light）
- `fontFamily`: 字体族
- `color`: 文本颜色
- `alignment`: 文本对齐（leading, center, trailing, justified）
- `lineHeight`: 行高
- `letterSpacing`: 字符间距
- `maxLines`: 最大行数
- `lineBreakMode`: 换行模式（wordWrap, charWrap, truncateHead, truncateTail, truncateMiddle）

#### Image（图片元素）

**属性：**

- `source`: 图片路径或 URL
- `scaleMode`: 缩放模式（fit, fill, stretch, center）
- `aspectRatio`: 宽高比
- `cornerRadius`: 圆角半径
- `opacity`: 透明度
- `tintColor`: 着色

#### Container（容器元素）

**属性：**

- `background`: 背景颜色
- `cornerRadius`: 圆角半径
- `borderWidth`: 边框宽度
- `borderColor`: 边框颜色
- `shadow`: 阴影效果
- `opacity`: 透明度

### 2.2 布局容器

#### VStack（垂直堆叠）

**属性：**

- `spacing`: 元素间距
- `alignment`: 水平对齐方式（leading, center, trailing）
- `distribution`: 分布方式（fill, fillEqually, fillProportionally, equalSpacing, equalCentering）

#### HStack（水平堆叠）

**属性：**

- `spacing`: 元素间距
- `alignment`: 垂直对齐方式（top, center, bottom, firstBaseline, lastBaseline）
- `distribution`: 分布方式（fill, fillEqually, fillProportionally, equalSpacing, equalCentering）

#### ZStack（层叠容器）

**属性：**

- `alignment`: 对齐方式
- `zIndex`: 层级顺序

#### Spacer（弹性空间）

**属性：**

- `minLength`: 最小长度
- `priority`: 优先级

## 3. 约束系统

### 3.1 位置约束

#### 绝对位置约束

- `Top(value)`: 距离父容器顶部的距离
- `Bottom(value)`: 距离父容器底部的距离
- `Leading(value)`: 距离父容器左边的距离
- `Trailing(value)`: 距离父容器右边的距离
- `CenterX(value)`: 水平居中，可选偏移量
- `CenterY(value)`: 垂直居中，可选偏移量

#### 相对位置约束

- `Top(element, spacing)`: 位于指定元素下方
- `Bottom(element, spacing)`: 位于指定元素上方
- `Leading(element, spacing)`: 位于指定元素右侧
- `Trailing(element, spacing)`: 位于指定元素左侧
- `CenterX(element, offset)`: 与指定元素水平对齐
- `CenterY(element, offset)`: 与指定元素垂直对齐

### 3.2 尺寸约束

#### 固定尺寸

- `Width(value)`: 固定宽度
- `Height(value)`: 固定高度
- `Size(width, height)`: 同时设置宽高

#### 相对尺寸

- `Width(element, multiplier)`: 相对于其他元素的宽度
- `Height(element, multiplier)`: 相对于其他元素的高度
- `WidthPercent(container, percent)`: 占容器宽度的百分比
- `HeightPercent(container, percent)`: 占容器高度的百分比

#### 比例约束

- `AspectRatio(ratio)`: 宽高比约束
- `MinWidth(value)`: 最小宽度
- `MaxWidth(value)`: 最大宽度
- `MinHeight(value)`: 最小高度
- `MaxHeight(value)`: 最大高度

### 3.3 对齐约束

#### 边缘对齐

- `AlignTop(element)`: 顶部对齐
- `AlignBottom(element)`: 底部对齐
- `AlignLeading(element)`: 左边缘对齐
- `AlignTrailing(element)`: 右边缘对齐

#### 基线对齐

- `AlignBaseline(element)`: 文本基线对齐
- `AlignFirstBaseline(element)`: 首行基线对齐
- `AlignLastBaseline(element)`: 末行基线对齐

### 3.4 间距约束

#### 内边距

- `Padding(value)`: 四周内边距
- `PaddingTop(value)`: 顶部内边距
- `PaddingBottom(value)`: 底部内边距
- `PaddingLeading(value)`: 左侧内边距
- `PaddingTrailing(value)`: 右侧内边距

#### 外边距

- `Margin(value)`: 四周外边距
- `MarginTop(value)`: 顶部外边距
- `MarginBottom(value)`: 底部外边距
- `MarginLeading(value)`: 左侧外边距
- `MarginTrailing(value)`: 右侧外边距

### 3.5 优先级约束

#### 约束优先级

- `Required(1000)`: 必须满足的约束
- `High(750)`: 高优先级约束
- `Medium(500)`: 中等优先级约束
- `Low(250)`: 低优先级约束
- `Custom(value)`: 自定义优先级

#### 内容优先级

- `ContentHuggingPriority`: 内容紧贴优先级
- `ContentCompressionResistance`: 内容抗压缩优先级

## 4. DSL 设计

### 4.1 JSON DSL 结构

```json
{
  "version": "1.0",
  "canvas": {
    "width": 800,
    "height": 600,
    "background": "#FFFFFF",
    "padding": {
      "top": 20,
      "bottom": 20,
      "left": 20,
      "right": 20
    }
  },
  "elements": [
    {
      "id": "element_id",
      "type": "text|image|vstack|hstack|zstack|spacer",
      "properties": {},
      "constraints": [],
      "children": []
    }
  ]
}
```

### 4.2 YAML DSL 结构

```yaml
version: "1.0"
canvas:
  width: 800
  height: 600
  background: "#FFFFFF"
  padding:
    top: 20
    bottom: 20
    left: 20
    right: 20

elements:
  - id: element_id
    type: text|image|vstack|hstack|zstack|spacer
    properties: {}
    constraints: []
    children: []
```

## 5. 布局示例

### 5.1 图片上文字下垂直排列

**YAML 配置：**

```yaml
version: "1.0"
canvas:
  width: 400
  height: 500
  background: "#F5F5F5"
  padding: 20

elements:
  - id: mainVStack
    type: vstack
    properties:
      spacing: 15
      alignment: center
      distribution: fill
    constraints:
      - type: top
        target: canvas
        value: 0
      - type: leading
        target: canvas
        value: 0
      - type: trailing
        target: canvas
        value: 0
    children:
      - id: heroImage
        type: image
        source: "photo.jpg"
        properties:
          aspectRatio: "16:9"
          scaleMode: fit
          cornerRadius: 8
        constraints:
          - type: width
            target: mainVStack

      - id: title
        type: text
        content: "图片标题"
        properties:
          fontSize: 24
          fontWeight: bold
          color: "#333333"
          alignment: center

      - id: description
        type: text
        content: "这里是图片的详细描述内容"
        properties:
          fontSize: 16
          color: "#666666"
          alignment: center
          lineHeight: 1.5
        constraints:
          - type: width
            target: mainVStack
```

### 5.2 两个文字左右对齐

**YAML 配置：**

```yaml
version: "1.0"
canvas:
  width: 400
  height: 60
  background: "#FFFFFF"
  padding: 20

elements:
  - id: headerHStack
    type: hstack
    properties:
      spacing: 0
      alignment: center
      distribution: fillEqually
    constraints:
      - type: top
        target: canvas
        value: 0
      - type: leading
        target: canvas
        value: 0
      - type: trailing
        target: canvas
        value: 0
      - type: height
        value: 40
    children:
      - id: leftText
        type: text
        content: "左侧文字"
        properties:
          fontSize: 16
          alignment: leading
          color: "#333333"

      - id: rightText
        type: text
        content: "右侧文字"
        properties:
          fontSize: 16
          alignment: trailing
          color: "#333333"
```

### 5.3 复杂图文混排

**YAML 配置：**

```yaml
version: "1.0"
canvas:
  width: 600
  height: 800
  background: "#FFFFFF"
  padding: 30

elements:
  - id: articleContainer
    type: vstack
    properties:
      spacing: 20
      alignment: leading
      distribution: fill
    constraints:
      - type: top
        target: canvas
        value: 0
      - type: leading
        target: canvas
        value: 0
      - type: trailing
        target: canvas
        value: 0
    children:
      # 文章标题
      - id: articleTitle
        type: text
        content: "文章标题：Auto Layout 设计指南"
        properties:
          fontSize: 28
          fontWeight: bold
          color: "#222222"
          alignment: leading
          lineHeight: 1.3

      # 作者信息区域
      - id: authorSection
        type: hstack
        properties:
          spacing: 15
          alignment: center
        constraints:
          - type: width
            target: articleContainer
        children:
          - id: authorAvatar
            type: image
            source: "avatar.jpg"
            properties:
              scaleMode: fill
              cornerRadius: 20
            constraints:
              - type: width
                value: 40
              - type: height
                value: 40

          - id: authorInfo
            type: vstack
            properties:
              spacing: 2
              alignment: leading
            children:
              - id: authorName
                type: text
                content: "作者姓名"
                properties:
                  fontSize: 16
                  fontWeight: bold
                  color: "#333333"

              - id: publishDate
                type: text
                content: "2024年1月15日"
                properties:
                  fontSize: 14
                  color: "#666666"

      # 主要内容图片
      - id: mainImage
        type: image
        source: "main-content.jpg"
        properties:
          aspectRatio: "3:2"
          scaleMode: fit
          cornerRadius: 12
        constraints:
          - type: width
            target: articleContainer

      # 图片说明
      - id: imageCaption
        type: text
        content: "图片说明：这是一张展示Auto Layout概念的示意图"
        properties:
          fontSize: 14
          color: "#888888"
          alignment: center
          lineHeight: 1.4
        constraints:
          - type: width
            target: articleContainer

      # 正文内容
      - id: articleContent
        type: text
        content: "Auto Layout是一种强大的布局系统，它允许开发者通过约束来描述界面元素之间的关系..."
        properties:
          fontSize: 16
          color: "#444444"
          alignment: leading
          lineHeight: 1.6
        constraints:
          - type: width
            target: articleContainer

      # 底部操作区域
      - id: actionSection
        type: hstack
        properties:
          spacing: 20
          alignment: center
          distribution: fillEqually
        constraints:
          - type: width
            target: articleContainer
          - type: height
            value: 50
        children:
          - id: likeButton
            type: text
            content: "👍 点赞"
            properties:
              fontSize: 16
              color: "#007AFF"
              alignment: center
              background: "#F0F8FF"
              cornerRadius: 8
              padding: 12

          - id: shareButton
            type: text
            content: "📤 分享"
            properties:
              fontSize: 16
              color: "#34C759"
              alignment: center
              background: "#F0FFF0"
              cornerRadius: 8
              padding: 12

          - id: commentButton
            type: text
            content: "💬 评论"
            properties:
              fontSize: 16
              color: "#FF9500"
              alignment: center
              background: "#FFF8F0"
              cornerRadius: 8
              padding: 12
```

## 6. 系统架构

### 6.1 核心模块

1. **Layout Tree 模块**

   - 管理元素的层次结构
   - 处理元素的添加、删除、移动
   - 维护父子关系和兄弟关系

2. **Constraint Solver 模块**

   - 实现约束求解算法
   - 处理约束冲突和优先级
   - 计算元素的最终位置和尺寸

3. **Render Engine 模块**

   - 基于计算结果进行图像渲染
   - 支持文本渲染和图片绘制
   - 输出最终的合成图像

4. **DSL Parser 模块**
   - 解析 JSON/YAML 配置文件
   - 构建布局树结构
   - 验证配置的正确性

### 6.2 数据流

```
DSL配置 → DSL解析器 → 布局树 → 约束求解器 → 渲染引擎 → 输出图像
```

## 7. 扩展性设计

### 7.1 自定义元素类型

- 支持插件式的元素类型扩展
- 提供元素类型注册机制
- 支持自定义属性和约束

### 7.2 自定义约束类型

- 支持用户定义的约束规则
- 提供约束求解器扩展接口
- 支持复合约束和条件约束

### 7.3 渲染后端扩展

- 支持多种输出格式（PNG、SVG、PDF 等）
- 提供渲染引擎插件接口
- 支持自定义绘制逻辑

## 8. 性能优化策略

### 8.1 约束求解优化

- 增量约束求解
- 约束缓存机制
- 并行计算支持

### 8.2 渲染优化

- 脏区域检测
- 分层渲染
- 异步渲染支持

### 8.3 内存优化

- 对象池管理
- 延迟加载
- 资源回收机制

---

本设计方案提供了完整的 Auto Layout 系统架构，支持灵活的图文排版需求，具备良好的扩展性和性能表现。通过 DSL 配置，用户可以轻松创建复杂的布局，而无需深入了解底层实现细节。

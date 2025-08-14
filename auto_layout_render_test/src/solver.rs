//! 约束求解器实现

use crate::layout::*;
use cassowary::{
    AddConstraintError, Solver, SuggestValueError, Variable, WeightedRelation::*, strength::*,
};
use std::collections::HashMap;

use rusttype::{Font, Scale, point};
use image::DynamicImage;

#[derive(Debug)]
pub enum SolverError {
    ConstraintError(String),
    SuggestValueError(String),
    ElementNotFound(String),
    InvalidConstraint(String),
}

impl std::fmt::Display for SolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolverError::ConstraintError(msg) => write!(f, "Constraint error: {}", msg),
            SolverError::SuggestValueError(msg) => write!(f, "Suggest value error: {}", msg),
            SolverError::ElementNotFound(id) => write!(f, "Element not found: {}", id),
            SolverError::InvalidConstraint(msg) => write!(f, "Invalid constraint: {}", msg),
        }
    }
}

impl std::error::Error for SolverError {}

impl From<AddConstraintError> for SolverError {
    fn from(err: AddConstraintError) -> Self {
        SolverError::ConstraintError(format!("{:?}", err))
    }
}

impl From<SuggestValueError> for SolverError {
    fn from(err: SuggestValueError) -> Self {
        SolverError::SuggestValueError(format!("{:?}", err))
    }
}

/// 元素变量集合
#[derive(Debug)]
struct ElementVariables {
    x: Variable,
    y: Variable,
    width: Variable,
    height: Variable,
    center_x: Variable,
    center_y: Variable,
    right: Variable,
    bottom: Variable,
}

impl ElementVariables {
    fn new() -> Self {
        Self {
            x: Variable::new(),
            y: Variable::new(),
            width: Variable::new(),
            height: Variable::new(),
            center_x: Variable::new(),
            center_y: Variable::new(),
            right: Variable::new(),
            bottom: Variable::new(),
        }
    }
}

/// 约束求解器
pub struct LayoutSolver {
    solver: Solver,
    variables: HashMap<ElementId, ElementVariables>,
    canvas_vars: ElementVariables,
    fonts: HashMap<String, Font<'static>>,
    images: HashMap<String, DynamicImage>,
}

impl Default for LayoutSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutSolver {
    pub fn new() -> Self {
        Self {
            solver: Solver::new(),
            variables: HashMap::new(),
            canvas_vars: ElementVariables::new(),
            fonts: HashMap::new(),
            images: HashMap::new(),
        }
    }

    /// 加载字体
    pub fn load_font(&mut self, font_family: &str) -> Result<(), SolverError> {
        if self.fonts.contains_key(font_family) {
            return Ok(());
        }

        // 使用默认的 DejaVu Sans 字体
        let font_data = include_bytes!("../assets/fonts/DejaVuSans.ttf");
        let font = Font::try_from_bytes(font_data as &[u8]).ok_or_else(|| {
            SolverError::ConstraintError("Failed to load DejaVu Sans font".to_string())
        })?;

        self.fonts.insert(font_family.to_string(), font);
        Ok(())
    }

    /// 加载图片
    pub fn load_image(&mut self, image_path: &str) -> Result<(), SolverError> {
        if self.images.contains_key(image_path) {
            return Ok(());
        }

        let image = image::open(image_path).map_err(|e| {
            SolverError::ConstraintError(format!("Failed to load image {}: {}", image_path, e))
        })?;

        self.images.insert(image_path.to_string(), image);
        Ok(())
    }

    /// 添加内在尺寸约束
    fn add_intrinsic_size_constraints(&mut self, element: &Element) -> Result<(), SolverError> {
        if let Element::Text {
                content,
                properties,
                constraints,
                ..
            } = element {
            // 检查是否已有显式的宽高约束
            let has_width_constraint = constraints.iter().any(|c| {
                matches!(
                    c.constraint_type,
                    ConstraintType::Width { value: Some(_), .. }
                )
            });
            let has_height_constraint = constraints.iter().any(|c| {
                matches!(
                    c.constraint_type,
                    ConstraintType::Height { value: Some(_), .. }
                )
            });

            if !has_width_constraint || !has_height_constraint {
                // 加载字体
                self.load_font(&properties.font_family)?;

                if let Some(font) = self.fonts.get(&properties.font_family) {
                    let vars = self
                        .variables
                        .get(element.id())
                        .ok_or_else(|| SolverError::ElementNotFound(element.id().clone()))?;

                    if !has_width_constraint {
                        let text_width =
                            self.measure_text_width(content, font, properties.font_size);
                        self.solver
                            .add_constraint(vars.width | EQ(MEDIUM) | (text_width as f64))?;
                    }

                    if !has_height_constraint {
                        // 使用字体大小作为文本高度的近似值
                        let text_height = properties.font_size * 1.2; // 添加一些行间距
                        self.solver
                            .add_constraint(vars.height | EQ(MEDIUM) | (text_height as f64))?;
                    }
                }
            }
        } else if let Element::Image {
                source,
                constraints,
                ..
            } = element {
            // 检查是否已有显式的宽高约束
            let has_width_constraint = constraints.iter().any(|c| {
                matches!(
                    c.constraint_type,
                    ConstraintType::Width { value: Some(_), .. }
                )
            });
            let has_height_constraint = constraints.iter().any(|c| {
                matches!(
                    c.constraint_type,
                    ConstraintType::Height { value: Some(_), .. }
                )
            });

            if !has_width_constraint || !has_height_constraint {
                // 加载图片
                self.load_image(source)?;

                if let Some(image) = self.images.get(source) {
                    let vars = self
                        .variables
                        .get(element.id())
                        .ok_or_else(|| SolverError::ElementNotFound(element.id().clone()))?;

                    let (width, height) = (image.width() as f64, image.height() as f64);

                    if !has_width_constraint {
                        self.solver
                            .add_constraint(vars.width | EQ(MEDIUM) | width)?;
                    }

                    if !has_height_constraint {
                        self.solver
                            .add_constraint(vars.height | EQ(MEDIUM) | height)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// 测量文本宽度
    fn measure_text_width(&self, text: &str, font: &Font<'static>, scale: f32) -> f32 {
        let scale = Scale::uniform(scale);
        let v_metrics = font.v_metrics(scale);
        let glyphs: Vec<_> = font
            .layout(text, scale, point(0.0, v_metrics.ascent))
            .collect();

        if let (Some(first), Some(last)) = (glyphs.first(), glyphs.last()) {
            let min_x = first.pixel_bounding_box().map(|bb| bb.min.x).unwrap_or(0) as f32;
            let max_x = last.pixel_bounding_box().map(|bb| bb.max.x).unwrap_or(0) as f32;
            max_x - min_x
        } else {
            0.0
        }
    }

    /// 求解布局约束
    pub fn solve_layout(&mut self, layout: &Layout) -> Result<ComputedLayout, SolverError> {
        // 清空之前的状态
        self.solver = Solver::new();
        self.variables.clear();
        self.canvas_vars = ElementVariables::new();

        // 设置画布约束
        self.setup_canvas_constraints(&layout.canvas)?;

        // 为所有元素创建变量
        self.create_variables_for_elements(&layout.elements)?;

        // 添加基础约束（位置关系约束）
        self.add_basic_constraints()?;

        // 添加用户定义的约束
        self.add_user_constraints(&layout.elements)?;

        // 求解
        // 注意：由于画布尺寸已经通过约束固定，不需要suggest_value
        // self.solver.suggest_value(self.canvas_vars.width, layout.canvas.width as f64)?;
        // self.solver.suggest_value(self.canvas_vars.height, layout.canvas.height as f64)?;

        // 提取结果
        let mut computed_layout = ComputedLayout::new(Size {
            width: layout.canvas.width,
            height: layout.canvas.height,
        });

        self.extract_results(&layout.elements, &mut computed_layout)?;

        Ok(computed_layout)
    }

    /// 设置画布约束
    fn setup_canvas_constraints(&mut self, canvas: &Canvas) -> Result<(), SolverError> {
        // 画布位置固定在原点
        self.solver
            .add_constraint(self.canvas_vars.x | EQ(REQUIRED) | 0.0)?;
        self.solver
            .add_constraint(self.canvas_vars.y | EQ(REQUIRED) | 0.0)?;

        // 画布尺寸
        self.solver
            .add_constraint(self.canvas_vars.width | EQ(REQUIRED) | canvas.width as f64)?;
        self.solver
            .add_constraint(self.canvas_vars.height | EQ(REQUIRED) | canvas.height as f64)?;

        // 计算画布的中心点和右下角
        self.solver.add_constraint(
            self.canvas_vars.center_x
                | EQ(REQUIRED)
                | (self.canvas_vars.x + self.canvas_vars.width / 2.0),
        )?;
        self.solver.add_constraint(
            self.canvas_vars.center_y
                | EQ(REQUIRED)
                | (self.canvas_vars.y + self.canvas_vars.height / 2.0),
        )?;
        self.solver.add_constraint(
            self.canvas_vars.right | EQ(REQUIRED) | (self.canvas_vars.x + self.canvas_vars.width),
        )?;
        self.solver.add_constraint(
            self.canvas_vars.bottom | EQ(REQUIRED) | (self.canvas_vars.y + self.canvas_vars.height),
        )?;

        Ok(())
    }

    /// 为所有元素创建变量
    fn create_variables_for_elements(&mut self, elements: &[Element]) -> Result<(), SolverError> {
        for element in elements {
            let vars = ElementVariables::new();
            self.variables.insert(element.id().clone(), vars);

            // 递归处理子元素
            if let Some(children) = element.children() {
                self.create_variables_for_elements(children)?;
            }
        }
        Ok(())
    }

    /// 添加基础约束（位置关系约束）
    fn add_basic_constraints(&mut self) -> Result<(), SolverError> {
        // 为每个元素添加基础的位置关系约束
        for vars in self.variables.values() {
            // center_x = x + width / 2
            self.solver
                .add_constraint(vars.center_x | EQ(REQUIRED) | (vars.x + vars.width / 2.0))?;

            // center_y = y + height / 2
            self.solver
                .add_constraint(vars.center_y | EQ(REQUIRED) | (vars.y + vars.height / 2.0))?;

            // right = x + width
            self.solver
                .add_constraint(vars.right | EQ(REQUIRED) | (vars.x + vars.width))?;

            // bottom = y + height
            self.solver
                .add_constraint(vars.bottom | EQ(REQUIRED) | (vars.y + vars.height))?;

            // 尺寸必须为正数
            self.solver
                .add_constraint(vars.width | GE(REQUIRED) | 0.0)?;
            self.solver
                .add_constraint(vars.height | GE(REQUIRED) | 0.0)?;
        }

        Ok(())
    }

    /// 添加用户定义的约束
    fn add_user_constraints(&mut self, elements: &[Element]) -> Result<(), SolverError> {
        for element in elements {
            // 首先添加内在尺寸约束（如果需要）
            self.add_intrinsic_size_constraints(element)?;

            for constraint in element.constraints() {
                self.add_constraint(element.id(), constraint)?;
            }

            // 处理堆叠容器的特殊约束
            match element {
                Element::VStack {
                    children,
                    properties,
                    ..
                } => {
                    self.add_vstack_constraints(element.id(), children, properties)?;
                }
                Element::HStack {
                    children,
                    properties,
                    ..
                } => {
                    self.add_hstack_constraints(element.id(), children, properties)?;
                }
                _ => {}
            }

            // 递归处理子元素
            if let Some(children) = element.children() {
                self.add_user_constraints(children)?;
            }
        }
        Ok(())
    }

    /// 添加单个约束
    fn add_constraint(
        &mut self,
        element_id: &ElementId,
        constraint: &Constraint,
    ) -> Result<(), SolverError> {
        let vars = self
            .variables
            .get(element_id)
            .ok_or_else(|| SolverError::ElementNotFound(element_id.clone()))?;

        let strength = self.priority_to_strength(constraint.priority);

        match &constraint.constraint_type {
            ConstraintType::Top { target, value } => {
                let target_var = if let Some(target_id) = target {
                    if target_id == "canvas" {
                        self.canvas_vars.y
                    } else {
                        self.variables
                            .get(target_id)
                            .ok_or_else(|| SolverError::ElementNotFound(target_id.clone()))?
                            .bottom
                    }
                } else {
                    self.canvas_vars.y
                };

                self.solver
                    .add_constraint(vars.y | EQ(strength) | (target_var + *value as f64))?;
            }

            ConstraintType::Bottom { target, value } => {
                let target_var = if let Some(target_id) = target {
                    if target_id == "canvas" {
                        self.canvas_vars.bottom
                    } else {
                        self.variables
                            .get(target_id)
                            .ok_or_else(|| SolverError::ElementNotFound(target_id.clone()))?
                            .y
                    }
                } else {
                    self.canvas_vars.bottom
                };

                self.solver
                    .add_constraint(vars.bottom | EQ(strength) | (target_var - *value as f64))?;
            }

            ConstraintType::Leading { target, value } => {
                let target_var = if let Some(target_id) = target {
                    if target_id == "canvas" {
                        self.canvas_vars.x
                    } else {
                        self.variables
                            .get(target_id)
                            .ok_or_else(|| SolverError::ElementNotFound(target_id.clone()))?
                            .right
                    }
                } else {
                    self.canvas_vars.x
                };

                self.solver
                    .add_constraint(vars.x | EQ(strength) | (target_var + *value as f64))?;
            }

            ConstraintType::Trailing { target, value } => {
                let target_var = if let Some(target_id) = target {
                    if target_id == "canvas" {
                        self.canvas_vars.right
                    } else {
                        self.variables
                            .get(target_id)
                            .ok_or_else(|| SolverError::ElementNotFound(target_id.clone()))?
                            .x
                    }
                } else {
                    self.canvas_vars.right
                };

                self.solver
                    .add_constraint(vars.right | EQ(strength) | (target_var - *value as f64))?;
            }

            ConstraintType::CenterX { target, offset } => {
                let target_var = if let Some(target_id) = target {
                    if target_id == "canvas" {
                        self.canvas_vars.center_x
                    } else {
                        self.variables
                            .get(target_id)
                            .ok_or_else(|| SolverError::ElementNotFound(target_id.clone()))?
                            .center_x
                    }
                } else {
                    self.canvas_vars.center_x
                };

                self.solver
                    .add_constraint(vars.center_x | EQ(strength) | (target_var + *offset as f64))?;
            }

            ConstraintType::CenterY { target, offset } => {
                let target_var = if let Some(target_id) = target {
                    if target_id == "canvas" {
                        self.canvas_vars.center_y
                    } else {
                        self.variables
                            .get(target_id)
                            .ok_or_else(|| SolverError::ElementNotFound(target_id.clone()))?
                            .center_y
                    }
                } else {
                    self.canvas_vars.center_y
                };

                self.solver
                    .add_constraint(vars.center_y | EQ(strength) | (target_var + *offset as f64))?;
            }

            ConstraintType::Width {
                value,
                target,
                multiplier,
                percent,
            } => {
                if let Some(width_value) = value {
                    self.solver
                        .add_constraint(vars.width | EQ(strength) | (*width_value as f64))?;
                } else if let Some(target_id) = target {
                    let target_var = if target_id == "canvas" {
                        self.canvas_vars.width
                    } else {
                        self.variables
                            .get(target_id)
                            .ok_or_else(|| SolverError::ElementNotFound(target_id.clone()))?
                            .width
                    };

                    if let Some(percent_value) = percent {
                        self.solver.add_constraint(
                            vars.width
                                | EQ(strength)
                                | (target_var * (*percent_value as f64 / 100.0)),
                        )?;
                    } else {
                        self.solver.add_constraint(
                            vars.width | EQ(strength) | (target_var * *multiplier as f64),
                        )?;
                    }
                }
            }

            ConstraintType::Height {
                value,
                target,
                multiplier,
                percent,
            } => {
                if let Some(height_value) = value {
                    self.solver
                        .add_constraint(vars.height | EQ(strength) | (*height_value as f64))?;
                } else if let Some(target_id) = target {
                    let target_var = if target_id == "canvas" {
                        self.canvas_vars.height
                    } else {
                        self.variables
                            .get(target_id)
                            .ok_or_else(|| SolverError::ElementNotFound(target_id.clone()))?
                            .height
                    };

                    if let Some(percent_value) = percent {
                        self.solver.add_constraint(
                            vars.height
                                | EQ(strength)
                                | (target_var * (*percent_value as f64 / 100.0)),
                        )?;
                    } else {
                        self.solver.add_constraint(
                            vars.height | EQ(strength) | (target_var * *multiplier as f64),
                        )?;
                    }
                }
            }

            ConstraintType::AspectRatio { ratio } => {
                self.solver
                    .add_constraint(vars.width | EQ(strength) | (vars.height * *ratio as f64))?;
            }

            ConstraintType::MinWidth { value } => {
                self.solver
                    .add_constraint(vars.width | GE(strength) | (*value as f64))?;
            }

            ConstraintType::MaxWidth { value } => {
                self.solver
                    .add_constraint(vars.width | LE(strength) | (*value as f64))?;
            }

            ConstraintType::MinHeight { value } => {
                self.solver
                    .add_constraint(vars.height | GE(strength) | (*value as f64))?;
            }

            ConstraintType::MaxHeight { value } => {
                self.solver
                    .add_constraint(vars.height | LE(strength) | (*value as f64))?;
            }

            _ => {
                // 其他约束类型的实现
                return Err(SolverError::InvalidConstraint(format!(
                    "Unsupported constraint type: {:?}",
                    constraint.constraint_type
                )));
            }
        }

        Ok(())
    }

    /// 添加垂直堆叠约束
    fn add_vstack_constraints(
        &mut self,
        stack_id: &ElementId,
        children: &[Element],
        properties: &StackProperties,
    ) -> Result<(), SolverError> {
        if children.is_empty() {
            return Ok(());
        }

        let stack_vars = self
            .variables
            .get(stack_id)
            .ok_or_else(|| SolverError::ElementNotFound(stack_id.clone()))?;

        // 垂直排列：每个子元素的顶部等于前一个元素的底部加间距
        for (i, child) in children.iter().enumerate() {
            let child_vars = self
                .variables
                .get(child.id())
                .ok_or_else(|| SolverError::ElementNotFound(child.id().clone()))?;

            if i == 0 {
                // 第一个元素顶部对齐到容器顶部
                self.solver
                    .add_constraint(child_vars.y | EQ(REQUIRED) | stack_vars.y)?;
            } else {
                // 其他元素顶部等于前一个元素底部加间距
                let prev_child = &children[i - 1];
                let prev_vars = self
                    .variables
                    .get(prev_child.id())
                    .ok_or_else(|| SolverError::ElementNotFound(prev_child.id().clone()))?;

                self.solver.add_constraint(
                    child_vars.y | EQ(REQUIRED) | (prev_vars.bottom + properties.spacing as f64),
                )?;
            }

            // 水平对齐
            match properties.alignment {
                Alignment::Leading => {
                    self.solver
                        .add_constraint(child_vars.x | EQ(REQUIRED) | stack_vars.x)?;
                }
                Alignment::Center => {
                    self.solver
                        .add_constraint(child_vars.center_x | EQ(REQUIRED) | stack_vars.center_x)?;
                }
                Alignment::Trailing => {
                    self.solver
                        .add_constraint(child_vars.right | EQ(REQUIRED) | stack_vars.right)?;
                }
                _ => {}
            }
        }

        // 容器高度等于所有子元素高度加间距
        if let Some(last_child) = children.last() {
            let last_vars = self
                .variables
                .get(last_child.id())
                .ok_or_else(|| SolverError::ElementNotFound(last_child.id().clone()))?;

            self.solver.add_constraint(
                stack_vars.height | EQ(REQUIRED) | (last_vars.bottom - stack_vars.y),
            )?;
        }

        Ok(())
    }

    /// 添加水平堆叠约束
    fn add_hstack_constraints(
        &mut self,
        stack_id: &ElementId,
        children: &[Element],
        properties: &StackProperties,
    ) -> Result<(), SolverError> {
        if children.is_empty() {
            return Ok(());
        }

        let stack_vars = self
            .variables
            .get(stack_id)
            .ok_or_else(|| SolverError::ElementNotFound(stack_id.clone()))?;

        // 水平排列：每个子元素的左边等于前一个元素的右边加间距
        for (i, child) in children.iter().enumerate() {
            let child_vars = self
                .variables
                .get(child.id())
                .ok_or_else(|| SolverError::ElementNotFound(child.id().clone()))?;

            if i == 0 {
                // 第一个元素左边对齐到容器左边
                self.solver
                    .add_constraint(child_vars.x | EQ(REQUIRED) | stack_vars.x)?;
            } else {
                // 其他元素左边等于前一个元素右边加间距
                let prev_child = &children[i - 1];
                let prev_vars = self
                    .variables
                    .get(prev_child.id())
                    .ok_or_else(|| SolverError::ElementNotFound(prev_child.id().clone()))?;

                self.solver.add_constraint(
                    child_vars.x | EQ(REQUIRED) | (prev_vars.right + properties.spacing as f64),
                )?;
            }

            // 垂直对齐
            match properties.alignment {
                Alignment::Top => {
                    self.solver
                        .add_constraint(child_vars.y | EQ(REQUIRED) | stack_vars.y)?;
                }
                Alignment::Center => {
                    self.solver
                        .add_constraint(child_vars.center_y | EQ(REQUIRED) | stack_vars.center_y)?;
                }
                Alignment::Bottom => {
                    self.solver
                        .add_constraint(child_vars.bottom | EQ(REQUIRED) | stack_vars.bottom)?;
                }
                _ => {}
            }
        }

        // 容器宽度等于所有子元素宽度加间距
        if let Some(last_child) = children.last() {
            let last_vars = self
                .variables
                .get(last_child.id())
                .ok_or_else(|| SolverError::ElementNotFound(last_child.id().clone()))?;

            self.solver.add_constraint(
                stack_vars.width | EQ(REQUIRED) | (last_vars.right - stack_vars.x),
            )?;
        }

        Ok(())
    }

    /// 提取求解结果
    fn extract_results(
        &self,
        elements: &[Element],
        computed_layout: &mut ComputedLayout,
    ) -> Result<(), SolverError> {
        for element in elements {
            let vars = self
                .variables
                .get(element.id())
                .ok_or_else(|| SolverError::ElementNotFound(element.id().clone()))?;

            let x = self.solver.get_value(vars.x) as f32;
            let y = self.solver.get_value(vars.y) as f32;
            let width = self.solver.get_value(vars.width) as f32;
            let height = self.solver.get_value(vars.height) as f32;
            let frame = Rect {
                origin: Point { x, y },
                size: Size { width, height },
            };

            computed_layout.set_frame(element.id().clone(), frame);

            // 递归处理子元素
            if let Some(children) = element.children() {
                self.extract_results(children, computed_layout)?;
            }
        }

        Ok(())
    }

    /// 将优先级转换为Cassowary强度
    fn priority_to_strength(&self, priority: Priority) -> f64 {
        match priority {
            Priority::Required => REQUIRED,
            Priority::High => STRONG,
            Priority::Medium => MEDIUM,
            Priority::Low => WEAK,
        }
    }
}

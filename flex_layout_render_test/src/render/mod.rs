//! 渲染引擎模块
//!
//! 负责将布局结果渲染为图像输出。

pub mod canvas;
pub mod renderer;

// 重新导出主要类型
pub use canvas::*;
pub use renderer::*;
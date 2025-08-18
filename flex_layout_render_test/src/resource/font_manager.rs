//! 字体管理器
//!
//! 负责字体的加载、缓存和管理。

use crate::error::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// 字体信息
#[derive(Debug, Clone)]
pub struct FontInfo {
    /// 字体族名
    pub family: String,
    /// 字体文件路径
    pub path: PathBuf,
    /// 字体数据
    pub data: Arc<Vec<u8>>,
    /// 字体大小（字节）
    pub size: usize,
}

/// 字体管理器
pub struct FontManager {
    /// 字体缓存
    fonts: Arc<Mutex<HashMap<String, FontInfo>>>,
    /// 字体搜索路径
    search_paths: Vec<PathBuf>,
    /// 默认字体
    default_font: Option<FontInfo>,
}

impl FontManager {
    /// 创建新的字体管理器
    pub fn new() -> Self {
        let mut manager = Self {
            fonts: Arc::new(Mutex::new(HashMap::new())),
            search_paths: Vec::new(),
            default_font: None,
        };

        // 添加系统字体路径
        manager.add_system_font_paths();

        // 尝试加载默认字体
        if let Ok(default_font) = manager.load_default_font() {
            manager.default_font = Some(default_font);
        }

        manager
    }

    /// 添加字体搜索路径
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.search_paths.push(path.as_ref().to_path_buf());
    }

    /// 添加系统字体路径
    fn add_system_font_paths(&mut self) {
        #[cfg(target_os = "macos")]
        {
            self.search_paths
                .push(PathBuf::from("/System/Library/Fonts"));
            self.search_paths.push(PathBuf::from("/Library/Fonts"));
            self.search_paths
                .push(PathBuf::from("/System/Library/Fonts/Helvetica.ttc"));
        }

        #[cfg(target_os = "windows")]
        {
            self.search_paths.push(PathBuf::from("C:\\Windows\\Fonts"));
        }

        #[cfg(target_os = "linux")]
        {
            self.search_paths.push(PathBuf::from("/usr/share/fonts"));
            self.search_paths
                .push(PathBuf::from("/usr/local/share/fonts"));
            self.search_paths.push(PathBuf::from("~/.fonts"));
        }

        // 项目本地字体目录
        self.search_paths.push(PathBuf::from("fonts"));
        self.search_paths.push(PathBuf::from("assets/fonts"));
    }

    /// 加载字体
    pub fn load_font(&mut self, family: &str) -> Result<Arc<Vec<u8>>> {
        // 检查缓存
        {
            let fonts = self.fonts.lock().unwrap();
            if let Some(font_info) = fonts.get(family) {
                return Ok(font_info.data.clone());
            }
        }

        // 搜索字体文件
        let font_path = self
            .find_font_file(family)
            .ok_or_else(|| FlexRenderError::render_error(format!("找不到字体: {}", family)))?;

        // 加载字体数据
        let font_data = std::fs::read(&font_path)
            .map_err(|e| FlexRenderError::render_error(format!("读取字体文件失败: {:?}", e)))?;

        let font_data_arc = Arc::new(font_data);
        let font_info = FontInfo {
            family: family.to_string(),
            path: font_path,
            data: font_data_arc.clone(),
            size: font_data_arc.len(),
        };

        // 缓存字体
        {
            let mut fonts = self.fonts.lock().unwrap();
            fonts.insert(family.to_string(), font_info.clone());
        }

        Ok(font_info.data)
    }

    /// 查找字体文件
    fn find_font_file(&self, family: &str) -> Option<PathBuf> {
        let font_extensions = ["ttf", "otf", "ttc", "woff", "woff2"];

        for search_path in &self.search_paths {
            if search_path.is_file() {
                // 如果搜索路径本身就是文件，直接检查
                if let Some(stem) = search_path.file_stem() {
                    if stem
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&family.to_lowercase())
                    {
                        return Some(search_path.clone());
                    }
                }
                continue;
            }

            if !search_path.exists() {
                continue;
            }

            // 搜索目录中的字体文件
            for ext in &font_extensions {
                let font_file = search_path.join(format!("{}.{}", family, ext));
                if font_file.exists() {
                    return Some(font_file);
                }

                // 尝试不区分大小写的匹配
                let font_file = search_path.join(format!("{}.{}", family.to_lowercase(), ext));
                if font_file.exists() {
                    return Some(font_file);
                }
            }

            // 递归搜索子目录（限制深度）
            if let Ok(entries) = std::fs::read_dir(search_path) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_file() {
                            if let Some(file_name) = entry.file_name().to_str() {
                                if file_name.to_lowercase().contains(&family.to_lowercase()) {
                                    for ext in &font_extensions {
                                        if file_name.to_lowercase().ends_with(ext) {
                                            return Some(entry.path());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// 加载默认字体
    fn load_default_font(&self) -> Result<FontInfo> {
        let default_fonts = [
            "SourceHanSansSC-Regular",
            "DejaVuSans",
            "Arial",
            "Helvetica",
            "DejaVu Sans",
            "Liberation Sans",
            "Noto Sans",
        ];

        for font_name in &default_fonts {
            if let Some(font_path) = self.find_font_file(font_name) {
                if let Ok(font_data) = std::fs::read(&font_path) {
                    return Ok(FontInfo {
                        family: font_name.to_string(),
                        path: font_path,
                        data: Arc::new(font_data.clone()),
                        size: font_data.len(),
                    });
                }
            }
        }

        // 如果找不到任何系统字体，创建一个占位符
        Ok(FontInfo {
            family: "Default".to_string(),
            path: PathBuf::from("default.ttf"),
            data: Arc::new(vec![0; 1024]), // 占位符数据
            size: 1024,
        })
    }

    /// 获取默认字体
    pub fn get_default_font(&self) -> Result<Arc<Vec<u8>>> {
        if let Some(ref default_font) = self.default_font {
            Ok(default_font.data.clone())
        } else {
            Err(FlexRenderError::render_error("没有可用的默认字体"))
        }
    }

    /// 预加载字体
    pub fn preload_fonts(&mut self, families: &[&str]) -> Result<()> {
        for family in families {
            self.load_font(family)?;
        }
        Ok(())
    }

    /// 获取已缓存的字体列表
    pub fn cached_fonts(&self) -> Vec<String> {
        let fonts = self.fonts.lock().unwrap();
        fonts.keys().cloned().collect()
    }

    /// 清理字体缓存
    pub fn clear_cache(&mut self) {
        let mut fonts = self.fonts.lock().unwrap();
        fonts.clear();
    }

    /// 获取缓存统计信息
    pub fn cache_stats(&self) -> (usize, usize) {
        let fonts = self.fonts.lock().unwrap();
        let count = fonts.len();
        let total_size = fonts.values().map(|f| f.size).sum();
        (count, total_size)
    }

    /// 移除特定字体缓存
    pub fn remove_font(&mut self, family: &str) -> bool {
        let mut fonts = self.fonts.lock().unwrap();
        fonts.remove(family).is_some()
    }
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}

// 线程安全的字体管理器单例
lazy_static::lazy_static! {
    static ref GLOBAL_FONT_MANAGER: Arc<Mutex<FontManager>> = {
        Arc::new(Mutex::new(FontManager::new()))
    };
}

/// 获取全局字体管理器
pub fn get_font_manager() -> Arc<Mutex<FontManager>> {
    GLOBAL_FONT_MANAGER.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_manager_creation() {
        let manager = FontManager::new();
        assert!(!manager.search_paths.is_empty());
        assert!(manager.default_font.is_some());
    }

    #[test]
    fn test_add_search_path() {
        let mut manager = FontManager::new();
        let initial_count = manager.search_paths.len();

        manager.add_search_path("/custom/fonts");
        assert_eq!(manager.search_paths.len(), initial_count + 1);
    }

    #[test]
    fn test_cache_operations() {
        let mut manager = FontManager::new();

        let (count, _size) = manager.cache_stats();
        assert_eq!(count, 0);

        // 尝试加载一个可能不存在的字体（不应该崩溃）
        let _ = manager.load_font("NonExistentFont");

        manager.clear_cache();
        let (count, _size) = manager.cache_stats();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_default_font() {
        let manager = FontManager::new();
        let default_font = manager.get_default_font();
        assert!(default_font.is_ok());

        let font_data = default_font.unwrap();
        assert!(!font_data.is_empty());
    }
}

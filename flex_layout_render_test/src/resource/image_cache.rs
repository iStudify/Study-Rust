//! 图片缓存管理器
//!
//! 负责图片的加载、缓存和管理。

use crate::error::*;
use crate::types::*;
use image::{DynamicImage, ImageFormat, GenericImageView};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 缓存的图片信息
#[derive(Debug, Clone)]
pub struct CachedImage {
    /// 图片数据
    pub image: Arc<DynamicImage>,
    /// 原始尺寸
    pub original_size: Size,
    /// 文件路径
    pub path: PathBuf,
    /// 文件大小（字节）
    pub file_size: usize,
    /// 加载时间
    pub loaded_at: Instant,
    /// 最后访问时间
    pub last_accessed: Instant,
    /// 访问次数
    pub access_count: usize,
}

/// 图片缓存配置
#[derive(Debug, Clone)]
pub struct ImageCacheConfig {
    /// 最大缓存数量
    pub max_entries: usize,
    /// 最大缓存大小（字节）
    pub max_size_bytes: usize,
    /// 缓存过期时间
    pub expire_duration: Duration,
    /// 是否启用 LRU 清理
    pub enable_lru: bool,
}

impl Default for ImageCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 100,
            max_size_bytes: 100 * 1024 * 1024, // 100MB
            expire_duration: Duration::from_secs(3600), // 1小时
            enable_lru: true,
        }
    }
}

/// 图片缓存管理器
pub struct ImageCache {
    /// 缓存数据
    cache: Arc<Mutex<HashMap<String, CachedImage>>>,
    /// 缓存配置
    config: ImageCacheConfig,
    /// 当前缓存大小
    current_size: Arc<Mutex<usize>>,
}

impl ImageCache {
    /// 创建新的图片缓存
    pub fn new(config: ImageCacheConfig) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            config,
            current_size: Arc::new(Mutex::new(0)),
        }
    }
    
    /// 加载图片
    pub fn load_image<P: AsRef<Path>>(&self, path: P) -> Result<Arc<DynamicImage>> {
        let path = path.as_ref();
        let path_str = path.to_string_lossy().to_string();
        
        // 检查缓存
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get_mut(&path_str) {
                // 更新访问信息
                cached.last_accessed = Instant::now();
                cached.access_count += 1;
                return Ok(cached.image.clone());
            }
        }
        
        // 加载图片
        let image = self.load_image_from_file(path)?;
        let original_size = Size::new(
            image.width() as f32,
            image.height() as f32,
        );
        
        // 估算内存使用量
        let estimated_size = self.estimate_image_memory_size(&image);
        
        let cached_image = CachedImage {
            image: Arc::new(image),
            original_size,
            path: path.to_path_buf(),
            file_size: estimated_size,
            loaded_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 1,
        };
        
        // 检查是否需要清理缓存
        self.cleanup_if_needed();
        
        // 添加到缓存
        {
            let mut cache = self.cache.lock().unwrap();
            let mut current_size = self.current_size.lock().unwrap();
            
            cache.insert(path_str, cached_image.clone());
            *current_size += estimated_size;
        }
        
        Ok(cached_image.image)
    }
    
    /// 从文件加载图片
    fn load_image_from_file<P: AsRef<Path>>(&self, path: P) -> Result<DynamicImage> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(FlexRenderError::render_error(format!(
                "图片文件不存在: {:?}",
                path
            )));
        }
        
        // 根据文件扩展名确定格式
        let _format = self.detect_image_format(path)?;
        
        // 加载图片
        let image = image::open(path)
            .map_err(|e| FlexRenderError::render_error(format!(
                "加载图片失败: {:?}",
                e
            )))?;
        
        Ok(image)
    }
    
    /// 检测图片格式
    fn detect_image_format<P: AsRef<Path>>(&self, path: P) -> Result<ImageFormat> {
        let path = path.as_ref();
        
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            match ext.as_str() {
                "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
                "png" => Ok(ImageFormat::Png),
                "gif" => Ok(ImageFormat::Gif),
                "webp" => Ok(ImageFormat::WebP),
                "bmp" => Ok(ImageFormat::Bmp),
                "tiff" | "tif" => Ok(ImageFormat::Tiff),
                "ico" => Ok(ImageFormat::Ico),
                _ => Err(FlexRenderError::render_error(format!(
                    "不支持的图片格式: {}",
                    ext
                ))),
            }
        } else {
            Err(FlexRenderError::render_error(
                "无法确定图片格式".to_string()
            ))
        }
    }
    
    /// 估算图片内存使用量
    fn estimate_image_memory_size(&self, image: &DynamicImage) -> usize {
        let (width, height) = image.dimensions();
        let bytes_per_pixel = match image {
            DynamicImage::ImageLuma8(_) => 1,
            DynamicImage::ImageLumaA8(_) => 2,
            DynamicImage::ImageRgb8(_) => 3,
            DynamicImage::ImageRgba8(_) => 4,
            DynamicImage::ImageLuma16(_) => 2,
            DynamicImage::ImageLumaA16(_) => 4,
            DynamicImage::ImageRgb16(_) => 6,
            DynamicImage::ImageRgba16(_) => 8,
            DynamicImage::ImageRgb32F(_) => 12,
            DynamicImage::ImageRgba32F(_) => 16,
            _ => 4, // 默认假设 RGBA
        };
        
        (width * height) as usize * bytes_per_pixel
    }
    
    /// 清理缓存（如果需要）
    fn cleanup_if_needed(&self) {
        let current_size = *self.current_size.lock().unwrap();
        let cache_len = self.cache.lock().unwrap().len();
        
        let needs_cleanup = cache_len >= self.config.max_entries ||
                           current_size >= self.config.max_size_bytes;
        
        if needs_cleanup {
            self.cleanup_cache();
        }
    }
    
    /// 清理缓存
    fn cleanup_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        let mut current_size = self.current_size.lock().unwrap();
        
        let now = Instant::now();
        let mut to_remove = Vec::new();
        
        // 收集过期的条目
        for (key, cached) in cache.iter() {
            if now.duration_since(cached.loaded_at) > self.config.expire_duration {
                to_remove.push((key.clone(), cached.file_size));
            }
        }
        
        // 如果启用 LRU 且仍然需要清理
        if self.config.enable_lru && 
           (cache.len() - to_remove.len() >= self.config.max_entries ||
            *current_size - to_remove.iter().map(|(_, size)| size).sum::<usize>() >= self.config.max_size_bytes) {
            
            // 按最后访问时间排序，移除最久未访问的
            let mut entries: Vec<_> = cache.iter()
                .filter(|(key, _)| !to_remove.iter().any(|(k, _)| k == *key))
                .map(|(key, cached)| (key.clone(), cached.last_accessed, cached.file_size))
                .collect();
            
            entries.sort_by_key(|(_, last_accessed, _)| *last_accessed);
            
            // 移除最久未访问的条目，直到满足限制
            let target_count = self.config.max_entries / 2; // 清理到一半
            let target_size = self.config.max_size_bytes / 2;
            
            let mut removed_size = to_remove.iter().map(|(_, size)| size).sum::<usize>();
            let mut removed_count = to_remove.len();
            
            for (key, _, size) in entries {
                if removed_count >= cache.len() - target_count ||
                   removed_size >= *current_size - target_size {
                    break;
                }
                
                to_remove.push((key, size));
                removed_size += size;
                removed_count += 1;
            }
        }
        
        // 执行移除
        for (key, size) in to_remove {
            cache.remove(&key);
            *current_size = current_size.saturating_sub(size);
        }
    }
    
    /// 预加载图片
    pub fn preload_images<P: AsRef<Path>>(&self, paths: &[P]) -> Result<()> {
        for path in paths {
            self.load_image(path)?;
        }
        Ok(())
    }
    
    /// 获取缓存统计信息
    pub fn cache_stats(&self) -> (usize, usize, usize) {
        let cache = self.cache.lock().unwrap();
        let current_size = *self.current_size.lock().unwrap();
        let total_accesses = cache.values().map(|c| c.access_count).sum();
        
        (cache.len(), current_size, total_accesses)
    }
    
    /// 清空缓存
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        let mut current_size = self.current_size.lock().unwrap();
        
        cache.clear();
        *current_size = 0;
    }
    
    /// 移除特定图片缓存
    pub fn remove_image<P: AsRef<Path>>(&self, path: P) -> bool {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let mut cache = self.cache.lock().unwrap();
        let mut current_size = self.current_size.lock().unwrap();
        
        if let Some(cached) = cache.remove(&path_str) {
            *current_size = current_size.saturating_sub(cached.file_size);
            true
        } else {
            false
        }
    }
    
    /// 检查图片是否在缓存中
    pub fn contains<P: AsRef<Path>>(&self, path: P) -> bool {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let cache = self.cache.lock().unwrap();
        cache.contains_key(&path_str)
    }
    
    /// 获取缓存的图片路径列表
    pub fn cached_images(&self) -> Vec<PathBuf> {
        let cache = self.cache.lock().unwrap();
        cache.values().map(|c| c.path.clone()).collect()
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new(ImageCacheConfig::default())
    }
}

// 全局图片缓存单例
lazy_static::lazy_static! {
    static ref GLOBAL_IMAGE_CACHE: Arc<Mutex<ImageCache>> = {
        Arc::new(Mutex::new(ImageCache::default()))
    };
}

/// 获取全局图片缓存
pub fn get_image_cache() -> Arc<Mutex<ImageCache>> {
    GLOBAL_IMAGE_CACHE.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    #[test]
    fn test_image_cache_creation() {
        let cache = ImageCache::default();
        let (count, size, _) = cache.cache_stats();
        assert_eq!(count, 0);
        assert_eq!(size, 0);
    }
    
    #[test]
    fn test_cache_config() {
        let config = ImageCacheConfig {
            max_entries: 50,
            max_size_bytes: 50 * 1024 * 1024,
            expire_duration: Duration::from_secs(1800),
            enable_lru: false,
        };
        
        let cache = ImageCache::new(config.clone());
        assert_eq!(cache.config.max_entries, 50);
        assert_eq!(cache.config.max_size_bytes, 50 * 1024 * 1024);
        assert!(!cache.config.enable_lru);
    }
    
    #[test]
    fn test_image_format_detection() {
        let cache = ImageCache::default();
        
        assert!(matches!(cache.detect_image_format(Path::new("test.jpg")), Ok(ImageFormat::Jpeg)));
        assert!(matches!(cache.detect_image_format(Path::new("test.png")), Ok(ImageFormat::Png)));
        assert!(matches!(cache.detect_image_format(Path::new("test.gif")), Ok(ImageFormat::Gif)));
        assert!(cache.detect_image_format(Path::new("test.unknown")).is_err());
    }
    
    #[test]
    fn test_cache_operations() {
        let cache = ImageCache::default();
        
        // 测试不存在的文件
        let result = cache.load_image("nonexistent.jpg");
        assert!(result.is_err());
        
        // 测试缓存统计
        let (count, size, accesses) = cache.cache_stats();
        assert_eq!(count, 0);
        assert_eq!(size, 0);
        assert_eq!(accesses, 0);
        
        // 测试清空缓存
        cache.clear();
        let (count, size, _) = cache.cache_stats();
        assert_eq!(count, 0);
        assert_eq!(size, 0);
    }
}
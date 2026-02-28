use std::collections::HashMap;
use std::fs;
use std::time::SystemTime;

use eframe::egui;

#[derive(Clone)]
enum PhotoCacheEntry {
    Loaded {
        texture: egui::TextureHandle,
        modified_at: Option<SystemTime>,
    },
    Failed {
        modified_at: Option<SystemTime>,
    },
}

/// 人物写真テクスチャの読み込みとキャッシュを管理する。
#[derive(Default)]
pub struct PhotoTextureCache {
    entries: HashMap<String, PhotoCacheEntry>,
}

impl PhotoTextureCache {
    /// 指定パスのテクスチャを取得する。未キャッシュ時のみファイルI/Oとデコードを行う。
    pub fn get_or_load(
        &mut self,
        ctx: &egui::Context,
        photo_path: &str,
    ) -> Option<egui::TextureHandle> {
        let modified_at = Self::read_modified_at(photo_path);

        if let Some(entry) = self.entries.get(photo_path) {
            match entry {
                PhotoCacheEntry::Loaded {
                    texture,
                    modified_at: cached_modified_at,
                } if *cached_modified_at == modified_at => {
                    return Some(texture.clone());
                }
                PhotoCacheEntry::Failed {
                    modified_at: cached_modified_at,
                } if *cached_modified_at == modified_at => {
                    return None;
                }
                _ => {}
            }
        }

        let color_image = match Self::load_color_image(photo_path) {
            Some(color_image) => color_image,
            None => {
                self.entries.insert(
                    photo_path.to_string(),
                    PhotoCacheEntry::Failed { modified_at },
                );
                return None;
            }
        };

        let texture = ctx.load_texture(
            format!("person_photo::{photo_path}"),
            color_image,
            Default::default(),
        );
        self.entries.insert(
            photo_path.to_string(),
            PhotoCacheEntry::Loaded {
                texture: texture.clone(),
                modified_at,
            },
        );

        Some(texture)
    }

    fn read_modified_at(photo_path: &str) -> Option<SystemTime> {
        fs::metadata(photo_path).ok()?.modified().ok()
    }

    fn load_color_image(photo_path: &str) -> Option<egui::ColorImage> {
        let image = image::open(photo_path).ok()?;
        let size = [image.width() as usize, image.height() as usize];
        let rgba = image.to_rgba8();
        let pixels = rgba.as_flat_samples();
        Some(egui::ColorImage::from_rgba_unmultiplied(
            size,
            pixels.as_slice(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::PhotoTextureCache;

    #[test]
    fn returns_none_for_invalid_file_path() {
        let mut cache = PhotoTextureCache::default();
        let ctx = eframe::egui::Context::default();
        let texture = cache.get_or_load(&ctx, "__missing_photo__.png");
        assert!(texture.is_none());
    }
}
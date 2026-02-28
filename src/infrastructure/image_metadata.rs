/// 画像ファイルから幅・高さのメタデータを取得する。
pub fn read_image_dimensions(file_path: &str) -> Option<(u32, u32)> {
    let image = image::open(file_path).ok()?;
    Some((image.width(), image.height()))
}

#[cfg(test)]
mod tests {
    use super::read_image_dimensions;

    #[test]
    fn returns_none_for_nonexistent_file() {
        let result = read_image_dimensions("__not_found_image__.png");
        assert!(result.is_none());
    }
}
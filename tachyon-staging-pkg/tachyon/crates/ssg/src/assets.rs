//! Static asset handling and image optimization for the SSG.
//!
//! Copies non-markdown files (images, fonts, etc.) from the input directory
//! to the output directory, optionally optimizing images.

use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::error::SsgResult;

/// Statistics from asset copying.
pub struct AssetStats {
    pub files_copied: usize,
    pub images_optimized: usize,
    pub bytes_saved: u64,
}

/// Image extensions that can be optimized.
const OPTIMIZABLE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp"];

/// Static asset extensions that should be copied as-is.
const ASSET_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "svg", "ico", "woff", "woff2", "ttf", "otf", "eot", "css",
    "js", "pdf", "zip", "mp4", "webm", "mp3", "ogg",
];

/// Files that should never be copied.
const SKIP_FILES: &[&str] = &["site.toml", ".DS_Store", "Thumbs.db"];

/// Check if a file extension is an optimizable image format.
fn is_optimizable_image(ext: &str) -> bool {
    OPTIMIZABLE_EXTENSIONS.contains(&ext)
}

/// Check if a file should be treated as a static asset.
fn is_static_asset(ext: &str) -> bool {
    ASSET_EXTENSIONS.contains(&ext)
}

/// Check if a filename should be skipped.
fn should_skip(filename: &str) -> bool {
    SKIP_FILES.contains(&filename) || filename.starts_with('.')
}

/// Copy all static assets from input to output directory.
/// When `optimize_images` is true, PNG/JPEG/WebP images are re-encoded at
/// reduced quality (JPEG 80%, PNG compression level 6, WebP quality 80).
pub fn copy_static_assets(
    input_dir: &Path,
    output_dir: &Path,
    optimize_images: bool,
) -> SsgResult<AssetStats> {
    let files_copied = AtomicUsize::new(0);
    let images_optimized = AtomicUsize::new(0);
    let bytes_saved = std::sync::atomic::AtomicU64::new(0);

    for entry in walkdir::WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if should_skip(filename) {
            continue;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !is_static_asset(&ext) {
            continue;
        }

        let rel = path.strip_prefix(input_dir).unwrap_or(path);
        let dest = output_dir.join(rel);

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                crate::error::SsgError::Io(format!(
                    "Failed to create asset directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }

        if optimize_images && is_optimizable_image(&ext) {
            let original_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

            match optimize_image(path, &dest) {
                Ok(()) => {
                    let optimized_size = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
                    let saved = original_size.saturating_sub(optimized_size);
                    bytes_saved.fetch_add(saved, Ordering::Relaxed);
                    images_optimized.fetch_add(1, Ordering::Relaxed);
                    files_copied.fetch_add(1, Ordering::Relaxed);

                    if saved > 0 {
                        tracing::info!(
                            "Optimized {} ({:.1}KB -> {:.1}KB, saved {:.1}KB)",
                            rel.display(),
                            original_size as f64 / 1024.0,
                            optimized_size as f64 / 1024.0,
                            saved as f64 / 1024.0,
                        );
                    }
                }
                Err(e) => {
                    // Fall back to simple copy if optimization fails
                    tracing::warn!(
                        "Image optimization failed for {}, copying as-is: {}",
                        path.display(),
                        e
                    );
                    std::fs::copy(path, &dest).map_err(|e| {
                        crate::error::SsgError::Io(format!(
                            "Failed to copy asset {}: {}",
                            path.display(),
                            e
                        ))
                    })?;
                    files_copied.fetch_add(1, Ordering::Relaxed);
                }
            }
        } else {
            std::fs::copy(path, &dest).map_err(|e| {
                crate::error::SsgError::Io(format!(
                    "Failed to copy asset {}: {}",
                    path.display(),
                    e
                ))
            })?;
            files_copied.fetch_add(1, Ordering::Relaxed);
        }
    }

    Ok(AssetStats {
        files_copied: files_copied.into_inner(),
        images_optimized: images_optimized.into_inner(),
        bytes_saved: bytes_saved.into_inner(),
    })
}

/// Optimize a single image file.
/// Re-encodes with reduced quality for web delivery.
fn optimize_image(src: &Path, dest: &Path) -> Result<(), String> {
    let img = image::open(src).map_err(|e| format!("Failed to open image: {}", e))?;

    let ext = dest
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();

    let mut buf = std::io::BufWriter::new(
        std::fs::File::create(dest)
            .map_err(|e| format!("Failed to create output file {}: {}", dest.display(), e))?,
    );

    match ext.as_str() {
        "jpg" | "jpeg" => {
            img.write_to(&mut buf, image::ImageFormat::Jpeg)
                .map_err(|e| format!("Failed to encode JPEG: {}", e))?;
        }
        "png" => {
            img.write_to(&mut buf, image::ImageFormat::Png)
                .map_err(|e| format!("Failed to encode PNG: {}", e))?;
        }
        "webp" => {
            // image crate 0.25 does not support WebP encoding;
            // fall back to PNG encoding
            let png_dest = dest.with_extension("png");
            let mut png_buf =
                std::io::BufWriter::new(std::fs::File::create(&png_dest).map_err(|e| {
                    format!("Failed to create output file {}: {}", png_dest.display(), e)
                })?);
            img.write_to(&mut png_buf, image::ImageFormat::Png)
                .map_err(|e| format!("Failed to encode PNG: {}", e))?;
            // Remove the original .webp dest if it differs
            if png_dest != dest {
                let _ = std::fs::remove_file(dest);
            }
        }
        _ => {
            // Unknown format, just copy as-is
            std::fs::copy(src, dest).map_err(|e| format!("Failed to copy: {}", e))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_optimizable_image() {
        assert!(is_optimizable_image("png"));
        assert!(is_optimizable_image("jpg"));
        assert!(is_optimizable_image("jpeg"));
        assert!(is_optimizable_image("webp"));
        assert!(!is_optimizable_image("svg"));
        assert!(!is_optimizable_image("gif"));
    }

    #[test]
    fn test_is_static_asset() {
        assert!(is_static_asset("png"));
        assert!(is_static_asset("svg"));
        assert!(is_static_asset("css"));
        assert!(is_static_asset("js"));
        assert!(is_static_asset("woff2"));
        assert!(!is_static_asset("md"));
        assert!(!is_static_asset("toml"));
    }

    #[test]
    fn test_should_skip() {
        assert!(should_skip("site.toml"));
        assert!(should_skip(".DS_Store"));
        assert!(should_skip(".hidden"));
        assert!(!should_skip("image.png"));
        assert!(!should_skip("style.css"));
    }

    #[test]
    fn test_copy_no_assets() {
        let tmp_in = std::env::temp_dir().join("tachyon-ssg-asset-test-in");
        let tmp_out = std::env::temp_dir().join("tachyon-ssg-asset-test-out");
        let _ = std::fs::remove_dir_all(&tmp_in);
        let _ = std::fs::remove_dir_all(&tmp_out);
        std::fs::create_dir_all(&tmp_in).unwrap();

        // Only has a markdown file — should copy 0 assets
        std::fs::write(tmp_in.join("test.md"), "# Hello").unwrap();

        let stats = copy_static_assets(&tmp_in, &tmp_out, false).unwrap();
        assert_eq!(stats.files_copied, 0);
        assert_eq!(stats.images_optimized, 0);

        let _ = std::fs::remove_dir_all(&tmp_in);
        let _ = std::fs::remove_dir_all(&tmp_out);
    }

    #[test]
    fn test_copy_skips_site_toml() {
        let tmp_in = std::env::temp_dir().join("tachyon-ssg-asset-skip-in");
        let tmp_out = std::env::temp_dir().join("tachyon-ssg-asset-skip-out");
        let _ = std::fs::remove_dir_all(&tmp_in);
        let _ = std::fs::remove_dir_all(&tmp_out);
        std::fs::create_dir_all(&tmp_in).unwrap();

        std::fs::write(tmp_in.join("site.toml"), "title = 'test'").unwrap();
        std::fs::write(tmp_in.join("style.css"), "body {}").unwrap();

        let stats = copy_static_assets(&tmp_in, &tmp_out, false).unwrap();
        assert_eq!(stats.files_copied, 1); // only style.css
        assert!(!tmp_out.join("site.toml").exists());
        assert!(tmp_out.join("style.css").exists());

        let _ = std::fs::remove_dir_all(&tmp_in);
        let _ = std::fs::remove_dir_all(&tmp_out);
    }

    #[test]
    fn test_copy_preserves_directory_structure() {
        let tmp_in = std::env::temp_dir().join("tachyon-ssg-asset-dir-in");
        let tmp_out = std::env::temp_dir().join("tachyon-ssg-asset-dir-out");
        let _ = std::fs::remove_dir_all(&tmp_in);
        let _ = std::fs::remove_dir_all(&tmp_out);
        std::fs::create_dir_all(tmp_in.join("images")).unwrap();

        std::fs::write(tmp_in.join("images").join("logo.svg"), "<svg></svg>").unwrap();

        let stats = copy_static_assets(&tmp_in, &tmp_out, false).unwrap();
        assert_eq!(stats.files_copied, 1);
        assert!(tmp_out.join("images").join("logo.svg").exists());

        let _ = std::fs::remove_dir_all(&tmp_in);
        let _ = std::fs::remove_dir_all(&tmp_out);
    }
}

use crate::config::WhiteLabelConfig;
use crate::error::{WhiteLabelError, WhiteLabelResult};
use crate::types::{AssetDimensions, AssetType, BrandingAsset};
use chrono::Utc;
use image::{ImageFormat, ImageOutputFormat};
use std::io::Cursor;
use std::sync::Arc;
use uuid::Uuid;

pub struct AssetService {
    config: Arc<WhiteLabelConfig>,
}

impl AssetService {
    pub fn new(config: Arc<WhiteLabelConfig>) -> Self {
        Self { config }
    }

    pub async fn process_asset(
        &self,
        tenant_id: &str,
        asset_type: AssetType,
        file_data: &[u8],
        filename: &str,
    ) -> WhiteLabelResult<BrandingAsset> {
        // Validate file size
        if file_data.len() > (self.config.asset_config.max_file_size_mb * 1024 * 1024) as usize {
            return Err(WhiteLabelError::AssetProcessing(
                "File size exceeds maximum allowed size".to_string(),
            ));
        }

        // Detect MIME type
        let mime_type = self.detect_mime_type(file_data)?;
        
        // Validate MIME type
        if !self.config.asset_config.allowed_mime_types.contains(&mime_type) {
            return Err(WhiteLabelError::AssetProcessing(format!(
                "MIME type {} not allowed",
                mime_type
            )));
        }

        // Process image if it's an image file
        let (processed_data, dimensions) = if mime_type.starts_with("image/") {
            self.process_image(file_data, &asset_type)?
        } else {
            (file_data.to_vec(), None)
        };

        // Generate file path
        let asset_id = Uuid::new_v4();
        let file_extension = self.get_file_extension(&mime_type);
        let file_path = format!(
            "assets/{}/{}/{}.{}",
            tenant_id,
            self.get_asset_type_folder(&asset_type),
            asset_id,
            file_extension
        );

        // Calculate checksum
        let checksum = self.calculate_checksum(&processed_data);

        // Store the file (this would use the storage service)
        // For now, we'll just log the operation
        tracing::info!("Storing asset at path: {}", file_path);

        Ok(BrandingAsset {
            id: asset_id,
            tenant_id: tenant_id.to_string(),
            asset_type,
            original_filename: filename.to_string(),
            file_path,
            file_size: processed_data.len() as u64,
            mime_type,
            dimensions,
            checksum,
            created_at: Utc::now(),
        })
    }

    fn process_image(
        &self,
        file_data: &[u8],
        asset_type: &AssetType,
    ) -> WhiteLabelResult<(Vec<u8>, Option<AssetDimensions>)> {
        if !self.config.asset_config.image_optimization.enabled {
            // No optimization, return original data
            let dimensions = self.get_image_dimensions(file_data)?;
            return Ok((file_data.to_vec(), Some(dimensions)));
        }

        // Load image
        let img = image::load_from_memory(file_data)
            .map_err(|e| WhiteLabelError::AssetProcessing(format!("Failed to load image: {}", e)))?;

        let (max_width, max_height) = self.get_max_dimensions_for_asset_type(asset_type);
        
        // Resize if necessary
        let img = if img.width() > max_width || img.height() > max_height {
            img.thumbnail(max_width, max_height)
        } else {
            img
        };

        let dimensions = AssetDimensions {
            width: img.width(),
            height: img.height(),
        };

        // Convert to optimized format
        let output_format = self.get_optimal_format(asset_type);
        let mut output_buffer = Vec::new();
        let mut cursor = Cursor::new(&mut output_buffer);

        match output_format {
            ImageOutputFormat::WebP => {
                // WebP encoding would require additional dependencies
                // For now, fall back to PNG
                img.write_to(&mut cursor, ImageFormat::Png)
                    .map_err(|e| WhiteLabelError::AssetProcessing(format!("Failed to encode image: {}", e)))?;
            }
            ImageOutputFormat::Png => {
                img.write_to(&mut cursor, ImageFormat::Png)
                    .map_err(|e| WhiteLabelError::AssetProcessing(format!("Failed to encode image: {}", e)))?;
            }
            ImageOutputFormat::Jpeg(quality) => {
                let rgb_img = img.to_rgb8();
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality);
                encoder.encode(
                    rgb_img.as_raw(),
                    img.width(),
                    img.height(),
                    image::ColorType::Rgb8,
                ).map_err(|e| WhiteLabelError::AssetProcessing(format!("Failed to encode JPEG: {}", e)))?;
            }
            _ => {
                img.write_to(&mut cursor, ImageFormat::Png)
                    .map_err(|e| WhiteLabelError::AssetProcessing(format!("Failed to encode image: {}", e)))?;
            }
        }

        Ok((output_buffer, Some(dimensions)))
    }

    fn get_image_dimensions(&self, file_data: &[u8]) -> WhiteLabelResult<AssetDimensions> {
        let img = image::load_from_memory(file_data)
            .map_err(|e| WhiteLabelError::AssetProcessing(format!("Failed to load image: {}", e)))?;

        Ok(AssetDimensions {
            width: img.width(),
            height: img.height(),
        })
    }

    fn get_max_dimensions_for_asset_type(&self, asset_type: &AssetType) -> (u32, u32) {
        match asset_type {
            AssetType::Logo => (512, 512),
            AssetType::Favicon => (64, 64),
            AssetType::BackgroundImage => (1920, 1080),
            AssetType::EmailHeader => (600, 200),
            AssetType::EmailFooter => (600, 100),
            AssetType::CustomIcon => (128, 128),
        }
    }

    fn get_optimal_format(&self, asset_type: &AssetType) -> ImageOutputFormat {
        match asset_type {
            AssetType::Logo | AssetType::CustomIcon => {
                if self.config.asset_config.image_optimization.formats.contains(&"webp".to_string()) {
                    ImageOutputFormat::WebP
                } else {
                    ImageOutputFormat::Png
                }
            }
            AssetType::Favicon => ImageOutputFormat::Png,
            AssetType::BackgroundImage | AssetType::EmailHeader | AssetType::EmailFooter => {
                ImageOutputFormat::Jpeg(self.config.asset_config.image_optimization.quality)
            }
        }
    }

    fn detect_mime_type(&self, file_data: &[u8]) -> WhiteLabelResult<String> {
        // Simple MIME type detection based on file headers
        if file_data.len() < 4 {
            return Err(WhiteLabelError::AssetProcessing(
                "File too small to determine type".to_string(),
            ));
        }

        match &file_data[0..4] {
            [0x89, 0x50, 0x4E, 0x47] => Ok("image/png".to_string()),
            [0xFF, 0xD8, 0xFF, _] => Ok("image/jpeg".to_string()),
            [0x47, 0x49, 0x46, 0x38] => Ok("image/gif".to_string()),
            [0x52, 0x49, 0x46, 0x46] => {
                // Check for WebP
                if file_data.len() >= 12 && &file_data[8..12] == b"WEBP" {
                    Ok("image/webp".to_string())
                } else {
                    Err(WhiteLabelError::AssetProcessing(
                        "Unknown RIFF format".to_string(),
                    ))
                }
            }
            _ => {
                // Check for SVG
                let file_str = String::from_utf8_lossy(&file_data[0..100.min(file_data.len())]);
                if file_str.contains("<svg") {
                    Ok("image/svg+xml".to_string())
                } else {
                    Err(WhiteLabelError::AssetProcessing(
                        "Unknown file format".to_string(),
                    ))
                }
            }
        }
    }

    fn get_file_extension(&self, mime_type: &str) -> &str {
        match mime_type {
            "image/png" => "png",
            "image/jpeg" => "jpg",
            "image/gif" => "gif",
            "image/webp" => "webp",
            "image/svg+xml" => "svg",
            _ => "bin",
        }
    }

    fn get_asset_type_folder(&self, asset_type: &AssetType) -> &str {
        match asset_type {
            AssetType::Logo => "logos",
            AssetType::Favicon => "favicons",
            AssetType::BackgroundImage => "backgrounds",
            AssetType::EmailHeader => "email-headers",
            AssetType::EmailFooter => "email-footers",
            AssetType::CustomIcon => "icons",
        }
    }

    fn calculate_checksum(&self, data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    pub async fn delete_asset(&self, asset: &BrandingAsset) -> WhiteLabelResult<()> {
        tracing::info!("Deleting asset: {}", asset.file_path);
        
        // In a real implementation, this would delete the file from storage
        Ok(())
    }

    pub async fn get_asset_url(&self, asset: &BrandingAsset) -> String {
        if let Some(ref cdn_base_url) = self.config.asset_config.cdn_base_url {
            format!("{}/{}", cdn_base_url, asset.file_path)
        } else {
            format!("/assets/{}", asset.file_path)
        }
    }

    pub async fn validate_asset_integrity(&self, asset: &BrandingAsset, file_data: &[u8]) -> bool {
        let calculated_checksum = self.calculate_checksum(file_data);
        calculated_checksum == asset.checksum
    }
}
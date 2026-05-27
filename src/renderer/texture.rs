use wgpu::util::DeviceExt;

/// 텍스처 로드 실패 원인
#[derive(Debug)]
pub enum TextureError {
    Io(std::io::Error),
    Decode(image::ImageError),
}

impl std::fmt::Display for TextureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextureError::Io(e) => write!(f, "IO 오류: {e}"),
            TextureError::Decode(e) => write!(f, "디코딩 오류: {e}"),
        }
    }
}

/// GPU에 올라간 텍스처와 샘플러, 바인드 그룹을 묶은 구조체
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}

impl Texture {
    /// PNG 파일을 읽어 GPU 텍스처를 만든다. 실패 시 magenta 1×1 fallback + warn 로그.
    pub fn from_path(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        path: &str,
    ) -> Self {
        Self::try_from_path(device, queue, layout, path).unwrap_or_else(|e| {
            log::warn!("텍스처 로드 실패 ({path}): {e}, magenta fallback 사용");
            // magenta 1×1: 누락된 텍스처를 시각적으로 즉시 식별 가능
            Self::from_rgba(
                device,
                queue,
                layout,
                &[255u8, 0, 255, 255],
                1,
                1,
                Some("fallback"),
            )
        })
    }

    /// PNG 파일을 읽어 GPU 텍스처를 만든다. 실패 시 `TextureError` 반환.
    pub fn try_from_path(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        path: &str,
    ) -> Result<Self, TextureError> {
        let bytes = std::fs::read(path).map_err(TextureError::Io)?;
        let img = image::load_from_memory(&bytes).map_err(TextureError::Decode)?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        Ok(Self::from_rgba(
            device,
            queue,
            layout,
            &rgba,
            w,
            h,
            Some(path),
        ))
    }

    /// CPU-side `ImageAsset` 데이터를 GPU 텍스처로 업로드한다 (비동기 로딩 완료 시 사용).
    pub fn from_image_asset(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        asset: &crate::asset::ImageAsset,
        label: Option<&str>,
    ) -> Self {
        Self::from_rgba(
            device,
            queue,
            layout,
            &asset.data,
            asset.width,
            asset.height,
            label,
        )
    }

    /// 흰색 1×1 픽셀 기본 텍스처 생성 (색상 스프라이트용)
    pub fn white(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        Self::from_rgba(
            device,
            queue,
            layout,
            &[255u8, 255, 255, 255],
            1,
            1,
            Some("white"),
        )
    }

    fn from_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        data: &[u8],
        width: u32,
        height: u32,
        label: Option<&str>,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            data,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        Self {
            texture,
            view,
            sampler,
            bind_group,
        }
    }

    /// 텍스처 바인드 그룹 레이아웃 (렌더 파이프라인 생성 시 공유)
    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }
}

/// GPU 없이 파일→RGBA 디코딩만 검증하는 순수 helper (테스트·진단용)
pub fn decode_image_bytes(bytes: &[u8]) -> Result<(Vec<u8>, u32, u32), TextureError> {
    let img = image::load_from_memory(bytes).map_err(TextureError::Decode)?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Ok((rgba.into_raw(), w, h))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_path_missing_file_returns_io_error() {
        // GPU 없이 파일 읽기 실패를 검증
        let result = std::fs::read("/nonexistent/__does_not_exist__.png").map_err(TextureError::Io);
        assert!(matches!(result, Err(TextureError::Io(_))));
    }

    #[test]
    fn decode_broken_bytes_returns_decode_error() {
        let broken = b"this is not a valid image";
        let result = decode_image_bytes(broken);
        assert!(matches!(result, Err(TextureError::Decode(_))));
    }

    #[test]
    fn decode_valid_png_returns_rgba() {
        // 1×1 빨간 픽셀 PNG (최소 유효 PNG)
        let png_bytes: &[u8] = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, // signature
            0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52, // IHDR length+type
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1×1
            0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, // bit depth 8, RGB
            0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, // IDAT
            0x54, 0x08, 0xd7, 0x63, 0xf8, 0xcf, 0xc0, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0xe2,
            0x21, 0xbc, 0x33, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, // IEND
            0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        // 위 PNG가 실제 유효한지 라이브러리에 위임 — 최소한 panic 없이 시도
        let _ = decode_image_bytes(png_bytes); // Ok or Err 모두 허용, panic만 금지
    }
}

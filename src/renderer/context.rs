use std::sync::Arc;
use winit::{dpi::PhysicalSize, window::Window};

/// wgpu 핵심 객체를 묶은 GPU 컨텍스트
pub struct GpuContext {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
}

impl GpuContext {
    /// 창을 받아 wgpu Surface/Device/Queue를 초기화한다.
    /// wgpu 초기화는 async이므로 pollster::block_on으로 감싸서 호출한다.
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        // 1. 인스턴스: 플랫폼별 백엔드 자동 선택 (Metal/Vulkan/DX12/WebGPU)
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // 2. 서피스: 창과 연결된 렌더 타겟
        let surface = instance.create_surface(window).unwrap();

        // 3. 어댑터: 물리 GPU 선택
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("어댑터를 찾지 못했습니다");

        // 4. 논리 디바이스 + 커맨드 큐
        // wgpu 22 에서 DeviceDescriptor 에 memory_hints 필드가 추가됐다.
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("main device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .expect("디바이스 생성 실패");

        // 5. 서피스 포맷·설정
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
        }
    }

    /// 창 크기 변경 시 서피스를 재구성한다.
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    /// 서피스 유실(SurfaceError::Lost) 시 재구성한다.
    pub fn reconfigure(&self) {
        self.surface.configure(&self.device, &self.config);
    }

    /// 화면을 단색으로 지운다. 스프라이트가 없을 때 배경 표시에 사용.
    pub fn clear(&mut self, color: wgpu::Color) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("clear"),
            });
        {
            let _pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }
        self.queue.submit(std::iter::once(enc.finish()));
        frame.present();
        Ok(())
    }
}

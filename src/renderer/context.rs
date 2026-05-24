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
        // WASM: winit이 canvas를 attach한 직후 inner_size()가 1x1을 반환하는 경우가 있다.
        // canvas의 width/height 속성을 DOM에서 직접 읽어 실제 해상도를 사용한다.
        #[cfg(not(target_arch = "wasm32"))]
        let size = window.inner_size();
        #[cfg(target_arch = "wasm32")]
        let size = {
            use wasm_bindgen::JsCast;
            web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id("game-canvas"))
                .and_then(|el| el.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                .map(|c| winit::dpi::PhysicalSize::new(c.width().max(1), c.height().max(1)))
                .unwrap_or_else(|| window.inner_size())
        };

        // 1. 인스턴스: 플랫폼별 백엔드 선택
        // WASM: WebGPU 어댑터가 maxInterStageShaderComponents 등 미지원 limit을 거부하므로
        //       WebGL2 백엔드(GL)를 강제한다. Cargo.toml의 "webgl" feature로 활성화됨.
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            #[cfg(not(target_arch = "wasm32"))]
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
                    required_limits: {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            wgpu::Limits::default()
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            wgpu::Limits::downlevel_webgl2_defaults()
                        }
                    },
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

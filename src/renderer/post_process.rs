use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

/// 포스트프로세싱 설정. World 리소스로 삽입해 활성화한다.
///
/// ```rust,no_run
/// # use engine::{App, PostProcessConfig};
/// # let mut app = App::new();
/// app.world.insert_resource(PostProcessConfig {
///     enabled: true,
///     vignette_strength: 0.5,
///     ..Default::default()
/// });
/// ```
#[derive(Clone, Copy, Debug)]
pub struct PostProcessConfig {
    /// false이면 포스트프로세스 패스를 완전히 건너뛴다.
    pub enabled: bool,
    /// 비네팅 강도 (0.0=없음, 1.0=가장자리 완전 어두움)
    pub vignette_strength: f32,
    /// 비네팅이 시작되는 반경 (0.0~1.0, 화면 중심 기준)
    pub vignette_radius: f32,
    /// 색수차 강도 (0.0=없음, 0.005 정도가 적절)
    pub chroma_offset: f32,
    /// 블룸 발생 휘도 임계값 (0.0~1.0, 0.7~0.9 권장)
    pub bloom_threshold: f32,
    /// 블룸 밝기 배율
    pub bloom_intensity: f32,
}

impl Default for PostProcessConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            vignette_strength: 0.45,
            vignette_radius: 0.65,
            chroma_offset: 0.003,
            bloom_threshold: 0.75,
            bloom_intensity: 0.4,
        }
    }
}

// GPU로 전송되는 유니폼 구조체 (32바이트, 16B 정렬)
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PostProcessUniforms {
    vignette_strength: f32,
    vignette_radius:   f32,
    chroma_offset:     f32,
    bloom_threshold:   f32,
    bloom_intensity:   f32,
    _pad:              [f32; 3],
}

impl From<&PostProcessConfig> for PostProcessUniforms {
    fn from(c: &PostProcessConfig) -> Self {
        Self {
            vignette_strength: c.vignette_strength,
            vignette_radius:   c.vignette_radius,
            chroma_offset:     c.chroma_offset,
            bloom_threshold:   c.bloom_threshold,
            bloom_intensity:   c.bloom_intensity,
            _pad:              [0.0; 3],
        }
    }
}

/// 씬을 중간 텍스처에 받아 포스트프로세싱 후 최종 스왑체인에 출력하는 렌더러.
pub struct PostProcessRenderer {
    /// 씬을 먼저 그리는 중간 렌더 타겟 텍스처 뷰
    pub target_view: wgpu::TextureView,
    target_texture:  wgpu::Texture,
    /// 현재 중간 텍스처 해상도
    pub width:  u32,
    pub height: u32,
    format:          wgpu::TextureFormat,
    pipeline:        wgpu::RenderPipeline,
    sampler:         wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group:      wgpu::BindGroup,
    uniform_buffer:  wgpu::Buffer,
}

impl PostProcessRenderer {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("post_process shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("shaders/post_process.wgsl").into(),
            ),
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label:        Some("post_process sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter:   wgpu::FilterMode::Linear,
            min_filter:   wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("post_process uniforms"),
            contents: bytemuck::bytes_of(&PostProcessUniforms::zeroed()),
            usage:    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("post_process bgl"),
                entries: &[
                    // binding 0: 씬 텍스처
                    wgpu::BindGroupLayoutEntry {
                        binding:    0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type:    wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled:   false,
                        },
                        count: None,
                    },
                    // binding 1: 샘플러
                    wgpu::BindGroupLayoutEntry {
                        binding:    1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // binding 2: 유니폼
                    wgpu::BindGroupLayoutEntry {
                        binding:    2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty:                 wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size:   None,
                        },
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label:                Some("post_process pipeline layout"),
            bind_group_layouts:   &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label:  Some("post_process pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module:      &shader,
                entry_point: "vs_main",
                buffers:     &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module:      &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format:     surface_format,
                    blend:      Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive:    wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample:  wgpu::MultisampleState::default(),
            multiview:    None,
            cache:        None,
        });

        let (target_texture, target_view) =
            Self::create_target(device, width, height, surface_format);

        let bind_group = Self::create_bind_group(
            device,
            &bind_group_layout,
            &target_view,
            &sampler,
            &uniform_buffer,
        );

        Self {
            target_texture,
            target_view,
            width,
            height,
            format: surface_format,
            pipeline,
            sampler,
            bind_group_layout,
            bind_group,
            uniform_buffer,
        }
    }

    /// 창 크기 변경 시 중간 텍스처를 재생성한다.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let (tex, view) = Self::create_target(device, width, height, self.format);
        self.target_texture = tex;
        self.target_view = view;
        self.width = width;
        self.height = height;
        self.bind_group = Self::create_bind_group(
            device,
            &self.bind_group_layout,
            &self.target_view,
            &self.sampler,
            &self.uniform_buffer,
        );
    }

    /// 유니폼 버퍼를 현재 설정으로 업데이트한다.
    pub fn update_uniforms(&self, queue: &wgpu::Queue, config: &PostProcessConfig) {
        let uni: PostProcessUniforms = config.into();
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uni));
    }

    /// 중간 텍스처 → 최종 스왑체인 뷰로 포스트프로세스 패스를 실행한다.
    pub fn run_pass(
        &self,
        enc: &mut wgpu::CommandEncoder,
        final_view: &wgpu::TextureView,
    ) {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("post_process pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view:           final_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load:  wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set:      None,
            timestamp_writes:         None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        // 풀스크린 삼각형: 버텍스 버퍼 없이 3개 꼭짓점
        pass.draw(0..3, 0..1);
    }

    // ── 내부 헬퍼 ─────────────────────────────────────────────────────────────

    fn create_target(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label:  Some("post_process target"),
            size:   wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count:    1,
            dimension:       wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                 | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        (tex, view)
    }

    fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
        uniform_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label:   Some("post_process bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(sampler) },
                wgpu::BindGroupEntry {
                    binding:  2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        })
    }
}

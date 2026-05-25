use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::camera::Camera;
use crate::ecs::World;

// ─── GPU 파티클 데이터 (64 바이트, 16B 정렬) ──────────────────────────────────
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct GpuParticle {
    pub pos:         [f32; 2],
    pub vel:         [f32; 2],
    pub life:        f32,
    pub max_life:    f32,
    pub size:        f32,
    pub _pad:        f32,
    pub color_start: [f32; 4],
    pub color_end:   [f32; 4],
}

// ─── 컴퓨트 유니폼 ────────────────────────────────────────────────────────────
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct ComputeUniforms {
    dt:   f32,
    _pad: [f32; 3],
}

// ─── 카메라 유니폼 ────────────────────────────────────────────────────────────
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

/// GPU 컴퓨트 셰이더 기반 파티클 렌더러 (네이티브 전용).
///
/// `App`이 내부적으로 관리한다. 사용자는 `GpuParticleEmitter` 컴포넌트만 붙이면 된다.
pub struct GpuParticleRenderer {
    // ── 컴퓨트 파이프라인 ──────────────────────────────────────────────────
    compute_pipeline:    wgpu::ComputePipeline,
    compute_bind_group:  wgpu::BindGroup,
    compute_uniform_buf: wgpu::Buffer,
    // ── 파티클 버퍼 (STORAGE | VERTEX 겸용) ──────────────────────────────
    particle_buf:      wgpu::Buffer,
    particle_capacity: u32,
    // ── 렌더 파이프라인 ────────────────────────────────────────────────────
    render_pipeline:      wgpu::RenderPipeline,
    camera_buf:           wgpu::Buffer,
    camera_bind_group:    wgpu::BindGroup,
    particle_bind_group:  wgpu::BindGroup,
}

impl GpuParticleRenderer {
    /// `capacity` 개의 파티클을 동시에 처리할 수 있는 렌더러를 생성한다.
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat, capacity: u32) -> Self {
        // ── 컴퓨트 셰이더 ────────────────────────────────────────────────
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gpu particle compute"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("shaders/gpu_particle_compute.wgsl").into(),
            ),
        });

        // ── 파티클 버퍼 ──────────────────────────────────────────────────
        let particle_size = (capacity as usize) * std::mem::size_of::<GpuParticle>();
        let particle_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gpu particle buf"),
            size: particle_size as u64,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // ── 컴퓨트 유니폼 ────────────────────────────────────────────────
        let compute_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("gpu particle compute uniforms"),
            contents: bytemuck::bytes_of(&ComputeUniforms { dt: 0.0, _pad: [0.0; 3] }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // ── 컴퓨트 바인드 그룹 레이아웃 ──────────────────────────────────
        let compute_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gpu particle compute bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gpu particle compute bg"),
            layout: &compute_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: compute_uniform_buf.as_entire_binding(),
                },
            ],
        });

        // ── 컴퓨트 파이프라인 ─────────────────────────────────────────────
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("gpu particle compute layout"),
                bind_group_layouts: &[&compute_bgl],
                push_constant_ranges: &[],
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("gpu particle compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        // ── 렌더 셰이더 ──────────────────────────────────────────────────
        let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gpu particle render"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("shaders/gpu_particle_render.wgsl").into(),
            ),
        });

        // ── 카메라 유니폼 (group 0) ───────────────────────────────────────
        let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gpu particle camera buf"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gpu particle camera bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gpu particle camera bg"),
            layout: &camera_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buf.as_entire_binding(),
            }],
        });

        // ── 파티클 버퍼 바인드 그룹 (group 1) ────────────────────────────
        let particle_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("gpu particle render particle bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let particle_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gpu particle render particle bg"),
            layout: &particle_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: particle_buf.as_entire_binding(),
            }],
        });

        // ── 렌더 파이프라인 ───────────────────────────────────────────────
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("gpu particle render layout"),
                bind_group_layouts: &[&camera_bgl, &particle_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("gpu particle render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            compute_pipeline,
            compute_bind_group,
            compute_uniform_buf,
            particle_buf,
            particle_capacity: capacity,
            render_pipeline,
            camera_buf,
            camera_bind_group,
            particle_bind_group,
        }
    }

    /// 새 파티클 데이터를 GPU 버퍼에 업로드한다 (방출 슬롯에 덮어쓴다).
    pub fn upload_particles(&self, queue: &wgpu::Queue, particles: &[GpuParticle], offset: u32) {
        if particles.is_empty() {
            return;
        }
        let byte_offset = offset as u64 * std::mem::size_of::<GpuParticle>() as u64;
        let byte_data = bytemuck::cast_slice(particles);
        if byte_offset + byte_data.len() as u64
            <= self.particle_capacity as u64 * std::mem::size_of::<GpuParticle>() as u64
        {
            queue.write_buffer(&self.particle_buf, byte_offset, byte_data);
        }
    }

    /// 컴퓨트 셰이더로 파티클 위치/수명을 업데이트한다.
    pub fn dispatch_compute(&self, encoder: &mut wgpu::CommandEncoder, queue: &wgpu::Queue, dt: f32) {
        queue.write_buffer(
            &self.compute_uniform_buf,
            0,
            bytemuck::bytes_of(&ComputeUniforms { dt, _pad: [0.0; 3] }),
        );
        let workgroups = (self.particle_capacity + 63) / 64;
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("gpu particle compute pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.compute_pipeline);
        pass.set_bind_group(0, &self.compute_bind_group, &[]);
        pass.dispatch_workgroups(workgroups, 1, 1);
    }

    /// 파티클을 화면에 렌더링한다.
    pub fn render(
        &self,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        world: &World,
        width: u32,
        height: u32,
    ) {
        let fallback = Camera::default();
        let camera = world.resource::<Camera>().unwrap_or(&fallback);
        let view_proj = camera.view_proj(width as f32, height as f32);
        queue.write_buffer(
            &self.camera_buf,
            0,
            bytemuck::bytes_of(&CameraUniform {
                view_proj: view_proj.to_cols_array_2d(),
            }),
        );

        let vertex_count = self.particle_capacity * 6;
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("gpu particle render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.render_pipeline);
        pass.set_bind_group(0, &self.camera_bind_group, &[]);
        pass.set_bind_group(1, &self.particle_bind_group, &[]);
        pass.draw(0..vertex_count, 0..1);
    }

    pub fn capacity(&self) -> u32 {
        self.particle_capacity
    }
}

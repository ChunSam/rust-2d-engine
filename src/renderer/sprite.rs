use std::collections::HashMap;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};
use wgpu::util::DeviceExt;

use crate::camera::Camera;
use crate::components::{AnimationPlayer, Sprite, Transform, UvRect};
use crate::ecs::World;
use crate::renderer::texture::Texture;
use crate::renderer::ui::DrawRect;

// ─── GPU에 올라가는 버텍스 구조체 ─────────────────────────────────────────────
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

// 단위 쿼드: 중심 (0,0), 크기 1×1
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5],
        uv: [0.0, 0.0],
    },
];
const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

// ─── 인스턴스(스프라이트 1개)의 GPU 데이터 ────────────────────────────────────
// 구조: [모델행렬 64B][color 16B][uv_offset 8B][uv_size 8B] = 96B
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4], // offset   0 — 64 bytes
    color: [f32; 4],      // offset  64 — 16 bytes (shader_location 6)
    uv_offset: [f32; 2],  // offset  80 —  8 bytes (shader_location 7)
    uv_size: [f32; 2],    // offset  88 —  8 bytes (shader_location 8)
}

impl InstanceRaw {
    fn from(transform: &Transform, sprite: &Sprite, uv: UvRect) -> Self {
        Self {
            model: transform.to_matrix().to_cols_array_2d(),
            color: sprite.color,
            uv_offset: [uv.u_offset, uv.v_offset],
            uv_size: [uv.u_size, uv.v_size],
        }
    }

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 48,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 64,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: 80,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 88,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

// ─── 카메라 유니폼 ─────────────────────────────────────────────────────────────
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

// ─── 스프라이트 렌더러 ─────────────────────────────────────────────────────────
pub struct SpriteRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    instance_buf: wgpu::Buffer,
    instance_capacity: usize,
    camera_buf: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    texture_layout: wgpu::BindGroupLayout,
    white_texture: Texture,
    texture_cache: HashMap<String, Arc<Texture>>,
    // UI screen-space 렌더링용
    ui_camera_buf: wgpu::Buffer,
    ui_camera_bind_group: wgpu::BindGroup,
    ui_instance_buf: wgpu::Buffer,
    ui_instance_capacity: usize,
}

impl SpriteRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        // ── 셰이더 로드 (컴파일 타임 임베딩) ───────────────────────────────────
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("shaders/sprite.wgsl").into(),
            ),
        });

        // ── 카메라 유니폼 버퍼 ──────────────────────────────────────────────
        let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera uniform"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera layout"),
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
            label: Some("camera bind group"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buf.as_entire_binding(),
            }],
        });

        // ── 텍스처 레이아웃 + 기본 흰색 텍스처 ─────────────────────────────
        let texture_layout = Texture::bind_group_layout(device);
        let white_texture = Texture::white(device, queue, &texture_layout);

        // ── 렌더 파이프라인 ─────────────────────────────────────────────────
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite pipeline layout"),
            bind_group_layouts: &[&camera_layout, &texture_layout],
            push_constant_ranges: &[],
        });
        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
        };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprite pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex_layout, InstanceRaw::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            // wgpu 22 에서 추가된 파이프라인 캐시 필드 — None 이면 캐시 비활성화
            cache: None,
        });

        // ── 정적 버텍스·인덱스 버퍼 ────────────────────────────────────────
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad vertex"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad index"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        // ── 초기 인스턴스 버퍼 (128개 분량 예약) ───────────────────────────
        let capacity = 128;
        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance buffer"),
            size: (capacity * std::mem::size_of::<InstanceRaw>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // ── UI screen-space 카메라 버퍼 + 바인드 그룹 ──────────────────────
        let ui_camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui camera uniform"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let ui_camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ui camera bind group"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: ui_camera_buf.as_entire_binding(),
            }],
        });

        // ── UI 인스턴스 버퍼 (64개 분량 예약) ─────────────────────────────
        let ui_capacity = 64;
        let ui_instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui instance buffer"),
            size: (ui_capacity * std::mem::size_of::<InstanceRaw>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buf,
            index_buf,
            instance_buf,
            instance_capacity: capacity,
            camera_buf,
            camera_bind_group,
            texture_layout,
            white_texture,
            texture_cache: HashMap::new(),
            ui_camera_buf,
            ui_camera_bind_group,
            ui_instance_buf,
            ui_instance_capacity: ui_capacity,
        }
    }

    /// PNG 파일을 GPU에 로드하고 내부 캐시에 저장한다.
    ///
    /// 같은 경로를 두 번 호출하면 첫 번째 로드 결과를 그대로 사용한다 (중복 로드 방지).
    pub fn load_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, path: &str) {
        if !self.texture_cache.contains_key(path) {
            let tex = Texture::from_path(device, queue, &self.texture_layout, path);
            self.texture_cache.insert(path.to_string(), Arc::new(tex));
        }
    }

    /// 매 프레임: ECS World에서 스프라이트를 수집해 렌더링한다.
    ///
    /// # z-order 구현 한계
    /// 현재는 **텍스처 그룹 내부**에서만 z 오름차순 정렬을 수행한다.
    /// 서로 다른 텍스처 그룹 간의 z 충돌은 해결되지 않는다
    /// (예: 텍스처 A의 z=5 스프라이트가 텍스처 B의 z=0 스프라이트 뒤에 그려질 수 있음 —
    ///  group_order가 등록 순서 기준이므로).
    /// 전체 z-order 정렬은 후속 작업(모든 인스턴스를 한 번에 z 정렬 후 연속된 같은 텍스처를
    /// 묶어 multi-draw)에서 구현할 예정이다.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        world: &World,
        width: u32,
        height: u32,
    ) {
        // ── 카메라: ECS 리소스에서 Camera 를 읽어 view_proj 를 계산한다 ───
        //   Camera 리소스가 없으면 기본값(좌상단 직교 투영)으로 폴백한다.
        let fallback = Camera::default(); // Camera 가 없을 때만 사용 (이론상 없어야 정상)
        let camera = world.resource::<Camera>().unwrap_or(&fallback);
        let view_proj = camera.view_proj(width as f32, height as f32);
        let cam = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
        };
        queue.write_buffer(&self.camera_buf, 0, bytemuck::bytes_of(&cam));

        // ── 텍스처별로 스프라이트를 그룹핑 ────────────────────────────────
        // group_order: HashMap은 순서를 보장하지 않으므로 삽입 순서를 별도로 기록한다.
        // 각 그룹에 (z, InstanceRaw) 튜플을 저장해 그룹 내 z 정렬에 사용한다.
        let mut groups: HashMap<Option<String>, Vec<(f32, InstanceRaw)>> = HashMap::new();
        let mut group_order: Vec<Option<String>> = Vec::new();

        for (entity, sprite) in world.query::<Sprite>() {
            if let Some(transform) = world.get::<Transform>(entity) {
                // AnimationPlayer가 없으면 텍스처 전체(UvRect::FULL)를 사용한다.
                let uv = world
                    .get::<AnimationPlayer>(entity)
                    .map(|p| p.current_uv())
                    .unwrap_or(UvRect::FULL);

                let key = sprite.texture.clone();
                if !groups.contains_key(&key) {
                    group_order.push(key.clone());
                }
                // z 값과 함께 저장해 그룹 내 정렬에 활용한다.
                groups
                    .entry(key)
                    .or_default()
                    .push((transform.z, InstanceRaw::from(transform, sprite, uv)));
            }
        }
        if groups.is_empty() {
            return;
        }

        // ── 각 그룹 내부에서 z 오름차순 안정 정렬 ────────────────────────
        // z 가 같으면 등록 순서를 유지한다 (stable sort).
        // z = 0.0(기본값) 만 있으면 순서 변화 없음 → 플랫포머 데모 회귀 없음.
        for group in groups.values_mut() {
            group.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        }

        // ── 모든 그룹을 하나의 버퍼에 순서대로 쓴다 ───────────────────────
        let all_instances: Vec<InstanceRaw> = group_order
            .iter()
            .flat_map(|k| groups[k].iter().map(|(_, raw)| *raw))
            .collect();

        if all_instances.len() > self.instance_capacity {
            self.instance_capacity = all_instances.len().next_power_of_two();
            self.instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("instance buffer"),
                size: (self.instance_capacity * std::mem::size_of::<InstanceRaw>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&self.instance_buf, 0, bytemuck::cast_slice(&all_instances));

        // ── 렌더 패스 ───────────────────────────────────────────────────────
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("sprite pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // 배경색은 App이 먼저 Clear
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.camera_bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);

        // 텍스처 그룹마다 바인드 그룹을 교체하고 해당 인스턴스만 그린다.
        let instance_size = std::mem::size_of::<InstanceRaw>() as u64;
        let mut base: u64 = 0;
        for key in &group_order {
            let instances = &groups[key]; // Vec<(f32, InstanceRaw)>
            let byte_start = base * instance_size;
            let byte_end = byte_start + instances.len() as u64 * instance_size;

            // 캐시에 없거나 texture: None이면 흰색 텍스처(색상 스프라이트)로 폴백
            let bind_group = match key {
                Some(path) => self
                    .texture_cache
                    .get(path)
                    .map(|t| &t.bind_group)
                    .unwrap_or(&self.white_texture.bind_group),
                None => &self.white_texture.bind_group,
            };

            pass.set_bind_group(1, bind_group, &[]);
            pass.set_vertex_buffer(1, self.instance_buf.slice(byte_start..byte_end));
            pass.draw_indexed(0..INDICES.len() as u32, 0, 0..instances.len() as u32);

            base += instances.len() as u64;
        }
    }

    /// 화면 고정(screen-space) UI 사각형을 렌더링한다.
    ///
    /// 스프라이트 패스 직후, 텍스트 패스 직전에 호출한다.
    /// `rects`는 `UiQueue`에서 drain한 슬라이스를 전달한다.
    pub fn render_ui_rects_from_slice(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        rects: &[DrawRect],
        width: u32,
        height: u32,
    ) {
        if rects.is_empty() {
            return;
        }

        // 화면 좌상단 (0,0), 우하단 (width,height) 직교 투영
        let screen_proj = Mat4::orthographic_rh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);
        let cam = CameraUniform {
            view_proj: screen_proj.to_cols_array_2d(),
        };
        queue.write_buffer(&self.ui_camera_buf, 0, bytemuck::bytes_of(&cam));

        // z 오름차순 안정 정렬
        let mut sorted: Vec<&DrawRect> = rects.iter().collect();
        sorted.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal));

        // DrawRect → InstanceRaw 변환 (중심 좌표 기준)
        let instances: Vec<InstanceRaw> = sorted
            .iter()
            .map(|rect| {
                let cx = rect.x + rect.w * 0.5;
                let cy = rect.y + rect.h * 0.5;
                let model = Mat4::from_scale_rotation_translation(
                    Vec3::new(rect.w, rect.h, 1.0),
                    Quat::IDENTITY,
                    Vec3::new(cx, cy, 0.0),
                );
                InstanceRaw {
                    model: model.to_cols_array_2d(),
                    color: rect.color,
                    uv_offset: [0.0, 0.0],
                    uv_size: [1.0, 1.0],
                }
            })
            .collect();

        // 인스턴스 버퍼 용량 초과 시 동적 재할당
        if instances.len() > self.ui_instance_capacity {
            self.ui_instance_capacity = instances.len().next_power_of_two();
            self.ui_instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ui instance buffer"),
                size: (self.ui_instance_capacity * std::mem::size_of::<InstanceRaw>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&self.ui_instance_buf, 0, bytemuck::cast_slice(&instances));

        // UI 렌더 패스 (LoadOp::Load 로 스프라이트 위에 합성)
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ui pass"),
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

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.ui_camera_bind_group, &[]);
        pass.set_bind_group(1, &self.white_texture.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
        pass.set_vertex_buffer(1, self.ui_instance_buf.slice(..));
        pass.draw_indexed(0..INDICES.len() as u32, 0, 0..instances.len() as u32);
    }
}

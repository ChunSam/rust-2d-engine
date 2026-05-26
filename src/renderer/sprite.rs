use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};
use wgpu::util::DeviceExt;

use crate::animation::player::UvRect;
use crate::asset::AssetServer;
use crate::atlas::AtlasSprite;
use crate::camera::Camera;
use crate::components::{Sprite, Transform};
use crate::ecs::World;
use crate::hierarchy::GlobalTransform;
use crate::renderer::texture::Texture;
use crate::renderer::ui::DrawRect;
use crate::resources::CullConfig;

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

    fn from_global(gt: &GlobalTransform, sprite: &Sprite, uv: UvRect) -> Self {
        Self {
            model: gt.to_matrix().to_cols_array_2d(),
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
    // ── ShaderMaterial 커스텀 렌더링용 ────────────────────────────────────────
    sprite_shader: wgpu::ShaderModule,
    camera_layout: wgpu::BindGroupLayout,
    surface_format: wgpu::TextureFormat,
    params_layout: wgpu::BindGroupLayout,
    mat_instance_buf: wgpu::Buffer,
    mat_instance_capacity: usize,
    custom_pipelines: HashMap<u64, wgpu::RenderPipeline>,
    params_buffers: HashMap<u32, (wgpu::Buffer, wgpu::BindGroup)>,
    /// RenderTarget bind_group 캐시 (키 = RenderTarget 이름)
    rt_cache: HashMap<String, Arc<wgpu::BindGroup>>,
}

impl SpriteRenderer {
    pub fn texture_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_layout
    }

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        // ── 셰이더 로드 (컴파일 타임 임베딩) ───────────────────────────────────
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/sprite.wgsl").into()),
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

        // ── ShaderMaterial: params 유니폼 레이아웃 (@group(2)) ──────────────
        let params_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("material params layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // ── ShaderMaterial: 인스턴스 버퍼 (머티리얼 엔티티 수만큼 동적 재할당) ──
        let mat_capacity = 16usize;
        let mat_instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("material instance buffer"),
            size: (mat_capacity * std::mem::size_of::<InstanceRaw>()) as u64,
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
            sprite_shader: shader,
            camera_layout,
            surface_format: format,
            params_layout,
            mat_instance_buf,
            mat_instance_capacity: mat_capacity,
            custom_pipelines: HashMap::new(),
            params_buffers: HashMap::new(),
            rt_cache: HashMap::new(),
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

    /// CPU-side `ImageAsset`을 GPU 텍스처로 업로드한다 (비동기 로딩 완료 시 사용).
    ///
    /// 같은 경로가 이미 캐시에 있으면 재업로드한다 (비동기 완료 → 마젠타 폴백 교체).
    pub fn load_texture_from_image(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &str,
        asset: &crate::asset::ImageAsset,
    ) {
        use crate::renderer::texture::Texture;
        let tex = Texture::from_image_asset(device, queue, &self.texture_layout, asset, Some(path));
        self.texture_cache.insert(path.to_string(), Arc::new(tex));
    }

    /// 캐시를 무효화하고 파일에서 GPU 텍스처를 강제 재로드한다 (핫 리로딩용).
    pub fn reload_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, path: &str) {
        self.texture_cache.remove(path);
        let tex = Texture::from_path(device, queue, &self.texture_layout, path);
        self.texture_cache.insert(path.to_string(), Arc::new(tex));
        log::info!("텍스처 핫 리로드: {path}");
    }

    /// RenderTarget bind_group을 스프라이트 렌더러에 등록한다.
    ///
    /// `Sprite::texture`에 `key` 문자열을 지정하면 해당 RT 텍스처로 렌더링된다.
    pub fn register_render_target(&mut self, key: &str, bg: Arc<wgpu::BindGroup>) {
        self.rt_cache.insert(key.to_string(), bg);
    }

    /// 매 프레임: ECS World에서 스프라이트를 수집해 렌더링한다.
    ///
    /// # z-order
    /// 모든 스프라이트를 z 오름차순으로 전역 정렬한 뒤, 연속으로 같은 텍스처를 쓰는
    /// 구간마다 draw call을 한 번씩 발행한다. 텍스처가 섞이더라도 z 값이 정확히 반영된다.
    #[allow(clippy::too_many_arguments)]
    /// 스프라이트를 렌더한다.
    ///
    /// `layer_mask`가 0이면 모든 레이어를 렌더한다.
    /// `layer_mask`가 0이 아니면 `RenderLayer(n)` 엔티티 중 `(layer_mask >> n) & 1 == 1`인 것만 렌더한다.
    /// `RenderLayer`가 없는 엔티티는 레이어 0으로 취급한다.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        world: &World,
        width: u32,
        height: u32,
        layer_mask: u32,
    ) -> crate::resources::RenderStats {
        let mut stats = crate::resources::RenderStats::default();
        // ── 카메라: ECS 리소스에서 Camera 를 읽어 view_proj 를 계산한다 ───
        let fallback = Camera::default();
        let camera = world.resource::<Camera>().unwrap_or(&fallback);
        let view_proj = camera.view_proj(width as f32, height as f32);
        let cam = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
        };
        queue.write_buffer(&self.camera_buf, 0, bytemuck::bytes_of(&cam));

        // ── 컬링 설정 + 가시 영역 ──────────────────────────────────────────
        let cull = world
            .resource::<CullConfig>()
            .copied()
            .unwrap_or_default();
        let (vmin, vmax) = camera.visible_rect(width as f32, height as f32);

        // 회전을 고려한 보수적 AABB 교차 판정 헬퍼.
        // |cos θ|·w/2 + |sin θ|·h/2 공식으로 회전 후 AABB 반폭을 계산한다.
        let is_visible = |pos: glam::Vec2, scale: glam::Vec2, rotation: f32| -> bool {
            if !cull.frustum_culling {
                return true;
            }
            let sin_r = rotation.sin().abs();
            let cos_r = rotation.cos().abs();
            let hw = cos_r * scale.x * 0.5 + sin_r * scale.y * 0.5;
            let hh = sin_r * scale.x * 0.5 + cos_r * scale.y * 0.5;
            pos.x + hw >= vmin.x
                && pos.x - hw <= vmax.x
                && pos.y + hh >= vmin.y
                && pos.y - hh <= vmax.y
        };

        let is_above_lod = |scale: glam::Vec2| -> bool {
            if cull.min_pixel_size <= 0.0 {
                return true;
            }
            let px_w = scale.x * camera.zoom;
            let px_h = scale.y * camera.zoom;
            px_w.min(px_h) >= cull.min_pixel_size
        };

        // ── 전체 스프라이트 수집: (layer, tex_key, z, InstanceRaw) ──────
        // GlobalTransform이 있으면 계층 합성 결과를 사용하고, 없으면 Transform으로 fallback.
        // RenderLayer(i32)가 없으면 0 으로 취급한다.
        let mut sprites: Vec<(i32, String, f32, InstanceRaw)> = Vec::new();
        for (entity, sprite) in world.query::<Sprite>() {
            let uv = world.get::<UvRect>(entity).copied().unwrap_or(UvRect::FULL);
            let layer = world
                .get::<crate::components::RenderLayer>(entity)
                .map(|l| l.0)
                .unwrap_or(0);
            // layer_mask 필터: 0 = 전체 허용, 비 0 = 해당 비트만 렌더
            if layer_mask != 0 {
                let bit = (layer.clamp(0, 31)) as u32;
                if (layer_mask >> bit) & 1 == 0 {
                    continue;
                }
            }
            // image_handle이 있으면 그 경로를 우선 사용, 없으면 texture 경로 사용
            let tex_key = sprite
                .image_handle
                .as_ref()
                .map(|h| h.path().to_string())
                .or_else(|| sprite.texture.clone())
                .unwrap_or_default();
            if let Some(gt) = world.get::<GlobalTransform>(entity) {
                if !is_visible(gt.position, gt.scale, gt.rotation) {
                    stats.sprites_culled += 1;
                    continue;
                }
                if !is_above_lod(gt.scale) {
                    stats.sprites_culled += 1;
                    continue;
                }
                sprites.push((layer, tex_key, gt.z, InstanceRaw::from_global(gt, sprite, uv)));
            } else if let Some(transform) = world.get::<Transform>(entity) {
                if !is_visible(transform.position, transform.scale, transform.rotation) {
                    stats.sprites_culled += 1;
                    continue;
                }
                if !is_above_lod(transform.scale) {
                    stats.sprites_culled += 1;
                    continue;
                }
                sprites.push((
                    layer,
                    tex_key,
                    transform.z,
                    InstanceRaw::from(transform, sprite, uv),
                ));
            }
        }
        // ── AtlasSprite 수집: (index, color, atlas handle) 을 먼저 collect ──
        // query 이터레이터 borrow를 끊은 뒤 AssetServer 와 GlobalTransform 을 읽는다.
        let atlas_entries: Vec<(
            crate::ecs::Entity,
            u32,
            [f32; 4],
            crate::asset::Handle<crate::atlas::TextureAtlas>,
        )> = world
            .query::<AtlasSprite>()
            .map(|(e, s)| (e, s.index, s.color, s.atlas.clone()))
            .collect();

        if !atlas_entries.is_empty() {
            if let Some(server) = world.resource::<AssetServer>() {
                for (entity, index, color, atlas_handle) in &atlas_entries {
                    if let Some(atlas) = server.get_atlas(atlas_handle) {
                        let uv = atlas.uv_rect(*index);
                        let tex_key = atlas.texture_path().to_string();
                        let layer = world
                            .get::<crate::components::RenderLayer>(*entity)
                            .map(|l| l.0)
                            .unwrap_or(0);
                        if let Some(gt) = world.get::<GlobalTransform>(*entity) {
                            if !is_visible(gt.position, gt.scale, gt.rotation) {
                                stats.sprites_culled += 1;
                                continue;
                            }
                            if !is_above_lod(gt.scale) {
                                stats.sprites_culled += 1;
                                continue;
                            }
                            sprites.push((
                                layer,
                                tex_key,
                                gt.z,
                                InstanceRaw {
                                    model: gt.to_matrix().to_cols_array_2d(),
                                    color: *color,
                                    uv_offset: [uv.u_offset, uv.v_offset],
                                    uv_size: [uv.u_size, uv.v_size],
                                },
                            ));
                        } else if let Some(tr) = world.get::<Transform>(*entity) {
                            if !is_visible(tr.position, tr.scale, tr.rotation) {
                                stats.sprites_culled += 1;
                                continue;
                            }
                            if !is_above_lod(tr.scale) {
                                stats.sprites_culled += 1;
                                continue;
                            }
                            sprites.push((
                                layer,
                                tex_key,
                                tr.z,
                                InstanceRaw {
                                    model: tr.to_matrix().to_cols_array_2d(),
                                    color: *color,
                                    uv_offset: [uv.u_offset, uv.v_offset],
                                    uv_size: [uv.u_size, uv.v_size],
                                },
                            ));
                        }
                    }
                }
            }
        }

        // ── (layer, tex_key, z) 기준 안정 정렬 ──────────────────────────
        // layer가 같으면 tex_key 기준으로 배칭 → 같은 텍스처는 항상 인접.
        // tex_key가 같으면 z 오름차순으로 정렬.
        sprites.sort_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.cmp(&b.1))
                .then_with(|| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
        });
        stats.sprites_rendered = sprites.len() as u32;

        // ── 일반 스프라이트 렌더 패스 ────────────────────────────────────────
        if !sprites.is_empty() {
            let all_instances: Vec<InstanceRaw> = sprites.iter().map(|(_, _, _, raw)| *raw).collect();

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

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("sprite pass"),
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
            pass.set_bind_group(0, &self.camera_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);

            // ── 연속된 같은 텍스처 구간마다 draw call 1회 ──────────────
            // (layer, tex_key) 기준으로 정렬됐으므로 같은 텍스처는 항상 연속.
            // 텍스처가 바뀔 때만 bind group을 교체한다.
            let instance_size = std::mem::size_of::<InstanceRaw>() as u64;
            let mut i = 0usize;
            while i < sprites.len() {
                let run_key = &sprites[i].1;
                let run_start = i;
                while i < sprites.len() && &sprites[i].1 == run_key {
                    i += 1;
                }
                let run_len = i - run_start;
                let byte_start = run_start as u64 * instance_size;
                let byte_end = byte_start + run_len as u64 * instance_size;

                let bind_group = if run_key.is_empty() {
                    &self.white_texture.bind_group
                } else if let Some(rt_bg) = self.rt_cache.get(run_key.as_str()) {
                    rt_bg.as_ref()
                } else {
                    self.texture_cache
                        .get(run_key.as_str())
                        .map(|t| &t.bind_group)
                        .unwrap_or(&self.white_texture.bind_group)
                };

                pass.set_bind_group(1, bind_group, &[]);
                pass.set_vertex_buffer(1, self.instance_buf.slice(byte_start..byte_end));
                pass.draw_indexed(0..INDICES.len() as u32, 0, 0..run_len as u32);
                stats.draw_calls += 1;
            }
            // pass drops here → encoder 해방
        }

        // ── ShaderMaterial 렌더 패스 (별도 패스, z-sort 독립) ───────────────
        self.render_materials(device, queue, view, encoder, world, width, height);

        stats
    }

    /// 소스 해시를 키로 커스텀 파이프라인을 컴파일·캐싱한다.
    /// 렌더 패스가 열리기 **전에** 호출해야 한다.
    fn compile_material_pipeline(
        &mut self,
        device: &wgpu::Device,
        hash: u64,
        frag_source: &str,
    ) {
        let frag_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("custom material frag"),
            source: wgpu::ShaderSource::Wgsl(frag_source.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("material pipeline layout"),
            bind_group_layouts: &[
                &self.camera_layout,
                &self.texture_layout,
                &self.params_layout,
            ],
            push_constant_ranges: &[],
        });
        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
        };
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("material pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &self.sprite_shader,
                entry_point: "vs_main",
                buffers: &[vertex_layout, InstanceRaw::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: self.surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        self.custom_pipelines.insert(hash, pipeline);
    }

    /// [`crate::material::ShaderMaterial`] 컴포넌트를 가진 엔티티를 렌더링한다.
    ///
    /// 일반 스프라이트 패스 **이후**, UI 패스 **이전**에 호출된다.
    /// z-sort는 머티리얼 엔티티끼리만 적용된다 (일반 스프라이트와 인터리브 없음).
    #[allow(clippy::too_many_arguments)]
    fn render_materials(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        world: &World,
        _width: u32,
        _height: u32,
    ) {
        use crate::material::ShaderMaterial;

        // 1. 머티리얼 엔티티 데이터 수집 (borrow 해방 전)
        struct MatEntry {
            entity: crate::ecs::Entity,
            hash: u64,
            frag_source: String,
            params: [f32; 4],
            instance: InstanceRaw,
            tex_key: Option<String>,
            z: f32,
        }

        let mat_ids: Vec<(crate::ecs::Entity, u64, String, [f32; 4])> = world
            .query::<ShaderMaterial>()
            .map(|(e, mat)| {
                let mut h = std::collections::hash_map::DefaultHasher::new();
                mat.frag_source.hash(&mut h);
                (e, h.finish(), mat.frag_source.clone(), mat.params)
            })
            .collect();

        if mat_ids.is_empty() {
            return;
        }

        let mut entries: Vec<MatEntry> = Vec::new();
        for (e, hash, frag_source, params) in mat_ids {
            let uv = world.get::<UvRect>(e).copied().unwrap_or(UvRect::FULL);
            let sprite = match world.get::<Sprite>(e) {
                Some(s) => s,
                None => continue,
            };
            let tex_key = sprite
                .image_handle
                .as_ref()
                .map(|h| h.path().to_string())
                .or_else(|| sprite.texture.clone());

            if let Some(gt) = world.get::<GlobalTransform>(e) {
                entries.push(MatEntry {
                    entity: e, hash, frag_source, params, tex_key,
                    instance: InstanceRaw::from_global(gt, sprite, uv),
                    z: gt.z,
                });
            } else if let Some(tr) = world.get::<Transform>(e) {
                entries.push(MatEntry {
                    entity: e, hash, frag_source, params, tex_key,
                    instance: InstanceRaw::from(tr, sprite, uv),
                    z: tr.z,
                });
            }
        }

        if entries.is_empty() {
            return;
        }

        // 2. z 오름차순 안정 정렬
        entries.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal));

        // 3. 파이프라인 컴파일 (렌더 패스 열기 전)
        let hashes: Vec<(u64, String)> = entries
            .iter()
            .map(|e| (e.hash, e.frag_source.clone()))
            .collect();
        for (hash, src) in &hashes {
            if !self.custom_pipelines.contains_key(hash) {
                self.compile_material_pipeline(device, *hash, src);
            }
        }

        // 4. params 유니폼 버퍼 생성/갱신 (렌더 패스 열기 전)
        for entry in &entries {
            let eid = entry.entity.0;
            if !self.params_buffers.contains_key(&eid) {
                let buf = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("material params buf"),
                    size: 16, // vec4<f32>
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("material params bind group"),
                    layout: &self.params_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buf.as_entire_binding(),
                    }],
                });
                self.params_buffers.insert(eid, (buf, bg));
            }
            let (buf, _) = &self.params_buffers[&eid];
            queue.write_buffer(buf, 0, bytemuck::cast_slice(&entry.params));
        }

        // 5. 인스턴스 데이터 일괄 업로드 (렌더 패스 열기 전)
        let instances: Vec<InstanceRaw> = entries.iter().map(|e| e.instance).collect();
        if instances.len() > self.mat_instance_capacity {
            self.mat_instance_capacity = instances.len().next_power_of_two();
            self.mat_instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("material instance buffer"),
                size: (self.mat_instance_capacity * std::mem::size_of::<InstanceRaw>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&self.mat_instance_buf, 0, bytemuck::cast_slice(&instances));

        // 6. 렌더 패스 — 엔티티별로 커스텀 파이프라인 적용
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("material pass"),
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

        pass.set_bind_group(0, &self.camera_bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        pass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);

        let instance_size = std::mem::size_of::<InstanceRaw>() as u64;
        for (i, entry) in entries.iter().enumerate() {
            let pipeline = &self.custom_pipelines[&entry.hash];
            pass.set_pipeline(pipeline);

            let tex_bg = entry
                .tex_key
                .as_ref()
                .and_then(|k| {
                    self.rt_cache
                        .get(k)
                        .map(|bg| bg.as_ref())
                        .or_else(|| self.texture_cache.get(k).map(|t| &t.bind_group))
                })
                .unwrap_or(&self.white_texture.bind_group);
            pass.set_bind_group(1, tex_bg, &[]);

            let (_, params_bg) = &self.params_buffers[&entry.entity.0];
            pass.set_bind_group(2, params_bg, &[]);

            let byte_start = i as u64 * instance_size;
            let byte_end = byte_start + instance_size;
            pass.set_vertex_buffer(1, self.mat_instance_buf.slice(byte_start..byte_end));
            pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }
    }

    /// 화면 고정(screen-space) UI 사각형을 렌더링한다.
    ///
    /// 스프라이트 패스 직후, 텍스트 패스 직전에 호출한다.
    /// `rects`는 `UiQueue`에서 drain한 슬라이스를 전달한다.
    #[allow(clippy::too_many_arguments)]
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

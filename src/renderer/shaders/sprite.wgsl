// ─── 카메라 유니폼 (직교 투영 행렬) ───────────────────────────────────────────
struct Camera {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: Camera;

// ─── 버텍스 입력 (쿼드 1개 공유) ────────────────────────────────────────────
struct Vertex {
    @location(0) position: vec2<f32>,
    @location(1) uv:       vec2<f32>,
};

// ─── 인스턴스 입력 (스프라이트 1개마다) ─────────────────────────────────────
struct Instance {
    @location(2) row0:      vec4<f32>,   // 모델 행렬 열(column) 0
    @location(3) row1:      vec4<f32>,   // 모델 행렬 열 1
    @location(4) row2:      vec4<f32>,   // 모델 행렬 열 2
    @location(5) row3:      vec4<f32>,   // 모델 행렬 열 3
    @location(6) color:     vec4<f32>,   // RGBA 색상
    @location(7) uv_offset: vec2<f32>,   // 스프라이트시트 UV 시작점
    @location(8) uv_size:   vec2<f32>,   // 프레임 UV 크기
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv:    vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(v: Vertex, inst: Instance) -> VertexOutput {
    let model = mat4x4<f32>(inst.row0, inst.row1, inst.row2, inst.row3);
    var out: VertexOutput;
    out.clip_pos = camera.view_proj * model * vec4<f32>(v.position, 0.0, 1.0);
    // 쿼드 UV [0,1] → 스프라이트시트 서브영역으로 변환
    out.uv    = inst.uv_offset + v.uv * inst.uv_size;
    out.color = inst.color;
    return out;
}

// ─── 텍스처 (스프라이트 없으면 흰색 1×1 픽셀 사용) ──────────────────────────
@group(1) @binding(0) var t_sprite: texture_2d<f32>;
@group(1) @binding(1) var s_sprite: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_sprite, s_sprite, in.uv) * in.color;
}

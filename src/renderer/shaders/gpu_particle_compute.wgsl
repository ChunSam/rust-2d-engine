// GPU 파티클 컴퓨트 셰이더 — 파티클 수명 + 위치 업데이트

struct Particle {
    pos:        vec2<f32>,  //  0: 위치
    vel:        vec2<f32>,  //  8: 속도
    life:       f32,        // 16: 현재 수명 (0이면 비활성)
    max_life:   f32,        // 20: 최대 수명
    size:       f32,        // 24: 크기 (픽셀)
    _pad:       f32,        // 28: 패딩
    color_start: vec4<f32>, // 32: 시작 색상
    color_end:   vec4<f32>, // 48: 종료 색상
}                           // 64 bytes total

struct ComputeUniforms {
    dt:    f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform>             uniforms:  ComputeUniforms;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= arrayLength(&particles) { return; }
    var p = particles[i];
    if p.life <= 0.0 { return; }
    p.life -= uniforms.dt;
    p.pos  += p.vel * uniforms.dt;
    particles[i] = p;
}

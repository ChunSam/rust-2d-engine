// GPU 파티클 렌더 셰이더 — 파티클 버퍼를 읽어 쿼드 출력

struct Camera {
    view_proj: mat4x4<f32>,
}

struct Particle {
    pos:        vec2<f32>,
    vel:        vec2<f32>,
    life:       f32,
    max_life:   f32,
    size:       f32,
    _pad:       f32,
    color_start: vec4<f32>,
    color_end:   vec4<f32>,
}

@group(0) @binding(0) var<uniform>        camera:    Camera;
@group(1) @binding(0) var<storage, read>  particles: array<Particle>;

struct VOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0)       color:    vec4<f32>,
}

// 파티클당 6 버텍스 (삼각형 2개로 쿼드)
@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VOut {
    let pi = vi / 6u;
    let qi = vi % 6u;
    var out: VOut;

    if pi >= arrayLength(&particles) {
        out.clip_pos = vec4<f32>(0.0, 0.0, 10.0, 1.0);
        out.color    = vec4<f32>(0.0);
        return out;
    }

    let p = particles[pi];

    if p.life <= 0.0 {
        out.clip_pos = vec4<f32>(0.0, 0.0, 10.0, 1.0); // 클립 밖으로
        out.color    = vec4<f32>(0.0);
        return out;
    }

    let hs = p.size * 0.5;
    // 쿼드 6 버텍스 오프셋 (CCW)
    var offs = array<vec2<f32>, 6>(
        vec2<f32>(-hs, -hs),
        vec2<f32>( hs, -hs),
        vec2<f32>( hs,  hs),
        vec2<f32>(-hs, -hs),
        vec2<f32>( hs,  hs),
        vec2<f32>(-hs,  hs),
    );

    let world_pos = vec4<f32>(p.pos + offs[qi], 0.0, 1.0);
    out.clip_pos = camera.view_proj * world_pos;

    let t     = clamp(1.0 - p.life / max(p.max_life, 0.0001), 0.0, 1.0);
    out.color = mix(p.color_start, p.color_end, t);
    return out;
}

@fragment
fn fs_main(@location(0) color: vec4<f32>) -> @location(0) vec4<f32> {
    return color;
}

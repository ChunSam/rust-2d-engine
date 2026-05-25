// ─── 포스트프로세싱: 비네팅 · 색수차 · 근사 블룸 ─────────────────────────────

struct Uniforms {
    vignette_strength: f32,   // 0=없음, 1=강한 어두움
    vignette_radius:   f32,   // 어두워지기 시작하는 반경 (0~1)
    chroma_offset:     f32,   // 색수차 강도
    bloom_threshold:   f32,   // 블룸 발생 휘도 임계값
    bloom_intensity:   f32,   // 블룸 밝기 배율
    brightness:        f32,   // 밝기 오프셋 (-1~1, 0=원본)
    contrast:          f32,   // 대비 배율 (1=원본)
    saturation:        f32,   // 채도 배율 (1=원본, 0=흑백)
}

@group(0) @binding(0) var scene_tex:     texture_2d<f32>;
@group(0) @binding(1) var scene_sampler: sampler;
@group(0) @binding(2) var<uniform>  u: Uniforms;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0)       uv:  vec2<f32>,
}

// 버텍스 버퍼 없이 풀스크린 삼각형 생성
@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VOut {
    // 세 꼭짓점: (-1,3), (-1,-1), (3,-1) — NDC 쿼드 전체를 덮는다
    var clip = array<vec2<f32>, 3>(
        vec2<f32>(-1.0,  3.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
    );
    var uvs = array<vec2<f32>, 3>(
        vec2<f32>(0.0, -1.0),
        vec2<f32>(0.0,  1.0),
        vec2<f32>(2.0,  1.0),
    );
    var out: VOut;
    out.pos = vec4<f32>(clip[vi], 0.0, 1.0);
    out.uv  = uvs[vi];
    return out;
}

fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    let uv = in.uv;

    // ── 색수차 (Chromatic Aberration) ──────────────────────────────────────
    // 중심에서 멀수록 RGB 채널을 방사형으로 다른 위치에서 샘플링
    let center = uv - 0.5;
    let dist   = length(center);
    let dir    = select(normalize(center), vec2<f32>(1.0, 0.0), dist < 0.0001);
    let shift  = dir * dist * u.chroma_offset;

    let r_samp = textureSample(scene_tex, scene_sampler, uv + shift);
    let g_samp = textureSample(scene_tex, scene_sampler, uv);
    let b_samp = textureSample(scene_tex, scene_sampler, uv - shift);
    var color  = vec4<f32>(r_samp.r, g_samp.g, b_samp.b, g_samp.a);

    // ── 근사 블룸 (4-tap threshold blur) ───────────────────────────────────
    // 인접 픽셀에서 임계값 이상의 밝기만 뽑아 합산
    let texel = 1.0 / vec2<f32>(f32(textureDimensions(scene_tex).x),
                                 f32(textureDimensions(scene_tex).y));
    let spread = 4.0;
    var bloom = vec3<f32>(0.0);
    let tap_offsets = array<vec2<f32>, 4>(
        vec2<f32>( texel.x,  0.0) * spread,
        vec2<f32>(-texel.x,  0.0) * spread,
        vec2<f32>( 0.0,  texel.y) * spread,
        vec2<f32>( 0.0, -texel.y) * spread,
    );
    for (var i = 0; i < 4; i++) {
        let s = textureSample(scene_tex, scene_sampler, uv + tap_offsets[i]).rgb;
        let lum = luminance(s);
        let w   = max(0.0, lum - u.bloom_threshold) / max(0.001, 1.0 - u.bloom_threshold);
        bloom  += s * w;
    }
    color = vec4<f32>(color.rgb + bloom * u.bloom_intensity * 0.25, color.a);

    // ── 비네팅 (Vignette) ──────────────────────────────────────────────────
    let vig_dist = length(center) / max(u.vignette_radius, 0.001);
    let vignette = 1.0 - smoothstep(1.0 - u.vignette_strength, 1.0, vig_dist);
    color = vec4<f32>(color.rgb * vignette, color.a);

    // ── 색상 그레이딩 (Color Grading) ──────────────────────────────────────
    var graded = color.rgb;
    // 밝기
    graded = graded + vec3<f32>(u.brightness);
    // 대비 (0.5 중심)
    graded = (graded - 0.5) * u.contrast + 0.5;
    // 채도 (luminance 보존)
    let lum = dot(graded, vec3<f32>(0.2126, 0.7152, 0.0722));
    graded = mix(vec3<f32>(lum), graded, u.saturation);
    color = vec4<f32>(clamp(graded, vec3<f32>(0.0), vec3<f32>(1.0)), color.a);

    return color;
}

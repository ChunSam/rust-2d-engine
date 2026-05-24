/// 엔티티에 붙이면 내장 스프라이트 셰이더 대신 커스텀 fragment 셰이더를 사용한다.
///
/// vertex 입력은 표준 스프라이트와 동일하므로 fragment 셰이더만 작성하면 된다.
///
/// ## 커스텀 셰이더에서 사용 가능한 바인딩
///
/// ```wgsl
/// @group(1) @binding(0) var t_sprite: texture_2d<f32>;
/// @group(1) @binding(1) var s_sprite: sampler;
/// @group(2) @binding(0) var<uniform> params: vec4<f32>;  // ShaderMaterial::params
///
/// struct VertexOutput {
///     @builtin(position) clip_pos: vec4<f32>,
///     @location(0) uv:    vec2<f32>,
///     @location(1) color: vec4<f32>,
/// };
/// @fragment
/// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> { ... }
/// ```
///
/// ## 사용 예
///
/// ```text
/// world.add_component(e, ShaderMaterial {
///     frag_source: include_str!("shaders/dissolve.wgsl").to_string(),
///     params: [total_time, progress, 0.0, 0.0],
/// });
/// ```
///
/// `params`는 시스템 내에서 `world.get_mut::<ShaderMaterial>(e)` 로 매 프레임 갱신할 수 있다.
pub struct ShaderMaterial {
    /// WGSL fragment shader source (`fs_main` 진입점 필수).
    pub frag_source: String,
    /// 셰이더로 전달되는 float 파라미터 4개.
    /// WGSL 쪽에서는 `@group(2) @binding(0) var<uniform> params: vec4<f32>` 로 수신한다.
    /// 관례: `[time, intensity, user_x, user_y]`
    pub params: [f32; 4],
}

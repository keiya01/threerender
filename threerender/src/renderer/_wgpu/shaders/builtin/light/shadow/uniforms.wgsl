struct UniformShadow {
    projection: mat4x4<f32>,
    // 0 or 1
    use_shadow: u32,
    opacity: f32,

    // 0: Normal shadow
    // 1: PCSS
    shadow_type: u32,

    // For PCSS
    light_uv: f32,
    near_plane: f32,    
}

struct UniformShadow {
    projection: mat4x4<f32>,
    // 0 or 1
    use_shadow: u32,
}

fn fetch_shadow(light_id: u32, homogeneous_coords: vec4<f32>, t_shadow: texture_depth_2d_array, sampler_shadow: sampler_comparison) -> f32 {
    if (homogeneous_coords.w <= 0.0) {
        return 1.0;
    }

    // compensate for the Y-flip difference between the NDC and texture coordinates
    let flip_correction = vec2<f32>(0.5, -0.5);
    // compute texture coordinates for shadow lookup
    let proj_correction = 1.0 / homogeneous_coords.w;
    let light_local = homogeneous_coords.xy * flip_correction * proj_correction + vec2<f32>(0.5, 0.5);
    // do the lookup, using HW PCF and comparison
    return textureSampleCompareLevel(t_shadow, sampler_shadow, light_local, i32(light_id), homogeneous_coords.z * proj_correction);
}



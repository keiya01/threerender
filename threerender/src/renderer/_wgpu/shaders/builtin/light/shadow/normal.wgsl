fn normal_shadow(light_id: u32, homogeneous_coords: vec4<f32>, t_shadow: texture_depth_2d_array, sampler_shadow: sampler, sampler_shadow_comparison: sampler_comparison) -> f32 {
    let coords = shadow_coords(homogeneous_coords);
    let shadow = textureSampleCompareLevel(t_shadow, sampler_shadow_comparison, coords.xy, i32(light_id), coords.z);
    return shadow;
}

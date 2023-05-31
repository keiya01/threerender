fn calc_hemisphere_light(position: vec4<f32>, local_normal: vec3<f32>, ulight: UniformLight, light_position: vec4<f32>) -> HemisphereLight {
    let light_normal = normalize(calc_affine_normal(light_position, position).xyz);

    let hemisphere = (dot(local_normal, light_normal) + 1.0) * 0.5;
    let ambient: vec4<f32> = mix(ulight.hemisphere.ground_color, ulight.hemisphere.sky_color, hemisphere);

    var h: HemisphereLight;
    h.color = ambient;

    return h;
}

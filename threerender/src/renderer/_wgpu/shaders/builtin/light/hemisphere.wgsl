struct HemisphereLight {
    color: vec4<f32>,
    normal: vec3<f32>,
}

fn calc_hemisphere_light(model: mat4x4<f32>, position: vec4<f32>, normal: vec3<f32>, ulight: UniformLight) -> HemisphereLight {
    let world_normal = normalize(model * vec4<f32>(normal, 0.0)).xyz;
    let light_position = vec4<f32>(ulight.position, 1.0);

    let light_normal = normalize(calc_affine_normal(light_position, position).xyz);

    let sky = vec3<f32>(0.0, 1.0, 0.0);
    let hemisphere = (dot(normal, sky) + 1.0) * 0.5;
    let ambient: vec4<f32> = mix(ulight.hemisphere.ground_color, ulight.hemisphere.sky_color, hemisphere);

    var h: HemisphereLight;
    h.color = ambient;
    h.normal = light_normal;

    return h;
}

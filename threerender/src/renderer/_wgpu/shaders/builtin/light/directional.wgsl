fn calc_directional_light(world_normal: vec3<f32>, position: vec4<f32>, ulight: UniformLight) -> DirectionalLight {
    let light_position = vec4<f32>(ulight.position, 1.0);
    let blightness = vec4<f32>(ulight.brightness);

    let light_normal = normalize(calc_affine_normal(light_position, position).xyz);
    let diffuse = max(dot(world_normal, light_normal), 0.0) * ulight.color * blightness;

    var d: DirectionalLight;
    d.color = diffuse;
    d.normal = light_normal;

    return d;
}

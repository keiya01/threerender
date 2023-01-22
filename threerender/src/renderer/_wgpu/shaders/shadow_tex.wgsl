// This is used for baking shadow as texture.

// TODO: Create shared shader to share moules like light module, camera module, entity module, etc.

@group(0)
@binding(0)
var<uniform> umodel: mat4x4<f32>;

struct Entity {
    transform: mat4x4<f32>,
    color: vec4<f32>,
}

@group(1)
@binding(0)
var<uniform> entity: Entity;

// For shadow
@vertex
fn vs_bake(
    @location(0) position: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) _tex_coords: vec2<f32>,
) -> @builtin(position) vec4<f32> {
    return umodel * entity.transform * position;
}

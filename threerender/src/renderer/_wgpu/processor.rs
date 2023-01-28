use shader_processor::ShaderProcessor;

#[derive(Default)]
pub(super) struct ShaderProcessOption {
    pub(super) use_texture: bool,
}

pub(super) fn process_shader(shader: &str, option: ShaderProcessOption) -> String {
    let mut s = ShaderProcessor::from_shader_str(shader);

    // light builtin modules
    let p = make_builtin_path("light/directional");
    s.insert_builtin("light::directional", &p);
    let p = make_builtin_path("light/mod");
    s.insert_builtin("light", &p);

    // shadow builtin modules
    let p = make_builtin_path("shadow/shared");
    s.insert_builtin("shadow::shared", &p);
    let p = make_builtin_path("shadow/directional");
    s.insert_builtin("shadow::directional", &p);
    let p = make_builtin_path("shadow/mod");
    s.insert_builtin("shadow", &p);

    // envs
    s.insert_env("USE_TEXTURE", option.use_texture);

    s.process()
}

fn make_builtin_path(path: &str) -> String {
    let common_path =
        std::fs::canonicalize("./threerender/src/renderer/_wgpu/shaders/builtin/").unwrap();
    let common_path = common_path.to_str().unwrap();
    format!("{}/{}", common_path, path)
}

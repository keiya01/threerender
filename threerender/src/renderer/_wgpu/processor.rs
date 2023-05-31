use shader_processor::{EnvType, ShaderProcessor};

#[derive(Default)]
pub(super) struct ProcessOption {
    pub(super) has_texture: bool,
    pub(super) max_light_num: u32,
}

pub(super) struct Processor<'a>(ShaderProcessor<'a>);

impl<'a> Processor<'a> {
    pub(super) fn new(shader: &'a str) -> Self {
        Self(ShaderProcessor::from_shader_str(shader))
    }

    pub(super) fn process(&mut self, option: ProcessOption) -> String {
        // math builtin modules
        let s = &mut self.0;

        let p = make_builtin_path("math/affine");
        s.insert_builtin("math::affine", p);
        let p = make_builtin_path("math/mod");
        s.insert_builtin("math", p);

        // light builtin modules
        let p = make_builtin_path("light/uniforms");
        s.insert_builtin("light::uniforms", p);
        let p = make_builtin_path("light/types");
        s.insert_builtin("light::types", p);
        let p = make_builtin_path("light/directional");
        s.insert_builtin("light::directional", p);
        let p = make_builtin_path("light/hemisphere");
        s.insert_builtin("light::hemisphere", p);
        let p = make_builtin_path("light/mod");
        s.insert_builtin("light", p);

        // reflection
        let p = make_builtin_path("reflection");
        s.insert_builtin("reflection", p);

        // shadow builtin modules
        let p = make_builtin_path("light/shadow/uniforms");
        s.insert_builtin("light::shadow::uniforms", p);
        let p = make_builtin_path("light/shadow/directional");
        s.insert_builtin("light::shadow::directional", p);
        let p = make_builtin_path("light/shadow/mod");
        s.insert_builtin("light::shadow", p);

        // condition envs
        s.insert_env("HAS_TEXTURE", EnvType::Bool(option.has_texture));
        s.insert_env("MAX_LIGHT_NUM", EnvType::Number(option.max_light_num));

        s.process().unwrap()
    }
}

fn make_builtin_path(path: &str) -> String {
    let common_path =
        std::fs::canonicalize("./threerender/src/renderer/_wgpu/shaders/builtin/").unwrap();
    let common_path = common_path.to_str().unwrap();
    format!("{common_path}/{path}")
}

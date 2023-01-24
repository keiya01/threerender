//! This is processor to extend shader.  
//! For example, WGSL is not supporting `#ifdef` statement to define condition or `#include` statement to use other module.  
//! So this library provides processor for support of these statement.  
//!
//! As described above, this library provides the following statement.
//!
//! - `#include ./path/to/shader`
//! - `#include builtin::light`
//! - `#ifdef ENABLE_SHADOW`
//!
//! `#include` provides feature to include specified file or builtin module.
//!
//! `ifdef` provides feature to define condition by constance variable of boolean type. We can use like bellow.
//!
//! ```wgsl
//! #ifdef ENABLE_SHADOW
//! var<uniform> shadow: vec4<f32>;
//! #else
//! var<uniform> color: vec4<f32>;
//! #end
//! ```
//!
//! Made for [`threerender`](https://github.com/keiya01/threerender/).
//!
//! ```rust
//! let mut p = ShaderProcessor::from_shader_str(SHADER_INPUT);
//! p.insert_env("USE_SHADOW", true);
//! p.insert_builtin("light", "./assets/builtin/light");
//! p.process(); // Return processed string
//! ```
//!
//! ## Feature flags
//! - The `wgsl` feature enables processor for WGSL.
//!

#[cfg(feature = "wgsl")]
mod wgsl;
#[cfg(feature = "wgsl")]
pub use wgsl::*;

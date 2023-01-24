# Shader Processor

This is processor to extend shader.  
For example, WGSL is not supporting `#ifdef` statement to define condition or `#include` statement to use other module.  
So this library provides processor for support of these statement.  

As described above, this library provides the following statement.

- `#include ./path/to/shader`
- `#include builtin::light`
- `#ifdef ENABLE_SHADOW`

`#include` provides feature to include specified file or builtin module.

`ifdef` provides feature to define condition by constance variable of boolean type. We can use like bellow.

```wgsl
#ifdef ENABLE_SHADOW
var<uniform> shadow: vec4<f32>;
#else
var<uniform> color: vec4<f32>;
#end
```

## TODO

- module caching

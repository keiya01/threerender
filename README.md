# threerender

**CAUTION: Currently, this is POC, and in development, so not production ready.**
**If you interest this project, you can see [examples dir](/examples)**

## Overview

This is a simple 3D rendering engine.
This will target providing feature of fundamental for 3D development.

This is similar to Three.js, but this will provide more low level API. 

## Examples

You can try this project by [examples](/examples).

You can run these examples by below command.

```sh
cargo run -p examples_{PROJECT_NAME}
```

## Status

- [x] 3D entities
  - [x] Square
    - [x] 2D texture rendering
  - [x] Sphere
    - [x] 2D texture rendering
- [x] 2D entities
  - [x] Lines
  - [x] Plane
    - [x] 2D texture rendering
  - [x] Triangle
- [x] Camera
- [ ] Light
  - [x] Directional light
  - [ ] Spot light
  - [ ] Point light
  - [x] Hemisphere light
- [ ] Shadow
  - [x] Directional shadow
  - [ ] Opacity
  - [ ] Point light shadow
  - [ ] Spot light shadow
- [x] Multi light/shadow
- [ ] Reflection rate for entity
- [x] 2D texture
- [ ] 3D texture(3D model)
- [ ] Font
- [x] Override shader
- [ ] Extendable shader by user
- [ ] Performance improvement
    - [ ] dirty check
    - [ ] multi threading
    - [ ] Optimize image loading. Eg. We should not draw image if image is not visible.
- [ ] Integration with 2D library like egui
- [ ] Web support
- [ ] Convenient Math API for 3D development
- [ ] OpenGL Support
- [ ] Light variation

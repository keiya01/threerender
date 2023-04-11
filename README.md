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

## Development road map

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
- [x] Reflection rate for entity
- [x] 2D texture
- [x] Override shader
- [ ] Model loader
- [ ] Normal mapping
- [ ] Model transparency
- [ ] Extendable shader by user
- [ ] Custom render process by implementing trait
  - [ ] Shadow rendering(PSM, LSPSM, Blur)
  - [ ] Ray tracing
- [ ] Performance improvement
    - [ ] dirty check
    - [ ] Optimize multi object like cloning object and transfer vertex more efficiently
    - [ ] Mip map for texture
      - Provide some functionality to be able to define mip map.
    - [ ] multi threading
    - [ ] Optimize image loading. Eg. We should not draw image if image is not visible.
    - [ ] Level of details for polygon(LOD)
- [ ] Font
- [ ] Integration with 2D library like egui
- [ ] Web support
- [ ] Convenient Math API for 3D development
- [ ] OpenGL Support

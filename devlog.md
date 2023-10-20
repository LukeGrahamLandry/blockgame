## Humble beginnings

- https://sotrh.github.io/learn-wgpu/ (MIT License, Copyright (c) 2020 Benjamin Hansen)

Ripped the guts out of another of my projects experimenting with webgpu (based on learn-wgpu).
Got a nice start with the boilerplate, so I can render triangles and move around my camera.

## Inspiration 

- https://github.com/superjer/tinyc.games/tree/main/blocko-game (MIT License, Copyright (c) 2016 Jer Wilson)

I want to make a voxel game like minecraft. Inspired by how small/understandable blocko-game is (~4000 lines!). 
Only took 800 years to figure out how to build it, I assume, because apple has gone on an anti-opengl 
crusade sometime in the last 8 years. 

- `release-singlethreaded: gcc -DTERRAIN_THREAD=0 -O3 -o bin main.c -lSDL2 -framework OpenGL`
  - Note that's not called `clang`, but it's also definitely not gcc. It's whatever version of clang apple decided to alias `gcc` to, instead of whatever version of clang I last let homebrew shove in my path I guess. 
- Changed a bunch of SDL includes to directly "../macos/SDL2.framework/Headers/SDL.h" because I can't be bothered to figure out how to do it properly (-I?)
- glTexStorage3D doesn't exist. https://stackoverflow.com/a/34237328
- #define NO_OMPH and #define GL_SILENCE_DEPRECATION
- remove GLEW_NVX_gpu_memory_info in test.c

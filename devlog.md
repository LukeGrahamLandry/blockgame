## Block textures

Made an atlas texture loader. Take a bunch of little textures png files and write them into one big image as a grid. 
So a whole chunk can still be one draw call and each vertex has a different UV into that atlas. 
Can save the atlas to a file to make sure that it looks right. 

## Rendering a chunk

Simple representation of a 16x16x16 chunk. Then just have a bunch of those and build a world. 
Any time chunk data changes, recalculate the mesh and send it to the gpu. 
Got to the point of rendering a little triangle everywhere I want a block. 

Start with most inefficient way of adding cubes to the mesh to make sure it works. 
Think of the 8 corners of the unit cube, then add each face as two triangles.
So 36 vertices in the buffer per cube. But most are duplicates and even faces of adjacent cubes are redundant.
But it works! Have the shader colour by position (r = x%16/16, etc.) so I can tell what's going on. 
Each vertex added offset based on the block's LocalPos. Each chunk transformed based on its ChunkPos.

Demo chunk: full bottom layer plus 8 on next (264 cubes).

- Naive: 9504 vertices, 9504 indices.
- Reuse vertices within single cube: 2112 vertices, 9504 indices.
- No index top/right/far if adjacent is full: 2112 vertices, 6390 indices.
- ^ bottom/left/close (needs overflow check): 2112 vertices, 3564 indices.
  - Can check that it's working by flying inside a solid part and seeing no internal faces. 
  - Currently only pulls this trick within a chunk and assumes the edges are required. 
- Only make vertex if needed: 2097 vertices, 3564 indices.
  - Bad on this test with no inner solid bits because partial blocks omit faces but still need all the vertices for other faces
  - For a solid chunk this goes from (32768 vertices, 9216 indices) to (5768 vertices, 9216 indices).

The code for each face is a very ugly copy-paste that I don't really know how to make better at this point.

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

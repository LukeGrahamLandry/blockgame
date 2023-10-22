## Code generation for fun and profit (Oct 21)

My code for defining blocks is starting to feel a bit insane. Need to change things in many places to add a new block. 

First thought was I'll just write declarative functions for making blocks. 
They'd both load the textures into an atlas for runtime and generate a file with constants of all the block ids and uvs. 
Then you compile again, and now you have that file, so other code can reference those constants and since its deterministic,
it will line up with the runtime information from calling those functions this time. 

- You can't just `include!("target/gen.rs")`, that path is relative to the src directory.
- When using bindgen, they suggest `include!(concat!(env!("OUT_DIR"), "/gen.rs"));` but `error: environment variable `OUT_DIR` not defined at compile time`. 
- You need to have a buildscript if you want OUT_DIR to exist. 
- It prints zero as "0" but you can't assign a float field to that, need to print "0f32".
- RustRover can't cope with this at all. It can't find the generated file, so it highlights as errors even though it compiles. 
I'm hoping that's because I'm using and old OUT_DIR and the hash changed so would be fixed by using a build script properly.
- There's also a chicken and egg problem. If you can't compile without the generated file, then you can never generate the file the first time. 

So maybe that's just a bad idea. Maybe split into two crates and have a buildscript that depends on the first to generate code for the second. 

- RustRover can't find the common crate from build script even though it compiles. 
- RustRover still can't find the generated file in OUT_DIR.

But fixed both by deleting the `.idea` folder and restarting, so it re-indexed. Maybe that would have worked before.
Seems like I don't have to redo that when the file changes. Maybe only when adding new ones?

Now adding blocks is easy. 
- Solid: load the texture to the atlas, add its indexes to the list. Choose the right indexes for each shape: cube, grass, or pillar. 
- Custom: add the rendering function with the same name to the list.

During that process, save the full atlas texture and include those bytes in the final binary as well, 
so don't need to carefully find the assets folder at runtime anymore. 

No need to manually keep track of indexes. Tiles and uvs just live in constants with nice names. 
I can even generate tests that lookup the render function of each tile and check that the pointer matches the function with the same name. 
Just a nice little sanity check thing.

## Polymorphic rendering (Oct 20)

Want to have things like grass and wheat that aren't solid blocks. 

What I really want is a vtable on every tile, so I can have a custom render method, but I don't want to pay for 
it all the time since I suspect most blocks will just be solid cubes (there's a lot of stone in the world).

Instead, use the first bit in the id to say if its solid. If unset, use the rest as an index of UVs like before. 
If set, use the rest as an index into a table of function pointers. Call that to do the actual rendering. 

If I do more models, I should probably spend another bit on "just lookup a model to render" instead of writing a new function for each one.
Have to remember that these functions aren't called every frame, they just build the mesh and can only use textures from 
the atlas. So anything animated will need some other system. 

Problem: transparency doesn't work, it's just black. Just needed to change render pipeline fragment to have `blend: Some(BlendState::ALPHA_BLENDING)`. 
- https://stackoverflow.com/questions/72333404/wgpu-doesnt-render-with-alpha-channels

## Block textures (Oct 20)

Made an atlas texture loader. Take a bunch of little textures png files and write them into one big image as a grid. 
So a whole chunk can still be one draw call and each vertex has a different UV into that atlas. 
Can save the atlas to a file to make sure that it looks right. 

Just using the tile id as an index into the list of uvs in the atlas texture.
I can send the textures to the gpu just fine but alas my clever vertex reuse makes it a pain to texture the faces properly.
So undo that. Still skip internal faces but each lone cube has 24 vertices (vertex only added if its face is needed). 
Now demo chunk (2376 vertices, 3564 indices) and full chunk (6144 vertices, 9216 indices), not really that much of a hit.
But also vertices are bigger now because they have a texture UV.

They were super blurry but just needed to change my sampler to `mag_filter: FilterMode::Nearest`. 
Far away blocks still wierd while I'm moving. Need to investigate mip-mapping? 

If you fly inside a solid part you get z-fighting on the sides where its next to another solid chunk. 
Because I don't bother culling those faces. But it's fine since normal play can't ever see them. 

## Rendering a chunk (Oct 20)

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

## Humble beginnings (Oct 19)

- https://sotrh.github.io/learn-wgpu/ (MIT License, Copyright (c) 2020 Benjamin Hansen)

Ripped the guts out of another of my projects experimenting with webgpu (based on learn-wgpu).
Got a nice start with the boilerplate, so I can render triangles and move around my camera.

## Inspiration (Oct 19)

- https://github.com/superjer/tinyc.games/tree/main/blocko-game (MIT License, Copyright (c) 2016 Jer Wilson)

I want to make a voxel game like minecraft. Inspired by how small blocko-game is (~4000 lines!). 
Only took 800 years to figure out how to build it, I assume, because apple has gone on an anti-opengl 
crusade sometime in the last 8 years. 

- `release-singlethreaded: gcc -DTERRAIN_THREAD=0 -O3 -o bin main.c -lSDL2 -framework OpenGL`
  - Note that's not called `clang`, but it's also definitely not gcc. It's whatever version of clang apple decided to alias `gcc` to, instead of whatever version of clang I last let homebrew shove in my path I guess. 
- Changed a bunch of SDL includes to directly "../macos/SDL2.framework/Headers/SDL.h" because I can't be bothered to figure out how to do it properly (-I?)
- glTexStorage3D doesn't exist. https://stackoverflow.com/a/34237328
- #define NO_OMPH and #define GL_SILENCE_DEPRECATION
- remove GLEW_NVX_gpu_memory_info in test.c

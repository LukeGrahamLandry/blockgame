## A little lua with your rust (Oct 22)

- https://github.com/khvzak/mlua

Idea: write all the game logic in lua.

A few reasons this is interesting. 
- I don't know how to model a world with entity's other than the OOP way where they all have pointers to each other and rust really hates that. 
- You can have an in game console with very powerful introspection about the world state, and those console commands just be a lua program. Instead of making a wierd thing that's not quite a programming language like minecraft's commands
- Making mods would be really easy. Not useful at my scale but aesthetically pleasing. 

Luajit makes it really easy to call c functions. Just write the prototype and it magically knows the calling convention. 
You can even access struct fields.

My thought is you have lua own all the state (entities, chunks, etc.) and just call into rust when it needs to regenerate a chunk mesh or render a zombie or whatever. 
Having rust own the world struct is kinda awkward because you have to figure out some way of passing handles to chunks into lua, 
and it doesn't fix my mutable aliasing problem, so you just end up with a bunch of unsafe code anyway. 
But still need to be careful that the array of blocks in a chunk is a c struct, so it's packed. 
I can use LightUserData to pass a raw pointer to lua, but then I can't use it as the jit ffi type without a cast which gets ugly pretty fast. 
So lua just owning everything seems easiest for now. 

The compiler removes functions if you don't call them (even with no_mangle)
- Can't put `#[used]` on a function. It's only for static variables. 
- Tried putting `#[used]` on a static array of function pointers, didn't work.
- Tried thing in build script and `#![feature(export_executable_symbols)]` but didn't compile. Needs nightly.
  - https://rust-lang.github.io/rfcs/2841-export-executable-symbols.html
- Tried `assert_ne!(&generate_chunk as *const _ as usize, 0);`
- Tried `rustflags = ["-C", "link-args=-rdynamic"]` in `.cargo/config` 
  - https://stackoverflow.com/questions/43712979/how-to-export-a-symbol-from-a-rust-executable
- Works: create an array of function pointers at runtime. Don't even need to do anything with it. But can't `let _ = ...`

Current system is rust calls a lua function every tick (currently every frame), lua does whatever logic it wants and calls back into rust when it wants something rendered differently. 
Also, I think I'd rather do my world generation in rust so the first time lua tries to read a chunk, it calls back into rust. 
Need to remember to batch the mesh re-generation, so you only do it once per changed chunk at the end of the tick. For testing, the set_block function in lua is always asking for a new mesh. 

There's an unpleasant duplication of little utility functions for working with BlockPos, etc. 
And I'm offended by using a garbage-collected table for my 3 numbers, so currently I'm just passing them around separately. 
I might need to get over that. Or maybe commit to exposing all my rust stuff over cffi. I think the ffi structs are still aren't really value objects (just their fields are?).
So they make more sense for big things where you can shove lots into the one GC handle.

Need to think about how to implement this on wasm (luajit doesn't support it, and neither does mlua wrapper at all https://github.com/khvzak/mlua/issues/23).
Default lua (non-jit) still gives ways for the host to define callbacks. mlua has nice wrappers that do the conversion of arguments to rust types, 
so they're not even that bad to write, the functions just also take a &Lua first parameter. But that still doesn't work with wasm-unknown-unknown, maybe emnscripten?
But I recall that sucks for making wgpu work. 

Maybe it would be smarter to just find a tiny embeddable javascript, so you get it for free in the browser. 
Browser js can easily call wasm c abi functions (but can't seamlessly access fields of structs). 
Idk if there exists a tiny JS jit with good c ffi. And I kinda find lua cute in way JS doesn't because it's too popular of a language. 
Maybe that's a bit deranged and I just need to get over it. Another tempting option is transpiling my lua to JS. 
That would be a pleasing combination of my compiler obsession with my graphics obsession. 

There's still some code experimenting with having chunks owned by rust that I need to remove, but I feel I should commit this before I break something. 

## Look at me; I am the JS now (Oct 21)

- https://gfx-rs.github.io/2020/04/21/wgpu-web.html

I want it to run in the browser as well. 

- Don't use `pollster`, just annotate an async function for wasm-bindgen start.
- Can't use `std::Instant`, there's an `instant` library that has the same interface but works on both platforms. 
- Still blank screen but no error except "Don't mind me, just using exceptions as control flow."
- Tried to set up the logger to put wgpu errors in the browser console (new deps: console_error_panic_hook, console_log, log) but still nothing.
- Small brain! I was just being dumb and in rust adding a canvas element as a child of my html canvas and rendering into the second one, so you couldn't see it. Just make it a div. 
- Now it renders (tiny and in the corner). 
- Not getting keyboard input. Need to have the canvas grab focus. 
- Can't just set the canvas size to the window size with css. Need wgpu to know the size so the browser doesn't blurrily upscale. So need to get the dimensions in rust with web_sys. 

Problem: the colours are much darker in the web version. Idk what's up with that. 

I also dislike that I can't run wasm-bindgen from a cargo build script so now I have an ugly shell thing for the web version. 

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
Seems like I don't always have to redo that when the file changes, but it's unreliable. Clicking into it sometimes gets you an old version. 

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

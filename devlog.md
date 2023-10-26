## Fixing Transparency 

- https://webgpu.github.io/webgpu-samples/samples/A-buffer (Copyright 2019 WebGPU Samples Contributors, BSD 3-Clause License)

## Type stripping (Oct 25)

Ideas for how to do struct fields. 
- Generate rust get & set methods for every field and the compiler can emit those for field accesses.
  - Since my parser (in theory) knows the ABI for field sizes/padding, I could do field accesses based on offsets and only have a few helpers for reading primitive types out of raw pointers. 
- Use js typed array views into wasm memory. Relies on knowing ABI for field layout. 
- If I supported dynamic index lookups from metatables like real lua I could set that up with either of the above when the object was created and not need full static types for any c structs. 
  - Would need to unwrap the pointer when passing to wasm but that's safer anyway than just assuming it's fine. 
  - Feels a little sad to have double boxing of pointers. 

The first two require static types whenever accessing struct fields. 
Which is kinda something I want anyway. The lua parser I'm using supports luau (which is roblox's typed lua dialect I guess). 
I don't really want to use their vm, it seems to not have luajit's nice ffi which is the whole point of this. 
It's pretty easy to write something that strips the type annotations out of the ast and the library can print it out again for running with luajit. 
Also, there's a VS Code plugin for it so better autocomplete which is nice. I much prefer the typescript-like syntax over the comment driven Intellij plugin I was using before. 
- https://luau-lang.org/

## Lua wasm ffi (Oct 25)

- https://github.com/LukeGrahamLandry/seesea

Node supports wasm just like the browser so testing locally is still easy. 
Setup a little test rust library that builds to wasm and then load it in node so generated js can access. 
I don't want to fight with getting luajit to find the library so just write a rust wrapper that links to it, loads a lua file, and uses mlua to run it like the game will. 

Conveniently, I have another project where I wrote part of a c compiler and I *think* its parser is enough for dealing with `ffi.cdef`. 
I want this to only be a build time dependency, but might need to ship it to the browser if I eventually want a nice lua console in the game. 

Calling wasm function doesn't even need to parse any c headers. 
JS gives you the same transparent argument conversion as luajit, so can just do it lexically. 
Need to go through and make sure they handle weird type conversions the same at some point. 

luajit handles allocating and garbage collecting memory created by `ffi.new` so I need to do that myself for wasm. 
- JS doesn't give you gc hooks for destructors, but I can manually drop that's a no-op when running in luajit. 
- Have rust alloc and drop functions, I don't want to generate new constructors for each type so just have the compiler say how big the struct is. 
  - I'm zero initializing which technically isn't safe for all rust types. 
  - Need to make sure I'm not breaking any rules about alignment by pretending I'm just asking for bytes. 
  - My c parser should be doing the right struct padding, but I should really be generating tests that it's the same as whatever abi `#[repr(C)]` means in wasm. 
- Can't just call libc in wasm but found a rust allocator library that advertises its wasm support. 
  - https://github.com/rustwasm/wee_alloc
- The rust allocator api wants a "Layout" for both malloc and free, so it needs to know the size. How does libc free just need the pointer? 
For now, I'll just allocate an extra word and write the size before the pointer, so I can read it back when you free it to use the right layout
Would be cleverer if I trusted my typing enough to have the compiler implicitly pass the right size of the struct based on static typing when you call drop.
- Have some rudimentary leak detection by counting how many allocations minus de-allocations I do. 
- So that works for allocating structs and passing pointers to rust. More thought required for how to access fields from lua. 

## Lua in the browser (Oct 24)

- https://github.com/Kampfkarren/full-moon

A few options to consider
- Find a lua vm that will run in wasm. But couldn't find one that lets you do ffi as seamlessly as luajit. Also, it seems sad to bring my own shitty interpreter that can't even jit when V8 is just sitting there. 
- Write the thing in js instead and find a little js vm for native. Seems unlikely to find a tiny jit that also lets you call c functions without writing boilerplate. 
Even in the browser, js can't seamlessly access fields of wasm structs.
- Transpile lua to JS! More work but means more control over the output. Feels like I could get the ffi stuff to work if I find a c parser, and maybe use a typed dialect of lua, so I get more information.

Start with the simple thing for math. 
- It gets a bit wierd when they don't define things (like mod for negative numbers) the same, so I have to call a little function. 
Will also have to redo all this if I want to use lua's operator overloading. 

For using `require` to get modules, I'll just intercept that and define my own object that forwards to JS stuff. 
- Hard to test trig functions because they print high precision floats differently. 
- Hard to test random numbers. 
- Why doesn't `{ ...Math, pi: Math.PI }` work? Is Math not a real object somehow>? Manually assigning individual fields works. 
- Need to figure out how to use js object prototypes. I want my math object to forward to the js one, so I don't need to define everything (both languages have floor, sin, cos, etc). 
But maybe it's better to do it manually, so I have a hard error for things I didn't test yet. 
- Should cleverly only include the parts of my little runtime thingy that are actually used by the program.

There's something wierd with the library I'm using. The last statement in a block isn't in the main Vec of statements. 
I guess it's a type safety thing because only some statements (jumps) are allowed to be terminators, and they can't appear in the middle of a block. 
So maybe it's actually reasonable.

I'm using JS iterables for pairs/ipairs. Works for my uses where they're consumed by a for loop. 
The Lua docs have a more specific paragraph about what they're supposed to return so might have to revisit this later if I ever run into problems. 
- https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Iterators_and_Generators

Multiple return values can just use JS list destructuring. 

Have a simple debug mode that inserts checks like only doing arithmetic on numbers, no undefined argument values, etc.
Makes the generated code ugly and want to have a way to toggle this off for release builds eventually.
Also need to expand it so comparisons (>, >=, <=, <) are checked for coercion because lua won't auto convert strings to numbers (even though it does for normal math). 

Currently, it enforces that all arguments are used when calling functions. Both languages allow optional arguments but lua sets them to nil (which I'm representing as null) and js sets them to undefined. 
But maybe I just need to use undefined for nil instead because failed object lookups are supposed to return nil in lua. 
Actually yeah, I made that change. Now I can't check that you pass the right number of arguments since passing nil and nothing look the same. 
The code for that in rust looked ugly anyway tho so not a total loss. 
Should add proper arity checking once I'm looking at type info. 

Should think of a less generic name. I'm sure lua2js is already taken. 

Implementing a tiny part of metatables by using the `__index` field as the object's prototype (so that only supports a table, not any function like real lua). 
No other operator overloading stuff yet. I'd have to do that in the compiler since JS doesn't have operator overloading. Something like every binary operator instead calls 
`LuaHelper.add` or whatever and looks up what to do in the meta table. I think I'd want to wait until I'm doing stuff with type hints so numbers could still compile to raw JS ops. 
- https://gist.github.com/oatmealine/655c9e64599d0f0dd47687c1186de99f

Have to check that they don't use js keywords as variable names. And by "they" I mean me because I did exactly that with my `new` function for making a new table with a certain metatable. 

JS has variadic functions very similar to lua, just need to do a cringe rearranging of the array to make it a one indexed table. 

Method calls are weird because lua lets you decide at the call-site instead of at the function declaration. 
So I can't just use JS `this` inside the function because any function in a table can be called both ways. 
Can't just naively `a:name(b, c)` -> `a.name(a, b, c)` because that would evaluate the receiver expression twice, and it might have side effects. 
And I can't just put the object in a variable because I need method calls to be an expression. 
So for now I have a helper method you call like `call(a, "name", b, c)` that then calls the method with the variadic args as an array `receiver[method_name].apply(null, [receiver, ...args])`. 
Which is annoying because it doesn't look normal and its probably 0.01% harder to jit but eh, it works. 

Multiline strings (`[[ whatever ]]`) are interesting because its natural to u se backticks for them but that allows you to easily escape to js expression land (`${alert("xss lol")}`). 
Suppose really that just revealed how trivial it is for the rest of the code too since I'm mostly doing text based substitutions. 
You could just call eval without defining it first and bam, you referred to the js one. 
Admittedly that's not a meaningful threat model for this project, but fun food for thought anyway. 

So I think that's all the features of lua I'm using so far, took ~a day, but now I'm ready to tackle ffi. 

## Random ticks & cleanup (Oct 22)

Since I'm generating the list of block id constants for rust anyway, it's super easy to have it spit out a lua file as well. 
Maybe you'd rather generate that off the rust data at runtime, so it doesn't take up space in the binary but eh, code isn't free either so this is fine for now. 

The default Debug printing of lua errors by unwrap doesn't format it on multiple lines, so unwrap_or_else to panic with Display. 

Profiled it. Doing the update mesh at the end of the frame and only if a block had actually changed to something new make. 
Time in update_mesh went from ~30% to ~2%. Back to barely being able to see my code. (The test was with no vsync and uselessly setting the same block to stone every tick). 

Started having a naming convention of bx/lx/cx for x coord of BlocPos/LocalPos/ChunkPos which feels very fragile. 

Did random ticks for growing wheat. Every frame has a chance to pick a random chunk, if a chunk is chosen, some number of its blocks get a chance to tick.

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

I also dislike that I can't run wasm-bindgen from a cargo build script, so now I have an ugly shell thing for the web version. 

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

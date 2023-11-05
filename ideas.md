## Lua extensions

- comptime
- fancy two-way type inference
- compile to c

JS does let you intercept field accesses. Could use that for __index function in metatable. 
- https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Proxy

Js lets you write getters and setters. 
Could use generate those for accessing the wasm struct fields. 
But then the values wouldn't just be the raw pointer anymore. 
- https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/get
- https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/set

## Fixing Transparency

- https://webgpu.github.io/webgpu-samples/samples/A-buffer (Copyright 2019 WebGPU Samples Contributors, BSD 3-Clause License)

## Animation

- https://learn.microsoft.com/en-us/minecraft/creator/documents/entitymodelingandanimation
- https://learn.microsoft.com/en-us/minecraft/creator/reference/content/animationsreference/examples/animationgettingstarted
- https://learn.microsoft.com/en-us/minecraft/creator/reference/content/molangreference/examples/molangconcepts/molangintroduction
- https://github.com/bernie-g/geckolib

Use the same model and animation format as geckolib and minecraft bedrock.

For blocks without animation, just add them to the chunk mesh. For animated entities and blocks,
draw them all to one buffer every frame.

My molang implementation can just be lua. Don't think I really care about ternary operator (`b ? t : f`). 

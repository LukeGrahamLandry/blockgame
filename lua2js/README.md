# Lua -> JS Transpiler

The goal: transpile lua to human-readable javascript and support easy interaction with wasm functions/structs with the same syntax as luajit. 

## Implemented

See the `tests` directory for examples of scripts that produce the same output whether running on luajit or transpiled and running on nodejs. 

- Local and global variables 
- Arithmetic on numbers, `..` on strings, `#` on tables
- Flow control: if/elseif/else, numeric for*
- Functions, anonymous functions, variadic arguments, closures, method calls (`:`)
- Lib: error, pairs*, ipairs*, setmetatable*
- Type striping of luau*
- Call wasm functions (`ffi.cdef`), create wasm structs (`ffi.new`)*
- `require("math")`: floor, pow, min, max, random, sin, cos, tan, PI

> *Partial support. See limitations below. 

## Limitations

Goal: when I can't guarantee same results as luajit, compile error > runtime error > undefined behaviour. 

- There's no sandboxing between lua and your JS. Assume transpiled code has full access to the JS context it runs in.
- Almost none of the standard library is available. A bit of `require("math")` is implemented. 
- Minimal meta table support. Only __index as a table used as JS prototype. No operator overloading. 
    - Checked at runtime that only the __index field exists on the meta table. 
- Several bits of syntax are not implemented yet. 
  - These aren't scary since the compiler will panic, not generate in correct code. 
- Numeric for loops evaluate the max expression on each iteration. It must not have side effects. 
- Numeric for loops cannot have an explicit step expression. Checked at compile time. 
- C structs created with `ffi.new` are not garbage collected. 
  - You must manually call `lua_drop` or you will leak memory. You can easily get use-after-free undefined behaviour. 
- C structs passed to C functions are not type checked. Passing incorrect types is undefined behaviour.
- Limited c syntax support in `ffi.cdef`
- Cannot directly access fields of C structs
- Type striping sometimes incorrectly removes newlines that were required for separating statements. 
- Some short-circuiting coercion is incorrect: `print(0 or 1)`
- pairs/ipairs return one JS iterator object. 
  - Works with for loop constructions. 
- Comments are not preserved. 
- Array-like tables are represented as JS objects with number keys. `#arr` takes O(n) time. 
- Multiple arguments of `print` are printed on the same line. 
- Must `local ffi = require("ffi")`, can't rename it. Can't have another table called ffi. 

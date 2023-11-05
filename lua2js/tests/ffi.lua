local ffi = require("ffi")

ffi.cdef[[
    int add(int a, int b);
    void lua_drop(void* ptr);

    typedef struct Pos {
        int x; int y; int z;
    } Pos;

    void set_y(Pos* pos, int y);
    int get_y(Pos* pos);

    typedef struct Many {
        int things[10];
    } Many;
]]

print(ffi.C.add(1, 2))
print(ffi.C.add(1, 2) == 3)

local p = ffi.new("Pos");
print(ffi.C.get_y(p))
ffi.C.set_y(p, 10)
print(ffi.C.get_y(p))
ffi.C.lua_drop(p)

-- TODO: compile direct field access
--p = ffi.new("Pos");
--print(ffi.C.get_y(p))
--p.y = 15;
--print(ffi.C.get_y(p))

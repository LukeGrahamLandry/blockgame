function add(a: number, b: number): number
    local c: number = a + b
    return c
end

print(add(1, 2))

type Point = { x: number, y: number }
local p: Point = { x=10, y=15 }
print(p.x)

function foo(x: number, y: string): boolean
    local k: string = x :: string  -- this shouldn't type check but we're not there yet 
    return k == "a"
end

local foo2: (number, string) -> boolean = foo

print(foo(12, "apple"))

foo2(1, "a");

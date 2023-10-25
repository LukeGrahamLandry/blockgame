function add(a, b)
    local c = a + b
    return c
end

print(add(1, 2))
print(add(add(1.5, 10), add(-1, -10)))

function multi(a, b, c)
    return a + 1, b + 2, c + 3
end

local a, b, c = multi(1, 2, 3)
print(a)
print(b)
print(c)

function print_many(first, ...)
    print(first)
    local arg={...}
    for _,v in ipairs(arg) do
        print(v)
    end
end

print_many(1, 2, 3, 4)

print[[
Multi
Line
String
Call
]]

-- TODO: js does one line, lua does many lines. easy to fix, just don't call console.log directly. have a helper that unpacks args into many calls.
-- print(1, 2, 3)

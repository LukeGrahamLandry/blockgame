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

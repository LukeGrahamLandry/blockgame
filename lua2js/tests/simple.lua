local a = 10
print(a)
local b = a + 1
print(b)

print(1 + 5)
print(1 - 5)
print(5 - 1)
print(1 - 5)
print(5 * 10)
print(10 * 0.5)
print(5 / 10)
print(10 / 2)
print(11 % 2)
print("a" .. "b")
print(1 < 5)
print(1 <= 5)
print(5 <= 5)
print(1 > 5)
print(1 <= 5)
print(5 <= 5)
print(1 == 5)
print(5 == 5)
print(1 ~= 5)
print(5 ~= 5)
print(true or false)
print(true or true)
print(false or false)
print(true and false)
print(true and true)
print(false and false)
print(not true)
print(not false)

-- Precedence
print(1 + 2 * 3)
print(1 - 2 - 3)
print((1 + 2) * 3)
print(-2 - 2)

-- Short circuiting
print(false or 10)
print(true or 10)
print(false and 10)
print(true and 10)

-- Wierd ones
print((-1) % 10)
print(0.5 % 10)
print(0.5 % 0.25)
print(1 % -10)

-- Type coercion
print(0 == "0")
print(true == nil)
print(false == nil)
print(not nil)
print(not 1)
print(not 0)

-- Scope
a_global = 10
print(a_global)
if true then
    local a_global = 15
    print(a_global)
end
print(a_global)

-- TODO: broken
-- print(0 or 1)

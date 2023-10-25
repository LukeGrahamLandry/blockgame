local fruits = {
    apple = 5,
    orange = 15,
    pear = 1,
}

print(fruits.apple)
print(fruits["apple"])
fruits.apple = 10
print(fruits.apple)

local numbers = {
    [1] = 5,
    [2] = 6,
    [3] = 7,
}

print(#fruits)
print(#numbers)

local nested = {
    fruits = fruits,
    numbers = numbers,
    more = {
        another = "hello",
        [1] = "world"
    }
}

print(nested.fruits.apple)
print(nested.more.another)

for i,v in ipairs(numbers) do
    print(i)
    print(v)
end

-- Iterating by index on something that isn't used as an array is allowed, just useless.
for i,v in ipairs(fruits) do
    print(i)
    print(v)
end

-- Iteration order is not well defined so can't just print everything
local count = 0
for i,v in pairs(fruits) do
    count = count + 1
end
print(count)

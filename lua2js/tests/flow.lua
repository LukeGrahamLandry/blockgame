local a = 10

if a > 0 then
    print("yes")
    a = 0
else
    print("nope")
end

if a > 0 then
    print("nope")
else
    print("this time")
end


if a == "abc" then
    print("nope")
elseif a < 10 then
    print("nope2")
else
    print("this time")
end

if a == "abc" then
    print("nope")
elseif a == 0 then
    print("yay")
end

-- Coercion
if 1 then
    print("a")
end

if 2 then
    print("b")
end

if 0 then
    print("c")
end

-- Basic numeric for loop.
for i=1,3 do
    print(i)
end

-- TODO: support step expression
-- Custom step expression.
--for i=1,10,2 do
--    print(i)
--end
-- Step backwards,
--for i=3,1,-1 do
--    print(i)
--end

-- TODO: End expr is evaluated every time but shouldn't be
--local a = 5
--function lawful_evil()
--    a = a + 1
--    return a
--end
-- Stop expression should only be evaluated once.
--for i=1,lawful_evil() do
--    print(i)
--end
-- Step expression should only be evaluated once.
--for i=1,15,lawful_evil() do
--    print(i)
--end

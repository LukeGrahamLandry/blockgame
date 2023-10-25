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

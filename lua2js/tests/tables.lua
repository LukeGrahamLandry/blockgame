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

function table_len(t)
    local count = 0
    for a,b in pairs(t) do
        count = count + 1
    end
    return count
end
print(table_len(fruits))

function new(cls)
    local obj = {}
    setmetatable(obj, { __index=cls })
    return obj
end

Car = {
    speed = 10,
    drive = function(self, distance)
        print("Drive time: ")
        print(distance / self.speed)
        return distance / self.speed
    end
}

local a_car = new(Car)
local b_car = new(Car)
print(a_car.speed)
b_car.speed = 15
print(b_car.speed)
print(a_car.speed)
b_car.apple = true
print(a_car.apple == nil)
b_car:drive(45)
b_car.drive(b_car, 45)

function technically_legal(car)
    car.speed = car.speed + 10
    return car
end

-- This will fail if the receiver is evaluated twice.
technically_legal(b_car):drive(45)
-- This time it needs to be evaluated twice, even though its the same function, just the way it gets called matters.
technically_legal(a_car).drive(technically_legal(a_car), 45)
-- Method calls are an expression, no cheating!
print(b_car:drive(45) + a_car:drive(45))

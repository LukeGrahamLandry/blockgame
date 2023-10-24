function lua_random(first, second) {
    if (second === undefined) {
        if (first === undefined) {
            return Math.random()
        }
        let upper = first;
        return Math.floor(Math.random() * upper) + 1;
    }
    let lower = first;
    let upper = second;
    return lower + Math.floor(Math.random() * (upper - lower));
}

let LuaHelper = {
    // Lua and JS define modulo of negative numbers differently.
    // https://www.lua.org/manual/5.1/manual.html#2.5.1
    mod: (a, b) => a - Math.floor(a/b)*b,
    require: (name) => LuaHelper.modules[name],

    modules: {
        math: {
            floor: Math.floor,
            sin: Math.sin,
            cos: Math.cos,
            tan: Math.tan,
            max: Math.max,
            min: Math.min,
            pow: Math.pow,
            pi: Math.PI,
            random: lua_random,
        }
    }
}

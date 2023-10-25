const LuaHelper = (() => {
    function random(first, second) {
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

    // Lua's length operator is defined in a strange way since there's no distinction between arrays and tables.
    // https://www.lua.org/manual/5.1/manual.html#2.5.5
    function array_len(a) {
        let count = 0;
        let total = Object.keys(a).length;
        for (let i=1; i<=total; i++) {
            if (a[i] === undefined || a[i] === null) {
                break;
            }
            count += 1
        }
        return count;
    }

    function ipairs(arr){
        return {*[Symbol.iterator]() {
            let total = Object.keys(arr).length;
            for (let i=1; i<=total; i++) {
                if (arr[i] === undefined || arr[i] === null) {
                    break;
                }
                yield [i, arr[i]];
            }
        }};
    }

    function pairs(arr){
        return {*[Symbol.iterator]() {
            for (const k of Object.keys(arr)) {
                yield [k, arr[k]];
            }
        }};
    }

    return {
        // Lua and JS define modulo of negative numbers differently.
        // https://www.lua.org/manual/5.1/manual.html#2.5.1
        mod: (a, b) => a - Math.floor(a/b)*b,
        require: (name) => LuaHelper.modules[name],
        // Lua treats zero as true but JS treats it as false.
        as_bool: (a) => a !== false && a !== null && a !== undefined,
        array_len: array_len,
        ipairs: ipairs,
        pairs: pairs,
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
                random: random,
            }
        }
    }
})();

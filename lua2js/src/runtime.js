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
        setmetatable: (obj, meta) => {
            // TODO: support more powerful metatable stuff.
            if (typeof meta.__index !== "object") throw "meta.__index must be object";
            if (Object.keys(meta).length !== 1) throw "metatable may only define __index";
            obj.__proto__ = meta.__index   // TODO: this is probably not right!
        },
        // This sucks but is only used for variadic functions.
        // JS gives the additional arguments as an array but lua wants to call ipairs on it.
        // TODO: use real js arrays for lua array-like tables and this goes away.
        array_to_table: (arr) => {
            let table = {};
            for (const [i, v] of arr.entries()) {
                table[i + 1] = v;
            }
            return table;
        },
        require_number: (a) => {
            if (typeof a == "number" || (typeof a == "string" && !isNaN(Number(a)))) return a;
            else throw "Expected number (or coercible string) but found " + a;
        },
        require_defined: (a) => {
            if (a !== undefined) return a;
            else throw "Argument not defined";
        },
        // This is unfortunate. Need an expression to call a method and pass self as the first argument but not evaluate the object expression twice (because it might be a method call).
        // Can't use `this` inside the method because lua lets you call as normal function or method depending on the call-site.
        // TODO: the compiler could notice when the expression can't have side effects and emit the normal syntax if it's absolutely sure.
        method_call: (receiver, method_name, ...args) => receiver[method_name].apply(null, [receiver, ...args]),
        modules: {
            ffi: {},  // TODO: have cdef put functions from wasm onto a C object here so the generated code looks more like the original. its a step towards not forcing the variable to be called ffi
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

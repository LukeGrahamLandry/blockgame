local ffi = require("ffi")
local math = require("math")
--local string = require("string")

rust_state = nil  -- TODO: this sucks, but I don't really want to pass around rust privileges everywhere

-- TODO: seesea doesn't recognise 'unsigned' yet
ffi.cdef[[
typedef short u16;
typedef int i32;
typedef struct Tile { u16 v; } Tile;

typedef struct Chunk {
    i32 x; i32 y; i32 z;
    Tile tiles[4096];  // TODO: seesea can't evaluate expressions here (16*16*16)
    char dirty;  // TODO: seesea doesn't know bool yet
} Chunk;

void generate_chunk(void* state, Chunk* chunk, int x, int y, int z);
void update_mesh(void* state, Chunk* chunk);
int chunk_get_block(Chunk* chunk, int index);
int chunk_set_block(Chunk* chunk, int index, int tile);
void unload_chunk(void* state, Chunk* chunk);
void lua_drop(void* ptr);

]]

function new<T>(cls: T): T
    local obj = {} 
    setmetatable(obj, { __index=cls })
    return obj
end

local random_tick_delay_sec = 0.1  -- each chunk should tick once every x seconds
local blocks_per_random_tick = 4000  -- each time a chunk gets ticked, x blocks will get ticked
local ticks_per_sec = 60
local chunk_size = 16

-- Calls to this function are magically removed when transpiling to JS if not in SAFE mode. (TODO: do that in type stripping for native as well)
-- Arguments passed to it must not have side effects. With great power comes great responsibility!
function debug_assert(c, msg, ...)
    local arg={...}
    if not c then
        -- TODO: implement format and unpack in my transpiler
        -- error(string.format(msg, unpack(arg)))
        for _,v in arg do
            print(v)
        end
        error(msg)
    end
end

-- TODO: my type stripping is fragile. if these don't have a value, they get put on the same line
local block_random_tick_handlers: { [number]: (World, Chunk, number, number, number) -> () } = {}
--local gen: { tiles: { [string]: number} } = gen

type Chunk = {
    tiles: { [number]: { v: number } },
    dirty: boolean,
    x: number,
    y: number,
    z: number
}

World = {
    chunks = {} :: { [string]: Chunk },
    any_chunk_dirty = false,

    -- x,y,z are ChunkPos
    get_chunk = function(self, x: number, y: number, z: number): Chunk
        local key = x .. ":" .. y .. ":" .. z
        local chunk: Chunk = self.chunks[key]
        if chunk == nil then
            chunk = ffi.new("Chunk")
            ffi.C.generate_chunk(rust_state, chunk, x, y, z)
            self.any_chunk_dirty = true
            self.chunks[key] = chunk
        end
        return chunk
    end,

    unload_chunk = function(self, cx: number, cy: number, cz: number)
        local key = cx .. ":" .. cy .. ":" .. cz
        local chunk = self.chunks[key]
        if chunk ~= nil then
            ffi.C.unload_chunk(rust_state, chunk)
            ffi.C.lua_drop(chunk)
            self.chunks[key] = nil
        end
    end,

    -- x,y,z are BlockPos. tile must be a constant from the gen.tiles
    set_block = function(self, bx, by, bz, tile)
        local cx, cy, cz = block_to_chunk_pos(bx, by, bz)
        local lx, ly, lz = block_to_local_pos(bx, by, bz)
        local chunk = self:get_chunk(cx, cy, cz)
        self:set_block_local(chunk, lx, ly, lz, tile)
    end,

    set_block_local = function(self, chunk: Chunk, lx, ly, lz, tile)
        local index = local_to_index(lx, ly, lz)
        -- TODO: some sort of type safety so you can't just pass random numbers in. for now, debug mode rust checks when generating the mesh
        -- TODO: implement struct fields in my compiler so I don't have to write cringe getters
        --local old = chunk.tiles[index].v
        --chunk.tiles[index].v = tile
        --if old ~= tile then
        --    chunk.dirty = true
        --    self.any_chunk_dirty = true
        --end
        if ffi.C.chunk_set_block(chunk, index, tile) ~= 0 then
            self.any_chunk_dirty = true
        end
    end,

    get_block = function(self, bx, by, bz)
        local cx, cy, cz = block_to_chunk_pos(bx, by, bz)
        local lx, ly, lz = block_to_local_pos(bx, by, bz)
        local chunk = self:get_chunk(cx, cy, cz)
        return self.get_block_local(chunk, lx, ly, lz)
    end,

    -- TODO: its a little dumb that this is in the World table but isn't a method so has different syntax to call
    get_block_local = function(chunk, lx, ly, lz)
        local index = local_to_index(lx, ly, lz)
        --return chunk.tiles[index].v
        return ffi.C.chunk_get_block(chunk, index)
    end,

    -- Indexes are undefined and inconsistent. Only useful for choosing a random chunk
    --- @param i number
    get_chunk_index = function(self, i): Chunk
        -- TODO: this is kinda dumb
        local count = 0
        for _, chunk in pairs(self.chunks) do
            count = count + 1
            if count == i then
                return chunk
            end
        end
        debug_assert(false, "get_chunk_index %d out of bounds", i)
        error("unreachable")
    end,

    -- TODO: track separately since I know when a chunk is added
    chunk_count = function(self)
       return table_len(self.chunks)
    end,

    do_random_ticks = function(self, chunk: Chunk)
        for i=1,blocks_per_random_tick do
            local lx, ly, lz = math.random(0, chunk_size -1), math.random(0, chunk_size -1), math.random(0, chunk_size -1)
            local handler = block_random_tick_handlers[self.get_block_local(chunk, lx, ly, lz)]
            if handler ~= nil then
                handler(self, chunk, lx, ly, lz)
            end
        end
    end,
}

-- The # operator is only for array like ones. This is not a great language!
function table_len(t)
    local count = 0
    -- TODO: my transpiler doesn't handle name collision
    for _,__ in pairs(t) do
        count = count + 1
    end
    return count
end

-- TODO: how does lua % work on negative numbers

function block_to_chunk_pos(bx, by, bz)
    return (bx - (bx % chunk_size)) / chunk_size, (by - (by % chunk_size)) / chunk_size, (bz - (bz % chunk_size)) / chunk_size
end

function block_to_local_pos(bx, by, bz)
    return bx % chunk_size, by % chunk_size, bz % chunk_size
end

function local_to_index(lx, ly, lz)
    debug_assert(lx < chunk_size and ly < chunk_size and lz < chunk_size and lx >= 0 and ly >= 0 and lz >= 0, "local chunk index (%d, %d, %d) out of bounds.", lx, ly, lz)
    return (ly * chunk_size * chunk_size) + (lx * chunk_size) + lz
end

type World = typeof(World)

the_world = new(World)

local load_radius = 5
local unload_radius = 8
local prev_chunk = nil
local ticks_in_chunk = 0

function load_around_player(player_bx, player_by, player_bz) 
    local cx, cy, cz = block_to_chunk_pos(player_bx, player_by, player_bz)
    local current_chunk = the_world:get_chunk(cx, cy, cz)
    if prev_chunk ~= current_chunk then
        ticks_in_chunk = 0
        load_square(cx, cy, cz, load_radius)
        unload_square(cx, cy, cz, unload_radius)
        if prev_chunk == nil then
            for r=1,(load_radius - 1) do
                load_square(cx, cy, cz, r)
            end
        end
        prev_chunk = current_chunk
    elseif ticks_in_chunk < load_radius then
        ticks_in_chunk = ticks_in_chunk + 1 
        load_square(cx, cy + ticks_in_chunk, cz, load_radius)
        load_square(cx, cy - ticks_in_chunk, cz, load_radius)
        unload_square(cx, cy + ticks_in_chunk, cz, unload_radius)
        unload_square(cx, cy - ticks_in_chunk, cz, unload_radius)
    end
end

-- Load a hollow square around (cx, cy, cz) flat (y=cy). 
function load_square(cx, cy, cz, rad)
    for x=-rad,rad do 
        local _ = the_world:get_chunk(cx + x, cy, cz - rad)
        local __ = the_world:get_chunk(cx + x, cy, cz + rad)
    end
    for z=-rad,rad do 
        local _ = the_world:get_chunk(cx - rad, cy, cz + z)
        local __ = the_world:get_chunk(cx + rad, cy, cz + z)
    end
end

function unload_square(cx, cy, cz, rad)
    for x=-rad,rad do 
        local _ = the_world:unload_chunk(cx + x, cy, cz - rad)
        local __ = the_world:unload_chunk(cx + x, cy, cz + rad)
    end
    for z=-rad,rad do 
        local _ = the_world:unload_chunk(cx - rad, cy, cz + z)
        local __ = the_world:unload_chunk(cx + rad, cy, cz + z)
    end
end

function run_tick(state, player_bx, player_by, player_bz)
    rust_state = state

    load_around_player(player_bx, player_by, player_bz)
    
    local count = the_world:chunk_count()
    if count > 0 then
        -- Each chunk ticks every x so one of n chunks ticks every x/n
        local adjusted_tick_rate = (random_tick_delay_sec * ticks_per_sec) / count
        local do_tick = math.random(adjusted_tick_rate) == 1
        if do_tick then
            local chunk = the_world:get_chunk_index(math.random(count))
            the_world:do_random_ticks(chunk)
        end
    end

    -- TODO: rust could check the flags on all chunks in render distance every frame, is that better?
    -- If any changes were made this tick,
    if the_world.any_chunk_dirty then
        the_world.any_chunk_dirty = false
        for key, chunk in pairs(the_world.chunks) do
            ffi.C.update_mesh(rust_state, chunk)
        end
    end
end

-- for passing values from rust to lua:
-- LightUserData(*mut c_void)
-- local chunk = ffi.cast("Chunk", chunk)

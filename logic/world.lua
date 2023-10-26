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

Chunk* get_chunk(void* state, int x, int y, int z);
void update_mesh(void* state);
int chunk_get_block(Chunk* chunk, int index);
int chunk_set_block(Chunk* chunk, int index, int tile);
void unload_chunk(void* state, int x, int y, int z);
void lua_drop(void* ptr);
Chunk* random_chunk(void* state);
void gc_chunks(void* state, int x, int y, int z);
void render_entity(void* state, int id, int ty, float x, float y, float z);
void forget_entity(void* state, int id);
]]

function new<T>(cls: T): T
    local obj = {} 
    setmetatable(obj, { __index=cls })
    return obj
end

local random_tick_delay_sec = 0.1  -- each chunk should tick once every x seconds
local blocks_per_random_tick = 4000  -- each time a chunk gets ticked, x blocks will get ticked
local ticks_per_sec = 20
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

local load_radius = 5
local unload_radius = 8
local prev_chunk = nil  -- never read through this! it might have been dropped. just compare the pointer. that's also wrong if it decided to reuse but unlikely 
local ticks_in_chunk = 0

type Chunk = {
    tiles: { [number]: { v: number } },
    dirty: boolean,
    x: number,
    y: number,
    z: number
}

World = {
    any_chunk_dirty = false,
    entities = {},

    -- x,y,z are ChunkPos
    get_chunk = function(self, x: number, y: number, z: number): Chunk
        return ffi.C.get_chunk(rust_state, x, y, z)
    end,

    unload_chunk = function(self, cx: number, cy: number, cz: number)
        ffi.C.unload_chunk(rust_state, cx, cy, cz)
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

    do_random_ticks = function(self, chunk: Chunk)
        for i=1,blocks_per_random_tick do
            local lx, ly, lz = math.random(0, chunk_size -1), math.random(0, chunk_size -1), math.random(0, chunk_size -1)
            local handler = block_random_tick_handlers[self.get_block_local(chunk, lx, ly, lz)]
            if handler ~= nil then
                handler(self, chunk, lx, ly, lz)
            end
        end
    end,

    add_entity = function(self, entity)
        self.entities[entity.id] = entity
    end,

    remove_entity = function(self, entity)
        self.entities[entity.id] = nil
        ffi.C.forget_entity(rust_state, entity.id)
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
    return math.abs(math.floor(bx)) % chunk_size, math.abs(math.floor(by)) % chunk_size, math.abs(math.floor(bz)) % chunk_size
end

function local_to_index(lx, ly, lz)
    debug_assert(lx < chunk_size and ly < chunk_size and lz < chunk_size and lx >= 0 and ly >= 0 and lz >= 0, "local chunk index (%d, %d, %d) out of bounds.", lx, ly, lz)
    return (ly * chunk_size * chunk_size) + (lx * chunk_size) + lz
end

type World = typeof(World)

the_world = new(World)

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
        return true
    elseif ticks_in_chunk < load_radius then
        ticks_in_chunk = ticks_in_chunk + 1 
        load_square(cx, cy + ticks_in_chunk, cz, load_radius)
        load_square(cx, cy - ticks_in_chunk, cz, load_radius)
        unload_square(cx, cy + ticks_in_chunk, cz, unload_radius)
        unload_square(cx, cy - ticks_in_chunk, cz, unload_radius)
        return false
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

local extra_time = 0
local tick_interval_secs = 1/20
local spawn_x = 0

function run_tick(state, player_bx, player_by, player_bz, dt_sec)
    rust_state = state

    extra_time = math.min(extra_time + dt_sec, 1)
    if extra_time < tick_interval_secs then
        return
    end
    extra_time = extra_time - tick_interval_secs

    local changed_chunks = load_around_player(player_bx, player_by, player_bz)
    
    -- Each chunk ticks every x so one of n chunks ticks every x/n
    local count = load_radius*load_radius*load_radius
    local adjusted_tick_rate = (random_tick_delay_sec * ticks_per_sec) / count
    local do_tick = math.random(adjusted_tick_rate) == 1
    if do_tick then
        local chunk = ffi.C.random_chunk(rust_state)
        the_world:do_random_ticks(chunk)
    end

    if math.random() < 0.05 then
        the_world:add_entity(new(FallingBlock.init(the_world, spawn_x, 15, 0, gen.tiles.stone)))
        if math.random() < 0.3 then
            spawn_x = spawn_x + 1
        end
    end

    for id,entity in pairs(the_world.entities) do
        entity:tick()
    end

    -- TODO: rust could check the flags on all chunks in render distance every frame, is that better?
    -- If any changes were made this tick,
    if the_world.any_chunk_dirty then
        the_world.any_chunk_dirty = false
        ffi.C.update_mesh(rust_state)
    end

    -- You can never hold a chunk pointer accross this point because rust might decide to drop it!
    if changed_chunks and math.random() < 0.05 then
        ffi.C.gc_chunks(rust_state, player_bx, player_by, player_bz)
    end
end

-- for passing values from rust to lua:
-- LightUserData(*mut c_void)
-- local chunk = ffi.cast("Chunk", chunk)

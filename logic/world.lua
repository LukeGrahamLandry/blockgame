local ffi = require("ffi")
local math = require("math")
--local string = require("string")

ffi.cdef[[
typedef unsigned short u16;
typedef int i32;
typedef struct Tile { u16 v; } Tile;

typedef struct Chunk {
    i32 x; i32 y; i32 z;
    Tile tiles[16*16*16];
    bool dirty;
} Chunk;

void generate_chunk(void* state, Chunk* chunk);
void update_mesh(void* state, Chunk* chunk);

]]

--- @generic A
--- @param cls A
--- @return A
function new(cls)
    local obj = {}
    setmetatable(obj, { __index=cls })
    return obj
end

local random_tick_delay_sec = 0.1  -- each chunk should tick once every x seconds
local blocks_per_random_tick = 4000  -- each time a chunk gets ticked, x blocks will get ticked
local ticks_per_sec = 60
local chunk_size = 16

-- TODO: give a way to remove this. rust replace conditionally?
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

World = {
    chunks = {},
    any_chunk_dirty = false,

    -- x,y,z are ChunkPos
    get_chunk = function(self, x, y, z)
        local key = x .. ":" .. y .. ":" .. z
        local chunk = self.chunks[key]
        if chunk == nil then
            chunk = ffi.new("Chunk")
            chunk.x = x
            chunk.y = y
            chunk.z = z
            ffi.C.generate_chunk(rust_state, chunk)
            self.any_chunk_dirty = true
            self.chunks[key] = chunk
        end
        return chunk
    end,

    -- x,y,z are BlockPos. tile must be a constant from the gen.tiles
    set_block = function(self, bx, by, bz, tile)
        local cx, cy, cz = block_to_chunk_pos(bx, by, bz)
        local lx, ly, lz = block_to_local_pos(bx, by, bz)
        local chunk = self:get_chunk(cx, cy, cz)
        self:set_block_local(chunk, lx, ly, lz, tile)
    end,

    set_block_local = function(self, chunk, lx, ly, lz, tile)
        local index = local_to_index(lx, ly, lz)
        -- TODO: some sort of type safety so you can't just pass random numbers in. for now, debug mode rust checks when generating the mesh
        local old = chunk.tiles[index].v
        chunk.tiles[index].v = tile
        if old ~= tile then
            chunk.dirty = true
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
        return chunk.tiles[index].v
    end,

    -- Indexes are undefined and inconsistent. Only useful for choosing a random chunk
    --- @param i number
    get_chunk_index = function(self, i)
        -- TODO: this is kinda dumb
        local count = 0
        for _, chunk in pairs(self.chunks) do
            count = count + 1
            if count == i then
                return chunk
            end
        end
        debug_assert(false, "get_chunk_index %d out of bounds", i)
    end,

    -- TODO: track separately since I know when a chunk is added
    chunk_count = function(self)
       return table_len(self.chunks)
    end,

    do_random_ticks = function(self, chunk)
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

the_world = new(World)
rust_state = nil  -- TODO: this sucks, but I don't really want to pass around rust privileges everywhere

function run_tick(state)
    rust_state = state
    the_world:set_block(0, 0, 0, gen.tiles.stone)

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

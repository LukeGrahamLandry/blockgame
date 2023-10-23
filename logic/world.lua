local ffi = require("ffi")
ffi.cdef[[
int printf(const char *fmt, ...);
int add(int a, int b);
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



local a = ffi.C.add(1, 2)
print(a)
ffi.C.printf("Hello %f\n", a)

function tick_chunk(chunk)
    local chunkPtr = ffi.cast("Chunk", chunk)
    local tile = chunkPtr.tiles[1]
    print(tile.v)
    return 1
end

function new(cls)
    local obj = {}
    setmetatable(obj, { __index=cls })
    return obj
end

World = {
    chunks = {},

    -- x,y,z are ChunkPos
    get_chunk = function(self, x, y, z)
        if self.chunks[x] == nil then
            self.chunks[x] = {}
        end
        if self.chunks[x][y] == nil then
            self.chunks[x][y] = {}
        end
        if self.chunks[x][y][z] == nil then
            local chunk = ffi.new("Chunk")
            chunk.x = x
            chunk.y = y
            chunk.z = z
            ffi.C.generate_chunk(current_state, chunk)
            ffi.C.update_mesh(current_state, chunk)
            self.chunks[x][y][z] = chunk
        end
        return self.chunks[x][y][z]
    end,

    -- x,y,z are BlockPos
    set_block = function(self, x, y, z, tile)
        local cx, cy, cz = block_to_chunk_pos(x, y, z)
        print(cx, cy, cz)
        local lx, ly, lz = block_to_local_pos(x, y, z)
        local chunk = self:get_chunk(cx, cy, cz)
        local index = local_to_index(lx, ly, lz)
        print("local index", index)
        chunk.tiles[index].v = tile
        chunk.dirty = true
        ffi.C.update_mesh(current_state, chunk)  -- TODO: do this lazily

    end
}

local CHUNK_SIZE = 16

function block_to_chunk_pos(x, y, z)
    return (x - (x % CHUNK_SIZE)) / CHUNK_SIZE, (y - (y % CHUNK_SIZE)) / CHUNK_SIZE, (z - (z % CHUNK_SIZE)) / CHUNK_SIZE
end

function block_to_local_pos(x, y, z)
    return x % CHUNK_SIZE, y % CHUNK_SIZE, z % CHUNK_SIZE
end

function local_to_index(x, y, z)
    return (y * CHUNK_SIZE * CHUNK_SIZE) + (x * CHUNK_SIZE) + z
end


the_world = new(World)
current_state = nil

function run_tick(state)
    current_state = state
    the_world:set_block(0, 0, 0, 1)
    --local chunk = the_world:get_chunk(0, 0, 0);
    --print(chunk.tiles[0].v)

end


-- for passing values from rust to lua:
-- local chunk = ffi.new("Chunk")
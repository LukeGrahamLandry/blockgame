local next_id = 1

Entity = {
    x = 0, y = 0, z = 0,
    world = nil, ty = 0, id = 0,
    vel_x = 0, vel_y = 0, vel_z = 0,

    init = function(self, world, x, y, z, ty)
        self.world = world
        self.x = x
        self.y = y
        self.z = z
        self.ty = ty
        self.id = next_id
        next_id = next_id + 1
    end,

    tick = function(self)
        self.x = self.x + self.vel_x
        self.y = self.y + self.vel_y
        self.z = self.z + self.vel_z
        ffi.C.render_entity(rust_state, self.id, self.ty, self.x, self.y, self.z)
    end,
}

FallingBlock = {
    tile = 0,
    
    init = function(world, x, y, z, tile)
        local self = new(FallingBlock)
        Entity.init(self, world, x, y, z, 1)
        self.vel_y = -0.5
        self.tile = tile
        return self
    end,

    tick = function(self)
        local below = self.world:get_block(self.x, self.y - 1, self.z)
        if below ~= 0 then
            self.world:set_block(self.x, self.y, self.z, self.tile)
            self.world:remove_entity(self)
        else
            Entity.tick(self)
        end
    end,
}

setmetatable(FallingBlock, { __index = Entity })

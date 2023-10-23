block_random_tick_handlers = {
    [gen.tiles.stone] = function(world, chunk, lx, ly, lz)
        if math.random(20) == 1 then
            world:set_block_local(chunk, lx, ly, lz, gen.tiles.dirt)
        end
    end
}

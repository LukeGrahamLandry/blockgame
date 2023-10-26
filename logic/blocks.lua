function add_growth(before, after, chance)
    debug_assert(block_random_tick_handlers[before] == nil, "Cannot override tick handler... yet.")
    block_random_tick_handlers[before] = function(world, chunk, lx, ly, lz)
        if math.random(chance) == 1 then
            world:set_block_local(chunk, lx, ly, lz, after)
        end
    end
end

add_growth(gen.tiles.wheat1, gen.tiles.wheat2, 20)
add_growth(gen.tiles.wheat2, gen.tiles.wheat3, 20)
add_growth(gen.tiles.wheat3, gen.tiles.wheat, 20)

-- gamespy_advertise.lua
-- Advertises a server's state/capabilities for a given GameSpyIdentity.

local Server = {
    id = "SRV-EXAMPLE-001",
    region = "US-EAST",
    version = "1.0.0",
    protocol = "iog-gamespy-v1"
}

-- Static metadata – these should line up with GameSpyIdentity on the Rust side.
Server.game = {
    game_id = "battlefield2",
    title = "Battlefield 2",
    region_code = "US",
    platform = "pc"
}

-- Capabilities -> maps to classic GameSpy rules/caps
function Server:get_capabilities()
    return {
        sver = "1.41",
        ded = "1",
        password = "0",
        ff = "0",
        tc = "1",
        punkbuster = "0",
        max_players = 64,
        current_players = 12,
        map = "strike_at_karkand",
        gametype = "gpm_cq",
        mods = { "vanilla" }
    }
end

-- Called periodically by Rust; returns a table that can be serialized to JSON or KV.
function Server:on_heartbeat()
    local payload = {
        server_id = self.id,
        game = self.game,
        region = self.region,
        version = self.version,
        protocol = self.protocol,
        capabilities = self:get_capabilities(),
        timestamp = os.time()
    }
    return payload
end

return Server

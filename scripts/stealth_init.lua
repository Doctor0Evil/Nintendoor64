-- scripts/stealth_init.lua
local json = require("json")      -- any JSON lib bound into Lua
local StealthConfig = {}

local function load_stealth_config(path)
    local f, err = io.open(path, "r")
    if not f then
        error("Failed to open stealth config: " .. tostring(err))
    end
    local text = f:read("*a")
    f:close()
    local cfg, jerr = json.decode(text)
    if not cfg then
        error("Failed to decode stealth config: " .. tostring(jerr))
    end
    return cfg
end

function StealthConfig.Apply(path)
    local cfg = load_stealth_config(path)

    -- Global parameters
    local g = cfg.global
    Engine.Stealth.SetGlobalParams(
        g.base_visibility,
        g.d_max,
        g.k_decay,
        g.t_suspicious,
        g.t_alert,
        g.light_exponent,
        g.move_exponent
    )

    -- Posture factors
    local pf = cfg.posture_factors
    Engine.Stealth.SetPostureFactors(
        pf.standing,
        pf.crouch,
        pf.prone
    )

    -- Movement factors
    local mf = cfg.movement_factors
    Engine.Stealth.SetMovementFactors(
        mf.still,
        mf.walk,
        mf.run
    )

    -- Sound (optional v1)
    if cfg.sound and cfg.sound.enabled then
        local s = cfg.sound
        Engine.Stealth.SetSoundParams(
            s.max_distance,
            s.weapon_loudness or {},
            s.footstep_loudness or {}
        )
    end
end

return StealthConfig

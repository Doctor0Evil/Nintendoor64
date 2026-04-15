use anyhow::Result;
use iog_protocol_model::{PacketEnvelope, PacketHandlerResult};
use mlua::{Lua, LuaSerdeExt};
use wasmtime::{Engine, Store, Module, Linker};

pub struct LuaSandbox {
    lua: Lua,
}

impl LuaSandbox {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        Ok(Self { lua })
    }

    pub fn load_script(&self, source: &str) -> Result<()> {
        self.lua.load(source).exec()?;
        Ok(())
    }

    pub fn handle_packet(&self, packet: &PacketEnvelope) -> Result<PacketHandlerResult> {
        let func: mlua::Function = self.lua.globals().get("on_packet")?;
        let value = self.lua.to_value(packet)?;
        let res: PacketHandlerResult = func.call(value)?;
        Ok(res)
    }
}

pub struct WasmSandbox {
    engine: Engine,
    module: Module,
}

impl WasmSandbox {
    pub fn new(wasm_bytes: &[u8]) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::from_binary(&engine, wasm_bytes)?;
        Ok(Self { engine, module })
    }

    pub fn handle_packet(&self, packet: &PacketEnvelope) -> Result<PacketHandlerResult> {
        let mut store = Store::new(&self.engine, ());
        let mut linker = Linker::new(&self.engine);

        let instance = linker.instantiate(&mut store, &self.module)?;
        let func = instance.get_typed_func::<(i32, i32), i32>(&mut store, "handle_packet")?;

        let bytes = serde_json::to_vec(packet)?;
        let _ = bytes; // TODO: linear memory binding omitted for brevity

        // Placeholder: in real code, write bytes into WASM memory and return JSON back.
        let _ = func;

        Ok(PacketHandlerResult {
            drop: false,
            rewritten_payload_b64: None,
        })
    }
}

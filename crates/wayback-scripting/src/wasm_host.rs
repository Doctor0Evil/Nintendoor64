// crates/wayback-scripting/src/wasm_host.rs

//! Wasmtime hostcall glue for Wayback protocol plugins.
//!
//! This module defines a small JSON-over-linear-memory contract for passing
//! PacketEnvelope values into a WASM plugin and receiving a transformed
//! PacketEnvelope back out.
//!
//! The contract between host and guest is:
//!
//! Guest exports:
//!   - memory: default linear memory
//!   - alloc(len: i32) -> i32
//!       Allocate `len` bytes in guest memory, return offset.
//!   - dealloc(ptr: i32, len: i32)
//!       Free a previous allocation (optional; host will not reuse).
//!   - process_packet(ptr: i32, len: i32) -> i32
//!       Read `len` bytes of UTF-8 JSON at `ptr`, parse as PacketEnvelope,
//!       write a JSON-encoded PacketEnvelope to a new allocation, and
//!       return its offset. The guest must store the output length in
//!       a well-known 4-byte little-endian word at `RESULT_LEN_PTR`.
//!
//! Host responsibilities:
//!   - Serialize PacketEnvelope as JSON.
//!   - Copy JSON into guest memory at alloc'd offset.
//!   - Call process_packet.
//!   - Read result length from RESULT_LEN_PTR, then read JSON from
//!     returned offset, and deserialize into PacketEnvelope.
//!
//! Safety notes:
//!   - All linear memory access is bounds-checked by using wasmtime::Memory
//!     accessors and slices. See Wasmtime docs for linear memory usage.[web:47][web:50]

use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use wasmtime::{Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::sync::WasiCtxBuilder;

/// Offset in linear memory where the guest writes the length (u32, LE)
/// of the JSON result produced by `process_packet`.
///
/// This is a simple convention; you can later move this into a tiny
/// guest-side runtime or export a dedicated "get_result_len" function.
const RESULT_LEN_PTR: u32 = 0x10;

/// High-level envelope around a network packet.
///
/// This can be shared between native Rust, Lua, and WASM plugins via
/// JSON serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketEnvelope {
    pub src_addr: String,
    pub dst_addr: String,
    pub protocol: String,
    #[serde(with = "serde_bytes")]
    pub payload: Vec<u8>,
    pub meta: serde_json::Value,
}

/// Handle to a compiled and instantiated WASM plugin.
pub struct WasmPacketPlugin {
    engine: Engine,
    module: Arc<Module>,
}

impl WasmPacketPlugin {
    /// Compile a WASM module from disk and prepare it for instantiation.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, path).context("failed to load wasm module")?;
        Ok(Self {
            engine,
            module: Arc::new(module),
        })
    }

    /// Process a PacketEnvelope through the WASM plugin.
    ///
    /// This will:
    ///  - Create a Store with WASI context.
    ///  - Instantiate the module with a WASI linker.
    ///  - Copy JSON into linear memory.
    ///  - Call the guest's `process_packet(ptr, len) -> i32`.
    ///  - Read the output length from RESULT_LEN_PTR.
    ///  - Read the JSON result and deserialize it.
    pub fn process_packet(&self, packet: &PacketEnvelope) -> Result<PacketEnvelope> {
        let mut store = Store::new(
            &self.engine,
            WasiCtxBuilder::new()
                .inherit_stdio()
                .inherit_args()
                .build(),
        );

        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker(&mut linker, |ctx| ctx)?;

        let instance = linker
            .instantiate(&mut store, &self.module)
            .context("failed to instantiate wasm module")?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow!("wasm module does not export memory"))?;

        let alloc = instance
            .get_typed_func::<i32, i32, _>(&mut store, "alloc")
            .context("missing alloc(i32) -> i32 export")?;
        let dealloc = instance
            .get_typed_func::<(i32, i32), (), _>(&mut store, "dealloc")
            .ok(); // optional

        let process = instance
            .get_typed_func::<(i32, i32), i32, _>(&mut store, "process_packet")
            .context("missing process_packet(i32, i32) -> i32 export")?;

        // Serialize the packet as JSON.
        let input_json = serde_json::to_vec(packet).context("failed to serialize PacketEnvelope")?;
        let input_len = input_json.len() as i32;

        // Allocate memory in the guest.
        let ptr = alloc
            .call(&mut store, input_len)
            .context("alloc call failed")?;

        // Copy JSON into guest linear memory.
        write_to_memory(&memory, &mut store, ptr as u32, &input_json)?;

        // Call process_packet.
        let out_ptr = process
            .call(&mut store, (ptr, input_len))
            .context("process_packet call failed")?;

        // Optional: free input buffer.
        if let Some(dealloc) = dealloc {
            let _ = dealloc.call(&mut store, (ptr, input_len));
        }

        // Read result length (u32 LE) from RESULT_LEN_PTR.
        let out_len = read_u32_le(&memory, &mut store, RESULT_LEN_PTR)? as usize;

        // Read JSON result from out_ptr.
        let output_json = read_from_memory(&memory, &mut store, out_ptr as u32, out_len)?;
        let out_packet: PacketEnvelope =
            serde_json::from_slice(&output_json).context("failed to deserialize PacketEnvelope")?;

        // Optional: free output buffer.
        if let Some(dealloc) = dealloc {
            let _ = dealloc.call(&mut store, (out_ptr, out_len as i32));
        }

        Ok(out_packet)
    }
}

/// Write a byte slice into guest linear memory at the given offset.
fn write_to_memory(
    memory: &wasmtime::Memory,
    store: &mut Store<wasmtime_wasi::WasiCtx>,
    offset: u32,
    data: &[u8],
) -> Result<()> {
    let start = offset as usize;
    let end = start + data.len();

    let mem_size = memory.data_size(store);
    if end > mem_size {
        return Err(anyhow!(
            "write_to_memory out of bounds: end={} > mem_size={}",
            end,
            mem_size
        ));
    }

    let mem = memory.data_mut(store);
    mem[start..end].copy_from_slice(data);
    Ok(())
}

/// Read a byte vector from guest linear memory.
fn read_from_memory(
    memory: &wasmtime::Memory,
    store: &mut Store<wasmtime_wasi::WasiCtx>,
    offset: u32,
    len: usize,
) -> Result<Vec<u8>> {
    let start = offset as usize;
    let end = start + len;

    let mem_size = memory.data_size(store);
    if end > mem_size {
        return Err(anyhow!(
            "read_from_memory out of bounds: end={} > mem_size={}",
            end,
            mem_size
        ));
    }

    let mem = memory.data(store);
    Ok(mem[start..end].to_vec())
}

/// Read a little-endian u32 from guest memory.
fn read_u32_le(
    memory: &wasmtime::Memory,
    store: &mut Store<wasmtime_wasi::WasiCtx>,
    offset: u32,
) -> Result<u32> {
    let bytes = read_from_memory(memory, store, offset, 4)?;
    if bytes.len() != 4 {
        return Err(anyhow!("expected 4 bytes for u32 len, got {}", bytes.len()));
    }
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

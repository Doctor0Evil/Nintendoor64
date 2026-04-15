use std::{fs, path::PathBuf};
use schemars::schema_for;
use iog_protocol_model::{
    PacketEnvelope, PacketHandlerResult, ProtocolPluginDescriptor, ProtocolFingerprint,
};

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schemas");
    fs::create_dir_all(&root).expect("create schemas dir");

    let write_schema = |name: &str, schema| {
        let path = root.join(name);
        let json = serde_json::to_vec_pretty(&schema).unwrap();
        fs::write(path, json).unwrap();
    };

    write_schema(
        "iog.packet-envelope.schema.json",
        schema_for!(PacketEnvelope),
    );
    write_schema(
        "iog.packet-handler-result.schema.json",
        schema_for!(PacketHandlerResult),
    );
    write_schema(
        "iog.protocol-plugin-descriptor.schema.json",
        schema_for!(ProtocolPluginDescriptor),
    );
}

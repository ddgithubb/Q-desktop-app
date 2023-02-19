use std::io::Result;

fn main() -> Result<()> {
  let mut config = prost_build::Config::new();
  config.type_attribute(".sync_server.v1", "#[derive(serde::Serialize, serde::Deserialize)]");
  config.type_attribute(".sync_server.v1", "#[serde(rename_all = \"camelCase\")]");
  config.type_attribute(".pool.v1.PoolFileInfo", "#[derive(serde::Serialize, serde::Deserialize)]");
  config.type_attribute(".pool.v1.PoolFileInfo", "#[serde(rename_all = \"camelCase\")]");
  config.type_attribute(".pool.v1.PoolFileSeeders", "#[derive(serde::Serialize, serde::Deserialize)]");
  config.type_attribute(".pool.v1.PoolFileSeeders", "#[serde(rename_all = \"camelCase\")]");
  config.type_attribute(".pool.v1.PoolImageData", "#[derive(serde::Serialize, serde::Deserialize)]");
  config.type_attribute(".pool.v1.PoolImageData", "#[serde(rename_all = \"camelCase\")]");
  config.type_attribute(".pool.v1.PoolChunkRange", "#[derive(serde::Serialize, serde::Deserialize)]");
  config.type_attribute(".pool.v1.PoolChunkRange", "#[serde(rename_all = \"camelCase\")]");
  config.type_attribute(".pool.v1.PoolMessage", "#[derive(serde::Serialize, serde::Deserialize)]");
  config.type_attribute(".pool.v1.PoolMessage", "#[serde(rename_all = \"camelCase\")]");
  config.bytes(&["."]);
  config.compile_protos(&["src/sync_server.v1.proto", "src/pool.v1.proto"], &["src/"])?;
  tauri_build::build();
  Ok(())
}

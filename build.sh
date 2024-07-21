cargo build --release --target wasm32-unknown-unknown
wasm-bindgen \
  --no-typescript --target web \
  --out-name bevy_game --out-dir ./wasm/target \
  ./target/wasm32-unknown-unknown/release/bevy_jam_5_cycles.wasm
cp -r assets wasm
zip wasm.zip wasm/**/*
basic-http-server wasm
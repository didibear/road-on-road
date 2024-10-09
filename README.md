# Road on Road

![image](https://github.com/user-attachments/assets/681cab8c-9a4a-4654-9745-314caaf04cc6)

Little game for the [Bevy Jam #5 - Cycles](https://itch.io/jam/bevy-jam-5).

## Game

https://didibear.itch.io/road-on-road

Make as many journeys as possible, avoiding previous versions of yourself!

Commands: Arrow keys or WASD

## Development

Project setup using [bevy_github_ci_template](https://github.com/bevyengine/bevy_github_ci_template).  
Sounds generated using [Chiptone](https://sfbgames.itch.io/chiptone)

## Build

1. Compile wasm app

```sh
cargo build --release --target wasm32-unknown-unknown
```

2. Create JS bindings

```sh
wasm-bindgen \
  --no-typescript --target web \
  --out-name bevy_game --out-dir ./wasm/target \
  ./target/wasm32-unknown-unknown/release/bevy_jam_5_cycles.wasm
```

3. Copy assets to the `wasm` folder
```sh
cp -r assets wasm
```

## Publish

- Start a local web server

```sh
basic-http-server wasm
```

- Publish to itch.io

```sh
zip wasm.zip wasm/**/*
```


## Licence

Code is licensed under MIT or Apache-2.0.
Assets are licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).

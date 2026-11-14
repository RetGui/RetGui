set -e


#rustup target add wasm32-unknown-unknown
#cargo install -f wasm-bindgen-cli
#cargo install simple-http-server

cargo build --target wasm32-unknown-unknown --package gallery --no-default-features --profile web-release

wasm-bindgen target/wasm32-unknown-unknown/web-release/gallery.wasm --target web --no-typescript --out-dir target/generated --out-name gallery

simple-http-server . -c wasm,html,js -i --coep --coop --ip 0.0.0.0

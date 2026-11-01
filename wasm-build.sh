set -e


#rustup target add wasm32-unknown-unknown
#cargo install -f wasm-bindgen-cli
#cargo install simple-http-server

cargo build --target wasm32-unknown-unknown --package gallery --no-default-features

wasm-bindgen target/wasm32-unknown-unknown/debug/gallery.wasm --target web --no-typescript --out-dir target/generated --out-name gallery --debug --keep-debug

simple-http-server . -c wasm,html,js -i --coep --coop --ip 0.0.0.0

cargo build --target wasm32-unknown-unknown --release && near dev-deploy --wasmFile ./target/wasm32-unknown-unknown/release/contract.wasm

## With ABI

cargo near build --release --embed-abi && near dev-deploy --wasmFile ./target/near/proof_of_height.wasm


# requires cargo, dot, kitten and /tmp folder
cargo run -- -q "$1" > /tmp/cargo_output || { cat /tmp/cargo_output; exit 1; }
dot -Tpng < /tmp/cargo_output | kitten icat

# requires cargo, dot, and icat
set -o pipefail
cargo run -- -q "$1" | dot -Tpng | kitten icat

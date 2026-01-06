# 
# Description:
# This script is used to visualize the AST of a ClassQL query.
# It requires cargo, graphviz, kitten and /tmp folder
#
# Parameters:
# --- ---
# $1 -> The query to visualize
# --- ---
#
# Returns:
# --- ---
# Visualizes the AST of the query as a PNG image
# --- ---
#
cargo run -- -q "$1" > /tmp/cargo_output || { cat /tmp/cargo_output; exit 1; }
dot -Tpng < /tmp/cargo_output | kitten icat

# Clean up the temporary file
rm /tmp/cargo_output
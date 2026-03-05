set SCRIPT_DIR (dirname (status --current-filename))
cd $SCRIPT_DIR

echo "Running Rust tests..."
cargo test --no-default-features; or exit 1

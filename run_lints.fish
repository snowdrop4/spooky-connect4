set SCRIPT_DIR (dirname (status --current-filename))
cd $SCRIPT_DIR

# ------------------------------------------------------------------------------
# Python
# ------------------------------------------------------------------------------

uv run ruff format .; or exit 1
uv run ruff check --fix --unsafe-fixes .; or exit 1

# ------------------------------------------------------------------------------
# Rust
# ------------------------------------------------------------------------------

cargo fmt; or exit 1
cargo clippy --fix --allow-dirty; or exit 1

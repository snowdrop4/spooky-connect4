set SCRIPT_DIR (dirname (status --current-filename))
cd $SCRIPT_DIR

# Install dependencies
echo "Installing development dependencies..."
uv sync --dev; or exit 1

# Build the library
echo "Building spooky_connect4 Python library..."
uv pip install -e .; or exit 1

# Run the tests
echo "Running Python tests..."
uv run python -m pytest -v; or exit 1

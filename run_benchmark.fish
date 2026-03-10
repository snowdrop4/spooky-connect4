set SCRIPT_DIR (dirname (status --current-filename))
cd $SCRIPT_DIR

cargo bench --features bench; or exit 1

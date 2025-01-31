#!/bin/sh

set -e  # Exit immediately on errors

# Debug: Print the current directory and its contents
echo "Current directory: $(pwd)"
echo "Contents of the working directory:"
ls -l

# Run migrations
echo "Running migrations..."
if ! diesel migration run --database-url=$DATABASE_URL --migration-dir=./migrations; then
  echo "Migration failed! Entering debug shell."
  exec sh
fi

# Start the application with cargo-watch
echo "Starting application with cargo-watch..."
exec cargo watch -x 'run --release'
# Use the Rust official image
FROM rust:1.84.0

# Set the working directory
WORKDIR /app

# Install system dependencies for Diesel, PostgreSQL, and debugging tools
RUN apt-get update && apt-get install -y \
    libpq-dev \
    postgresql-client \
    iputils-ping \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Copy project files
COPY Cargo.toml Cargo.lock ./
COPY diesel.toml rustfmt.toml ./
COPY migrations ./migrations
COPY src ./src

# Install Diesel CLI for managing migrations
RUN cargo install diesel_cli --no-default-features --features postgres

# Install cargo-watch for automatic reloading
RUN cargo install cargo-watch

# Build the Rust project in release mode
RUN cargo build --release

# Debug: List the contents of the target directory to ensure the binary is built
RUN ls -l ./target/release

# Expose the application port
EXPOSE 8080

# Copy the entrypoint script
COPY entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

# Set the entrypoint to use the entrypoint script
ENTRYPOINT ["/app/entrypoint.sh"]
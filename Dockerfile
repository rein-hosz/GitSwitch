FROM rust:latest

# Set working directory
WORKDIR /app

# Copy source code
COPY . .

# Install dependencies if needed
RUN apt-get update && apt-get install -y openssh-client

# Build the binary
RUN cargo build --release

# Set entrypoint
ENTRYPOINT ["/app/target/release/git_switch"]

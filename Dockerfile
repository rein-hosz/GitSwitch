# ---- Builder Stage ----
# Use the official Rust image as a builder.
# 'latest' tag usually points to the latest stable Rust version.
FROM rust:latest AS builder

# Install git, as it's needed by build.rs to determine the long version string.
RUN apt-get update && apt-get install -y --no-install-recommends git && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy manifests and build script.
# Copying these first allows Docker to cache the dependency fetching layer.
COPY Cargo.toml Cargo.lock build.rs ./

# Copy the .git directory if you want build.rs to include git information in the version.
# Note: If .git is in your .dockerignore, this line will have no effect unless .dockerignore is modified.
# If .git is not available, your build.rs gracefully falls back to CARGO_PKG_VERSION.
COPY .git ./.git

# Create a dummy main.rs to build only dependencies.
# This helps in caching dependencies separately from source code changes.
RUN mkdir src && \
    echo 'fn main() {println!("Dummy main for dep caching. APP_VERSION: {}", env!("APP_VERSION"));}' > src/main.rs

# Build dependencies (this will also compile build.rs and run it).
# The output of build.rs (env vars like APP_VERSION) will be available for the next build step.
RUN cargo build --release

# Copy the rest of the application source code.
COPY src ./src

# Build the application.
# This will use the cached dependencies and recompile only the application code.
RUN cargo build --release

# ---- Runtime Stage ----
# Use a small base image for the runtime environment.
FROM debian:bullseye-slim AS runtime

# Install runtime dependencies: git and openssh-client are essential for git-switch.
# ca-certificates is good practice for network requests (e.g., git clone/fetch over https).
RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    openssh-client \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user and group for security.
# git-switch creates files in the user's home directory.
RUN groupadd --system --gid 1001 appgroup && \
    useradd --system --uid 1001 --gid appgroup --shell /bin/bash --create-home appuser

# Copy the compiled binary from the builder stage to a directory in PATH.
COPY --from=builder /usr/src/app/target/release/git_switch /usr/local/bin/git_switch

# Ensure the binary is executable
RUN chmod +x /usr/local/bin/git_switch

# Switch to the non-root user.
USER appuser

# Set /home/appuser as the home directory for the appuser, which git-switch will use.
ENV HOME=/home/appuser

# Set the entrypoint for the container.
ENTRYPOINT ["git_switch"]

# Optionally, set a default command if no arguments are provided to the entrypoint.
# CMD ["--help"]

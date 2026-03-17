# Use Ubuntu 20.04 for GLIBC compatibility
FROM ubuntu:20.04

# Install dependencies
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    git \
    apt-utils \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"

# Set working directory
WORKDIR /app

# Default command - will be overridden by volume mount
CMD ["cargo", "build", "--release"]

# This dockerfile creates a container with everything preconfigured to use this library

FROM debian:latest

# Update
RUN apt update -y && apt-get autoremove --yes

# Install dependencies
RUN apt install -y  \
    curl \
    git

# RUST
# Install rust as we need it for rust compiler
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal

# Add rust to path
ENV PATH="/root/.cargo/bin:${PATH}"

# Install rust wasm target as we need it for rust -> wasm compiler
RUN rustup target add wasm32-wasi

# Install clang as we need it for c++ compiler
RUN apt install -y \
    clang

# Install wasi-sdk as we need it for c++ -> wasi compiler
RUN curl https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-20/wasi-sdk-20.0-linux.tar.gz -L -o wasi-sdk.tar.gz
RUN tar -xzf wasi-sdk.tar.gz
RUN mv wasi-sdk-20.0 /wasi-sdk && rm wasi-sdk.tar.gz

# Set environment variables for wasi-sdk
ENV WASI_SDK=/wasi-sdk

# Install wasmer as we need it for running the compiled wasm
# RUN curl https://get.wasmer.io -sSfL | sh

# Install python3 as we need it for python compiler
RUN apt install -y \
    python3 \
    python3-pip

# Clean apt cache
RUN rm -rf /var/lib/apt/lists/*
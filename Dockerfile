# Step 1: Use the official Rust image as the build environment
FROM rust:latest AS builder

# Step 2: Create a new directory for the app
WORKDIR /usr/src/era-test-node

# Step 3: Copy the local files to the container
COPY . .

# Step 4: Build the Rust program
RUN cargo build --release

# Use a newer base image for the runtime
FROM ubuntu:latest

ARG CHAIN_ID
ENV CHAIN_ID=${CHAIN_ID}

# Set up dependencies
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Step 7: Set up a user for running the server
ENV USER=era-test-node
RUN useradd -m $USER

# Step 8: Copy the compiled binary from the build container
COPY --from=builder /usr/src/era-test-node/target/release/era_test_node /usr/local/bin/era_test_node

# Step 9: Verify and set execute permissions
RUN ls -l /usr/local/bin/era_test_node  # List permissions for debugging
RUN chmod +x /usr/local/bin/era_test_node && \
    chown -R $USER:$USER /usr/local/bin/era_test_node

# Step 10: Switch to the new user
USER root

# Step 11: Expose the server port
EXPOSE 8011

# Step 12: Explicitly run the server binary with its full path
CMD ["sh", "-c", "/usr/local/bin/era_test_node --chain-id ${CHAIN_ID}"]

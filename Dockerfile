##########################################
# Stage 1: Prepare dependency recipe
##########################################
ARG BUILDPLATFORM
FROM --platform=${BUILDPLATFORM} lukemathwalker/cargo-chef:latest AS chef
WORKDIR /kaze_backend

# Stage to create a dependency recipe (caching)
FROM chef AS planner
# Copy only files that affect dependency resolution
COPY Cargo.toml Cargo.lock build.rs ./
COPY src/ src/
RUN cargo chef prepare

##########################################
# Stage 2: Build the project
##########################################
FROM chef AS builder
# Install system dependency for bindgen
RUN apt-get update && apt-get install -y libclang-dev

# Use the prepared dependency recipe
COPY --from=planner /kaze_backend/recipe.json recipe.json
RUN cargo chef cook --release

# Copy the complete source code and build your project
COPY . .
RUN cargo build --release

# Move the built binary (named "kaze_backend") to a known location
RUN mv target/release/kaze_backend /kaze_backend/

##########################################
# Stage 3: Create the runtime image
##########################################
FROM --platform=${BUILDPLATFORM} debian:stable-slim AS runtime

# RUN apt-get update && apt-get install -y ca-certificates

WORKDIR /kaze_backend

# Copy the compiled binary into a standard location
COPY --from=builder /kaze_backend/kaze_backend /usr/local/bin/kaze_backend

# Copy the native library folder required by your build
COPY --from=builder /kaze_backend/libs/eusign/shared /kaze_backend/libs/eusign/shared
# Make sure the shared library can be found at runtime
ENV LD_LIBRARY_PATH="/kaze_backend/libs/eusign/shared:${LD_LIBRARY_PATH}"

# Expose the port your server listens on (default is 3000)
EXPOSE 3000

# Set the entrypoint to run the server subcommand
ENTRYPOINT ["/usr/local/bin/kaze_backend"]
CMD ["server"]

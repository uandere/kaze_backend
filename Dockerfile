##########################################
# Stage 1: Prepare dependency recipe (caching)
##########################################
ARG BUILDPLATFORM
FROM --platform=${BUILDPLATFORM} lukemathwalker/cargo-chef:latest AS chef
WORKDIR /kaze_backend

FROM chef AS planner
COPY Cargo.toml Cargo.lock build.rs ./
COPY src/ src/
RUN cargo chef prepare

##########################################
# Stage 2: Build the project
##########################################
FROM chef AS builder

RUN apt-get update && apt-get install -y libclang-dev

COPY --from=planner /kaze_backend/recipe.json recipe.json
RUN cargo chef cook --release

COPY . .
RUN cargo build --release

RUN mv target/release/kaze_backend /kaze_backend/

##########################################
# Stage 3: Create the runtime image
##########################################
FROM --platform=${BUILDPLATFORM} debian:stable-slim AS runtime

# RUN apt-get update && apt-get install -y ca-certificates

WORKDIR /kaze_backend

COPY --from=builder /kaze_backend/kaze_backend /usr/local/bin/kaze_backend

COPY --from=builder /kaze_backend/libs/eusign/shared /kaze_backend/libs/eusign/shared

ENV LD_LIBRARY_PATH="/kaze_backend/libs/eusign/shared:${LD_LIBRARY_PATH}"

EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/kaze_backend"]
CMD ["server"]

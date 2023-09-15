FROM rust:1.71.0-buster as build

WORKDIR /usr/src/bundler
COPY bundler .

# Build the application
RUN cargo build --release

# Using Distroless as runtime
FROM gcr.io/distroless/cc-debian10

# Copy the compiled binary and other required files
COPY --from=build /usr/src/bundler/target/release/bundler /usr/local/bin/bundler
COPY --from=build /usr/src/bundler/config /config

# Command to run
CMD ["bundler"]

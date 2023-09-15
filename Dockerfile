FROM rust:1.71.1-buster as build

# Set the environment variables
ENV ADMIN=ska1296@gmail.com
ENV RUN_ENV=Staging
ENV DATABASE_URL=postgres://walrusx:HorseGram9@host.docker.internal/toad
ENV PROVIDER_API_KEY='some_key'
ENV VERIFYING_PAYMASTER_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
ENV WALLET_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
ENV GOOGLE_APPLICATION_CREDENTIALS=/usr/src/bundler/toad-cash-auth.json

WORKDIR /usr/src/bundler
COPY bundler .

# Build the application
RUN cargo build --release

# Using Distroless as runtime
FROM gcr.io/distroless/cc-debian10

# Copy the environment variables to the new stage
ENV ADMIN=ska1296@gmail.com
ENV RUN_ENV=Staging
ENV DATABASE_URL=postgres://walrusx:HorseGram9@host.docker.internal/toad
ENV PROVIDER_API_KEY='some_key'
ENV VERIFYING_PAYMASTER_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
ENV WALLET_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
ENV GOOGLE_APPLICATION_CREDENTIALS=/usr/local/bin/toad-cash-auth.json

# Copy the compiled binary and other required files
COPY --from=build /usr/src/bundler/target/release/bundler /usr/local/bin/bundler
COPY --from=build /usr/src/bundler/toad-cash-auth.json /usr/local/bin/toad-cash-auth.json
COPY --from=build /usr/src/bundler/config /config

# Command to run
CMD ["bundler"]

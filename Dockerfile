FROM rust:1.71.0

# Create a working directory
WORKDIR /app

# Copy the 'bundler' project files, including the .env file, into the container
COPY bundler .

# Build the Rust application
RUN cargo build --release

# Expose port 9010
EXPOSE 9090

# Start your Actix-Web application
CMD ["./target/release/bundler"]


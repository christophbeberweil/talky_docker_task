# Dockerfile for creating a statically-linked Rust application using docker's
# multi-stage build feature. This also leverages the docker build cache to avoid
# re-downloading dependencies if they have not changed.
FROM rust:1.74-buster AS build
WORKDIR /usr/src

# Download the target for static linking.
#RUN rustup target add x86_64-unknown-linux-musl

# Create a dummy project and build the app's dependencies.
# If the Cargo.toml or Cargo.lock files have not changed,
# we can use the docker build cache and skip these (typically slow) steps.
RUN USER=root cargo new talky
WORKDIR /usr/src/talky
RUN rm -r /usr/src/talky/src

COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Copy the source and build the application.
# COPY src ./src
# COPY migrations ./migrations
# COPY templates ./templates

#RUN cargo install --target x86_64-unknown-linux-musl --path .
RUN cargo install --path .

# Copy the statically-linked binary into a scratch container.
FROM debian


RUN apt-get update \
    && apt-get install -y curl\
    && rm -rf /var/lib/apt/lists/*

HEALTHCHECK CMD curl --fail http://localhost:3000/ || exit 1


RUN mkdir /app
# RUN mkdir /app/static
RUN chown -R 1000:1000 /app
WORKDIR /app
COPY --from=build /usr/local/cargo/bin/talky /app/
# COPY ./static/ /app/static/
USER 1000
CMD ["./talky"]

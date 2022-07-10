#https://blog.logrocket.com/packaging-a-rust-web-service-using-docker/
FROM rust:latest as build

RUN USER=root cargo new --bin axum-demo
WORKDIR /axum-demo

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release

RUN rm src/*.rs
COPY ./src ./src

RUN rm ./target/release/deps/axum_demo*
RUN cargo build --release

FROM rust:slim-buster
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 8080

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

# copy the build artifact from the build stage
COPY --from=build /axum-demo/target/release/axum-demo ${APP}/axum-demo

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

# set the startup command to run your binary
CMD ["./axum-demo"]
#-------------------------------------------------------------------------------------------------------------
# Copyright (c) Microsoft Corporation. All rights reserved.
# Licensed under the MIT License. See https://go.microsoft.com/fwlink/?linkid=2090316 for license information.
#-------------------------------------------------------------------------------------------------------------

FROM ubuntu:20.04

# Avoid warnings by switching to noninteractive
ENV DEBIAN_FRONTEND=noninteractive

ARG SHELL=fish
# Configure apt and install packages
RUN    apt-get update \
    && apt-get -y install apt-utils \
                          build-essential \
                          cmake \
                          curl \
                          dialog \
                          git \
                          iproute2 \
                          libatlas-base-dev \
                          liblapack-dev \
                          libsqlite3-dev \
                          libssl-dev \
                          lldb \
                          lsb-release \
                          pkg-config \
                          procps \
                          python3 \
                          sudo \
                          $SHELL 2>&1 \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Switch back to dialog for any ad-hoc use of apt-get
ENV DEBIAN_FRONTEND=

# Install dlib
ARG DLIB_VERSION=19.21
RUN    curl -L https://github.com/davisking/dlib/archive/v${DLIB_VERSION}.tar.gz | tar -xzC /tmp \
    && cmake -S /tmp/dlib-${DLIB_VERSION} -B /tmp/dlib-${DLIB_VERSION}/build \
    && cmake --build /tmp/dlib-${DLIB_VERSION}/build --target install \
    && rm -r /tmp/dlib-${DLIB_VERSION}

# Create a non-root user to use
ARG USER=user
ARG UID=1000
ARG GID=$UID
ARG DOCKER_GID=100
RUN    groupadd --gid $GID $USER \
    && groupadd --gid $DOCKER_GID docker \
    && useradd -m -s /bin/$SHELL --uid $UID --gid $GID -G $DOCKER_GID $USER \
    # Add sudo support for the non-root user
    && echo $USER ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USER\
    && chmod 0440 /etc/sudoers.d/$USER

USER $USER

# Install Rust
ARG RUST_VERSION=1.49.0
ENV RUSTUP_HOME=/home/$USER/.rustup
ENV CARGO_HOME=/home/$USER/.cargo
ENV PATH="${PATH}:${CARGO_HOME}/bin"
RUN    mkdir ${RUSTUP_HOME} \
    # Install rust
    && curl https://sh.rustup.rs -sSf | bash -s -- \
      -y \
      --no-modify-path \
      --default-toolchain ${RUST_VERSION} \
      --component rls \
      --component rust-analysis \
      --component rust-src \
    # Add the wasm32 target
    && rustup target add wasm32-unknown-unknown \
    # Install diesel
    && cargo install diesel_cli --no-default-features --features sqlite \
    # Install wasm bindgen
    && cargo install wasm-bindgen-cli

WORKDIR /home/$USER

CMD ["tail", "-f", "/dev/null"]

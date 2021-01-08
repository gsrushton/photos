FROM ubuntu:20.04

# Avoid warnings by switching to noninteractive
ENV DEBIAN_FRONTEND=noninteractive

ARG SHELL=fish
# Configure apt and install packages
RUN    apt-get update \
    && apt-get -y install libatlas3-base \
                          libsqlite3-0 \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Switch back to dialog for any ad-hoc use of apt-get
ENV DEBIAN_FRONTEND=

COPY photosd /usr/local/bin
COPY share /usr/local/share/photos

# Create a non-root user to use
ARG USER=photosd
ARG UID=935
RUN useradd -r -u 935 -U $USER

ENV PHOTOSD_LOG=info

USER $USER

WORKDIR /

CMD ["photosd"]

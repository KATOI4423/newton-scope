FROM ubuntu:24.04

ENV TZ=Asia/Tokyo

# Install required packages, and remove unnecessary packages to reduse docker container size
RUN apt-get upgrade -y && apt-get update -qq \
 && apt-get install -yq --no-install-recommends \
    bash \
    build-essential \
    ca-certificates \
    curl \
    libgtk-3-dev \
    libcairo2-dev \
    libjavascriptcoregtk-4.1-dev \
    libsoup-3.0-dev \
    libwebkit2gtk-4.1-dev \
    make \
    pkg-config \
    sudo \
    tar \
    tzdata \
 && apt-get clean \
 && rm -rf /var/cache/apt/archives/* \
 && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp* \
 && apt-get autoremove -y \
;

ARG USERNAME=ubuntu
ARG USER_UID=1000
ARG USER_GID=1000

RUN if ! getent group ${USER_GID} >/dev/null; then \
        groupadd --gid ${USER_GID} ${USERNAME}; \
    fi \
 && if ! id -u ${USERNAME} >/dev/null 2>&1; then \
        useradd -m -s /bin/bash --uid ${USER_UID} --gid ${USER_GID} ${USERNAME}; \
    fi \
 && echo "${USERNAME} ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/nopasswd

USER ${USERNAME}
WORKDIR /home/${USERNAME}

# Install Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/home/${USERNAME}/.cargo/bin:${PATH}"

WORKDIR /work

CMD ["bash", "-c", "${BUILD_CMD}"]

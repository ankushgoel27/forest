# This Dockerfile is for the main forest binary
# 
# Build and run locally:
# ```
# docker build -t forest:latest -f ./Dockerfile .
# docker run --init -it forest
# ```
# 
# Build and manually push to Github Container Registry (see https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry)
# ```
# docker build -t ghcr.io/chainsafe/forest:latest .
# docker push ghcr.io/chainsafe/forest:latest
# ```

##
# Build stage
# Use github action runner cached images to avoid being rate limited
# https://github.com/actions/runner-images/blob/main/images/linux/Ubuntu2004-Readme.md#cached-docker-images
## 

# Cross-compilation helpers
# https://github.com/tonistiigi/xx
# FROM --platform=$BUILDPLATFORM tonistiigi/xx:1.5.0 AS xx

# FROM --platform=$BUILDPLATFORM ubuntu:22.04 AS build-env
FROM ubuntu:22.04 AS build-env
SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# export TARGETPLATFORM TARGETOS and TARGETARCH
ARG TARGETPLATFORM
ARG TARGETOS
ARG TARGETARCH

# install dependencies
RUN apt-get update && \
    apt-get install --no-install-recommends -y build-essential clang curl git ca-certificates
RUN update-ca-certificates

# install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --no-modify-path --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

# install Go. Note that F3 sidecar is not enabled for arm64, see `build.rs`
RUN curl -sSL https://raw.githubusercontent.com/voidint/g/master/install.sh | bash
RUN "${HOME}/.g/bin/g" install 1.21 && ln -sf "${HOME}/.g/go/bin/go" /usr/local/bin/go && go version

# Opt out F3 sidecar FFI build for arm64 for now
# RUN if [ "${TARGETARCH}" != "amd64" ] ;then echo FOREST_F3_SIDECAR_FFI_BUILD_OPT_OUT=1 >>/etc/environment ; fi
# RUN cat /etc/environment && echo "FOREST_F3_SIDECAR_FFI_BUILD_OPT_OUT=${FOREST_F3_SIDECAR_FFI_BUILD_OPT_OUT}"

# Copy the cross-compilation scripts 
# COPY --from=xx / /

# Install those packages for the target architecture
# RUN xx-apt-get update && \
#     xx-apt-get install -y libc6-dev g++

WORKDIR /forest
COPY . .

ENV GOOS=${TARGETOS}
ENV GOARCH=${TARGETARCH}

# Install Forest. Move it out of the cache for the prod image.
RUN --mount=type=cache,sharing=private,target=/root/.cargo/registry \
    --mount=type=cache,sharing=private,target=/root/.rustup \
    --mount=type=cache,sharing=private,target=/forest/target \
    make install && \
    mkdir /forest_out && \
    cp /root/.cargo/bin/forest* /forest_out

##
# Prod image for forest binary
# Use github action runner cached images to avoid being rate limited
# https://github.com/actions/runner-images/blob/main/images/linux/Ubuntu2004-Readme.md#cached-docker-images
##
# A slim image contains only forest binaries
FROM ubuntu:22.04 AS slim-image

ENV DEBIAN_FRONTEND="noninteractive"
# Install binary dependencies
RUN apt-get update && \
    apt-get install --no-install-recommends -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*
RUN update-ca-certificates

# Copy forest daemon and cli binaries from the build-env
COPY --from=build-env /forest_out/* /usr/local/bin/

# Basic verification of dynamically linked dependencies
RUN forest -V && forest-cli -V && forest-tool -V && forest-wallet -V

ENTRYPOINT ["forest"]

# A fat image contains forest binaries and fil proof parameter files under $FIL_PROOFS_PARAMETER_CACHE
FROM slim-image AS fat-image

# Move FIL_PROOFS_PARAMETER_CACHE out of forest data dir since users always need to mount the data dir
ENV FIL_PROOFS_PARAMETER_CACHE="/var/tmp/filecoin-proof-parameters"

# Populate $FIL_PROOFS_PARAMETER_CACHE
RUN forest-tool fetch-params --keys

# Cache actor bundle in the image
ENV FOREST_ACTOR_BUNDLE_PATH="/var/tmp/forest_actor_bundle.car.zst"

# Populate $FOREST_ACTOR_BUNDLE_PATH
RUN forest-tool state-migration actor-bundle $FOREST_ACTOR_BUNDLE_PATH

ENTRYPOINT ["forest"]

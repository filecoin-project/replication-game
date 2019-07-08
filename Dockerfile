# build with: docker build --tag replication-game .
# run with:   docker run -it -v `pwd`:/code/ replication-game

FROM debian:stretch

RUN apt-get update && \
    apt-get install -y curl file gcc g++ git make openssh-client \
    autoconf automake cmake libtool libcurl4-openssl-dev libssl-dev \
    libelf-dev libdw-dev binutils-dev zlib1g-dev libiberty-dev wget \
    xz-utils pkg-config python clang jq

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain `cat rust-toolchain`

ENV PATH "$PATH:/root/.cargo/bin"
ENV RUSTFLAGS "-C link-dead-code"
ENV CFG_RELEASE_CHANNEL "nightly"

RUN bash -l -c 'echo $(rustc --print sysroot)/lib >> /etc/ld.so.conf'
RUN bash -l -c 'echo /usr/local/lib >> /etc/ld.so.conf'
RUN ldconfig

WORKDIR /code
CMD ["/code/main.sh"]

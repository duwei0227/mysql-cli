FROM centos:7

# 安装基础工具
RUN yum install -y epel-release && \
    yum install -y gcc gcc-c++ openssl-devel make git curl which && \
    yum clean all

# 使用 Software Collections Library 安装较新工具链
RUN yum install -y centos-release-scl && \
    yum install -y devtoolset-9 devtoolset-9-libatomic-devel

# 安装 Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

# 设置环境变量
ENV PATH="/root/.cargo/bin:$PATH"
ENV CC="/opt/rh/devtoolset-9/root/usr/bin/gcc"
ENV CXX="/opt/rh/devtoolset-9/root/usr/bin/g++"

WORKDIR /build
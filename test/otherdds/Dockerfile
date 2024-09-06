FROM ros:humble-ros-core-jammy

RUN apt-get update -y

RUN apt-get install -y ros-humble-rmw-fastrtps-cpp ros-humble-rmw-cyclonedds-cpp
# ENV RMW_IMPLEMENTATION rmw_fastrtps_cpp
ENV RMW_IMPLEMENTATION rmw_cyclonedds_cpp

RUN apt-get install -y \
    build-essential curl gnupg2 lsb-release python3-pip git

RUN pip install git+https://github.com/tier4/colcon-cargo.git
RUN pip install git+https://github.com/colcon/colcon-ros-cargo.git

# rust
ENV RUST_VERSION stable
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_VERSION}
ENV PATH $PATH:$HOME/.cargo/bin

WORKDIR /root
RUN git clone https://github.com/tier4/cargo-ament-build.git
WORKDIR /root/cargo-ament-build
RUN . /root/.cargo/env && cargo install --path .

RUN . /root/.cargo/env && cargo install --git https://github.com/tier4/ros2msg_to_rs.git

# FastDDS
RUN mkdir /opt/Fast-DDS
WORKDIR /opt/Fast-DDS
RUN apt-get install -y \
    wget libasio-dev libtinyxml2-dev
RUN pip install -U colcon-common-extensions vcstool
RUN wget https://raw.githubusercontent.com/eProsima/Fast-DDS/master/fastdds.repos
RUN mkdir src
RUN vcs import src < fastdds.repos
RUN colcon build --packages-up-to fastdds

WORKDIR /home

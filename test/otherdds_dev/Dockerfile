FROM fastdds_img:latest AS builder

SHELL ["/bin/bash", "-c"]

WORKDIR /home

# build ShapesDemo
RUN apt-get update && apt-get install -y qtbase5-dev
COPY refarence/ShapesDemo /home/ShapesDemo
WORKDIR /home/ShapesDemo
RUN . /opt/Fast-DDS/install/setup.bash && colcon build

FROM ros:humble-ros-core-jammy AS runner

WORKDIR /home
RUN apt-get update && apt-get install -y libtinyxml2-dev \
    iproute2 tmux curl netcat qtbase5-dev build-essential \
    gnupg2 lsb-release python3-pip git \
    ros-humble-rmw-fastrtps-cpp ros-humble-rmw-cyclonedds-cpp

# ENV RMW_IMPLEMENTATION rmw_fastrtps_cpp
ENV RMW_IMPLEMENTATION rmw_cyclonedds_cpp


ENV USERNAME foo
RUN mkdir /etc/sudoers.d
RUN useradd -m $USERNAME && \
        echo "$USERNAME:$USERNAME" | chpasswd && \
        usermod --shell /bin/bash $USERNAME && \
        usermod -aG sudo $USERNAME && \
        echo "$USERNAME ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers.d/$USERNAME && \
        chmod 0440 /etc/sudoers.d/$USERNAME && \
        # Replace 1000 with your user/group id
        usermod  --uid 1000 $USERNAME && \
        groupmod --gid 1000 $USERNAME

RUN pip install git+https://github.com/tier4/colcon-cargo.git
RUN pip install git+https://github.com/colcon/colcon-ros-cargo.git

RUN cp /root/.bashrc /home/foo/ && cp /root/.profile /home/foo/
RUN echo ". /opt/ros/humble/setup.bash" >> /home/foo/.bashrc
USER foo

# rust
ENV RUST_VERSION stable
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_VERSION}
ENV PATH $PATH:$HOME/.cargo/bin

WORKDIR /home/foo
RUN git clone https://github.com/tier4/cargo-ament-build.git
WORKDIR /home/foo/cargo-ament-build
RUN . /home/foo/.cargo/env && cargo install --path .

RUN . /home/foo/.cargo/env && cargo install --git https://github.com/tier4/ros2msg_to_rs.git

# copy fastdds from fastdds_img
COPY --from=builder /opt/Fast-DDS /opt/Fast-DDS
# copy ShapesDemo from fastdds_img
COPY --from=builder /home/ShapesDemo /home/foo/ShapesDemo

WORKDIR /home/foo



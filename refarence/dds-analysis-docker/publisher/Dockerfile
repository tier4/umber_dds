FROM fastdds_img:latest AS builder

WORKDIR /home
RUN mkdir hw_workspace
SHELL ["/bin/bash", "-c"]
# build DDSHelloWorld
RUN --mount=type=bind,src=./dds-analysis-docker/workspace_DDSHelloWorld,target=./hw_workspace mkdir hw_build && cd hw_build \
    && . /opt/Fast-DDS/install/setup.bash && cmake ../hw_workspace && cmake --build .

# build ShapesDemo
RUN apt-get update && apt-get install -y qtbase5-dev
COPY ./ShapesDemo /home/ShapesDemo
WORKDIR /home/ShapesDemo
RUN . /opt/Fast-DDS/install/setup.bash && colcon build

FROM ubuntu:jammy AS runner

WORKDIR /home
RUN apt-get update && apt-get install -y libtinyxml2-dev \
     qtbase5-dev iproute2 tmux netcat
    # libssl-dev qtbase5-dev iproute2 tmux python3 curl locales netcat python3-pip

# copy fastdds from fastdds_img
COPY --from=builder /opt/Fast-DDS /opt/Fast-DDS
# copy DDSHelloWorld from fastdds_img
COPY --from=builder /home/hw_build /home/DDSHelloWorld
# copy ShapesDemo from fastdds_img
COPY --from=builder /home/ShapesDemo /home/ShapesDemo

WORKDIR /home

RUN mkdir -p UmberDDS/target

ENV USERNAME foo
RUN mkdir /etc/sudoers.d
RUN useradd -M $USERNAME && \
        echo "$USERNAME:$USERNAME" | chpasswd && \
        usermod --shell /bin/bash $USERNAME && \
        usermod -aG sudo $USERNAME && \
        echo "$USERNAME ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers.d/$USERNAME && \
        chmod 0440 /etc/sudoers.d/$USERNAME && \
        # Replace 1000 with your user/group id
        usermod  --uid 1000 $USERNAME && \
        groupmod --gid 1000 $USERNAME

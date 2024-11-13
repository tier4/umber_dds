FROM fastdds_img:latest AS builder

WORKDIR /home
RUN mkdir workspace
SHELL ["/bin/bash", "-c"]
# build DDSHelloWorld
RUN --mount=type=bind,src=./workspace_DDSHelloWorld,target=./workspace mkdir build && cd build \
    && . /opt/Fast-DDS/install/setup.bash && cmake ../workspace && cmake --build .

FROM ubuntu:jammy AS runner

RUN apt-get update && apt-get install -y \
    iproute2 tmux gdb lsof libtinyxml2-dev

# copy fastdds from fastdds_img
COPY --from=builder /opt/Fast-DDS /opt/Fast-DDS
# copy DDSHelloWorld from fastdds_img
COPY --from=builder /home/build /home/DDSHelloWorld

WORKDIR /home

RUN mkdir -p RustDDS UmberDDS/target rust-subscriber

FROM fastdds_img:latest AS fast_builder

WORKDIR /home
RUN mkdir workspace
SHELL ["/bin/bash", "-c"]
# build DDSHelloWorld
RUN --mount=type=bind,src=./dds-analysis-docker/workspace_DDSHelloWorld,target=./workspace mkdir build && cd build \
    && . /opt/Fast-DDS/install/setup.bash && cmake ../workspace && cmake --build .


FROM cyclonedds_img:latest AS cyclone_builder

WORKDIR /home
# build shapes_demo_cyclonedds
RUN --mount=type=bind,src=./dds-analysis-docker/cyclone_shapesdemo,target=./cyclone_shapesdemo mkdir cyclone_build && cd cyclone_build \
    && cmake ../cyclone_shapesdemo && cmake --build .


FROM ubuntu:jammy AS runner

RUN apt-get update && apt-get install -y \
    iproute2 tmux gdb lsof libtinyxml2-dev

# copy fastdds from fastdds_img
COPY --from=fast_builder /opt/Fast-DDS /opt/Fast-DDS
# copy DDSHelloWorld from fastdds_img
COPY --from=fast_builder /home/build /home/DDSHelloWorld
# copy cyclonedds from cyclonedds_img
COPY --from=cyclone_builder /opt/cyclonedds /opt/cyclonedds
# copy cyclonedds from cyclone_shapedemo
COPY --from=cyclone_builder /home/cyclone_build /home/cyclone_shapesdemo

WORKDIR /home

RUN mkdir -p RustDDS UmberDDS/target rust-subscriber

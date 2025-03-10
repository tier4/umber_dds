FROM cyclonedds_img:latest AS cyclone_installer

WORKDIR /home
# build shapes_demo_cyclonedds
RUN --mount=type=bind,src=./shapes_demo_cyclonedds,target=./shapes_demo_cyclonedds mkdir build && cd build \
   && cmake ../shapes_demo_cyclonedds && cmake --build .

FROM fastdds_img:latest AS fast_installer

FROM ubuntu:jammy AS runner

RUN apt-get update && apt-get install -y gdb

# copy fastdds from fastdds_img
COPY --from=fast_installer /opt/Fast-DDS /opt/Fast-DDS
# copy cyclonedds from cyclonedds_img
COPY --from=cyclone_installer /opt/cyclonedds /opt/cyclonedds
# copy shapes_demo_cyclonedds from cyclonedds_img
COPY --from=cyclone_installer /home/build /home/shapes_demo_cyclonedds

WORKDIR /home

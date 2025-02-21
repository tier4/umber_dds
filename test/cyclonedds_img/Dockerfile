FROM ubuntu:jammy

RUN apt-get update && apt-get install -y \
   wget libasio-dev libtinyxml2-dev python3 python3-pip git cmake gcc g++

# install FastDDS
WORKDIR /root
RUN git clone https://github.com/eclipse-cyclonedds/cyclonedds.git
WORKDIR /root/cyclonedds/build
RUN cmake -DCMAKE_INSTALL_PREFIX=/opt/cyclonedds -DBUILD_EXAMPLES=ON ..
RUN cmake --build . --parallel
RUN cmake --build . --target install
WORKDIR /home

ENV PATH=$PATH:/opt/cyclonedds/bin

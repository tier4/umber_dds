FROM ubuntu:jammy

RUN apt-get update && apt-get install -y \
   wget libasio-dev libtinyxml2-dev python3 python3-pip git cmake g++

# install FastDDS
RUN mkdir /opt/Fast-DDS
WORKDIR /opt/Fast-DDS
RUN pip install -U colcon-common-extensions vcstool
RUN wget https://raw.githubusercontent.com/eProsima/Fast-DDS/master/fastdds.repos
RUN mkdir src
RUN vcs import src < fastdds.repos
RUN colcon build --packages-up-to fastdds


FROM ubuntu:jammy

WORKDIR /home
RUN apt-get update && apt-get install -y libtinyxml2-dev \
libssl-dev iproute2 tmux python3 curl locales netcat
# set locale
RUN locale-gen en_US en_US.UTF-8
RUN update-locale
ENV LC_ALL=en_US.UTF-8
ENV LANG=en_US.UTF-8
ENV LANG=en_US.UTF-8

# install ros2
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get install -y software-properties-common && add-apt-repository universe
RUN curl -sSL https://raw.githubusercontent.com/ros/rosdistro/master/ros.key -o /usr/share/keyrings/ros-archive-keyring.gpg
RUN echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/ros-archive-keyring.gpg] http://packages.ros.org/ros2/ubuntu $(. /etc/os-release && echo $UBUNTU_CODENAME) main" | tee /etc/apt/sources.list.d/ros2.list > /dev/null
RUN apt-get update && apt-get install -y \
  ros-humble-desktop

# install FastDDS
RUN apt-get install -y \
    wget libasio-dev libtinyxml2-dev python3-pip git g++ cmake

WORKDIR /opt
RUN mkdir Fast-DDS
WORKDIR /opt/Fast-DDS
RUN pip install -U colcon-common-extensions vcstool
RUN wget https://raw.githubusercontent.com/eProsima/Fast-DDS/master/fastdds.repos
RUN mkdir src
RUN vcs import src < fastdds.repos
RUN colcon build --packages-up-to fastdds

WORKDIR /home

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

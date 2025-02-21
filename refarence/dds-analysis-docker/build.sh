#!/bin/bash

if [ "$(sudo docker image ls -q "fastdds_img")" ]; then
    echo fastdds_img is already exist
else
    sudo docker build ../../test/fastdds_img --network host -t fastdds_img
fi

if [ "$(sudo docker image ls -q "cyclonedds_img")" ]; then
    echo cyclonedds_img is already exist
else
    sudo docker build ../../test/cyclonedds_img --network host -t cyclonedds_img
fi

sudo docker compose build

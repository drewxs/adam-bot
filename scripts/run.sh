#!/bin/bash

docker build -t adam .
docker run -it --rm --name adam-running adam

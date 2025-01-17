FROM ubuntu:latest

RUN apt-get update && apt-get install -y \
    curl \
    wget \
    vim \
    && rm -rf /var/lib/apt/lists/*

// Create directory for yt-dlp and download the executable and give it permissions
RUN mkdir -p ~/.local/bin && \
    curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o ~/.local/bin/yt-dlp && \
    chown a+rwx ~/.local/bin/yt-dlp


CMD ["/bin/bash"]

// verify the required packages or and including yt-dlp are downloaded given and per the operating system
// call the command line in the respective operating system


// Copy the local .aws config to the docker for s3 permissions

// Install yt-dlp as an executable on the docker image

// Bundle the rust program and package it in the image
// For the database, provide two options
//  1. provide an s3 path to the database file
//  2. provide the relative database path for outside the docker image
//      https://stackoverflow.com/questions/53107441/giving-docker-access-to-db-file-outside-container

// Set entry point for the program to run as an executable with pass in options

// Why use a docker file?
// 1. portable - can run this program anywhere with the same database, not hardware dependent
//      we want to maintain database state between computers in case we lose one, and not have to
//      refactor the program in case we run on another peice of unsupported hardware (i.e. windows or cloud)
// 2. platform independent
// 3. can recreate upated image and not have to change the database (unless there is a database mutation)

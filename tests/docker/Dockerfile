# ARG GO_IMAGE=golang:1.13-stretch
# FROM $GO_IMAGE
FROM rust:1.57.0
RUN apt-get update && apt-get install -y make

# Copy SDK code into the container
RUN mkdir -p $HOME/algonaut
COPY . $HOME/algonaut
WORKDIR $HOME/algonaut

# Run integration tests
# CMD ["/bin/bash", "-c", "make unit && make integration"]
CMD ["/bin/bash", "-c", "make integration"]

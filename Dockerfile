ARG RUST_VERSION="1.64.0"
FROM rust:$RUST_VERSION
RUN apt-get update && apt-get install -y make

# Copy SDK code into the container
RUN mkdir -p $HOME/algonaut
COPY . $HOME/algonaut
WORKDIR $HOME/algonaut

# Run integration tests
# CMD ["/bin/bash", "-c", "make unit && make integration"]
CMD ["/bin/bash", "-c", "make integration"]

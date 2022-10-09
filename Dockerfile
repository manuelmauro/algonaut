ARG RUST_IMAGE
FROM $RUST_IMAGE
RUN apt-get update && apt-get install -y make

# Copy SDK code into the container
RUN mkdir -p $HOME/algonaut
COPY . $HOME/algonaut
WORKDIR $HOME/algonaut

# Run integration tests
# CMD ["/bin/bash", "-c", "make unit && make integration"]
CMD ["/bin/bash", "-c", "make integration"]

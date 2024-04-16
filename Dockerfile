FROM ubuntu:22.04

# show backtraces
ENV RUST_BACKTRACE 1

RUN useradd -m -u 1000 -U -s /bin/sh -d /regionx regionx && \
   	mkdir /data && \
    	chown -R regionx:regionx /data 

ARG PROFILE=release

# copy the compiled binary to the container
COPY ./regionx-node /usr/bin/regionx-node

USER regionx

# check if executable works in this container
RUN /usr/bin/regionx-node --version

# ws_port
EXPOSE 9333 9944 30333 30334

CMD ["/usr/bin/regionx-node"]

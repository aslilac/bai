FROM ghcr.io/gleam-lang/gleam:v1.11.0-erlang-alpine

COPY . /opt/app/

WORKDIR /opt/app/
RUN gleam export erlang-shipment \
	&& mv ./build/erlang-shipment/ /opt/deploy/

WORKDIR /opt/deploy/
CMD ["/opt/deploy/entrypoint.sh", "run"]

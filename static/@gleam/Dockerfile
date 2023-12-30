from ghcr.io/gleam-lang/gleam:v0.33.0-erlang-alpine

copy . /opt/app/

workdir /opt/app/
run gleam export erlang-shipment \
  && mv ./build/erlang-shipment/ /opt/deploy/


workdir /opt/deploy/
cmd ["/opt/deploy/entrypoint.sh", "run"]

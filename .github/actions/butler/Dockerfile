FROM ubuntu:22.04

LABEL com.github.actions.name="Butler Push"
LABEL com.github.actions.description="Publishes releases to Itch.io using Butler"

RUN apt-get update && apt-get install -y curl unzip

RUN curl -L -o butler.zip https://broth.itch.zone/butler/linux-amd64/LATEST/archive/default \
    && unzip butler.zip \
    && mv butler /usr/bin/butler \
    && chmod +x /usr/bin/butler

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]

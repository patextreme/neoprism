FROM nixos/nix
RUN mkdir -p /etc/nix && \
    echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

COPY . /workspace
WORKDIR /workspace
RUN nix build .#indexer-bin -L
RUN nix build .#indexer-ui-assets -L


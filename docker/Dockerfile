FROM nixos/nix as builder
RUN mkdir -p /etc/nix && \
    echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

COPY . /workspace
WORKDIR /workspace
RUN nix build .#indexer-bin -o ./result/indexer-bin -L
RUN nix build .#indexer-ui-assets -o ./result/indexer-ui-assets -L

RUN mkdir /tmp/nix-store-closure
RUN cp -R $(nix-store -qR result/indexer-bin) /tmp/nix-store-closure
RUN cp -R $(nix-store -qR result/indexer-ui-assets) /tmp/nix-store-closure


FROM scratch
WORKDIR /app
COPY --from=builder /tmp/nix-store-closure /nix/store
COPY --from=builder /workspace/result/indexer-bin /app
COPY --from=builder /workspace/result/indexer-ui-assets /app
CMD ["/app/bin/indexer-node"]

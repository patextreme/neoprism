FROM nixos/nix:2.26.4 AS builder
RUN mkdir -p /etc/nix && \
    echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

COPY . /workspace
WORKDIR /workspace
RUN nix build .#neoprism-bin -o ./result/neoprism-bin -L
RUN nix build .#ui-assets -o ./result/ui-assets -L

RUN mkdir /tmp/nix-store-closure
RUN cp -R $(nix-store -qR result/neoprism-bin) /tmp/nix-store-closure
RUN cp -R $(nix-store -qR result/ui-assets) /tmp/nix-store-closure


FROM scratch
WORKDIR /app
COPY --from=builder /tmp/nix-store-closure /nix/store
COPY --from=builder /workspace/result/neoprism-bin /app
COPY --from=builder /workspace/result/ui-assets /app
CMD ["/app/bin/indexer-node"]

# Stage 1 — Builder
# `docker build --platform linux/arm64` runs this stage as arm64 (natively on
# Apple Silicon, or via QEMU elsewhere), so no cross-compiler is needed.
FROM rust:1.83-bookworm AS builder

WORKDIR /workspace
COPY . .

# stack.toml is required at compile time by include_str! in stack.rs.
# Fall back to the example file if it is absent from the build context.
RUN test -f stack.toml || cp stack.example.toml stack.toml

RUN cargo build --release --package deploy-baba-ui

# Stage 2 — Runtime
FROM public.ecr.aws/lambda/provided:al2023-arm64

# Rename to `bootstrap` — the expected entry point for provided runtimes.
COPY --from=builder /workspace/target/release/deploy-baba-ui /var/task/bootstrap

CMD ["bootstrap"]

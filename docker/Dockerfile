# Runs webqa and its browser tests on Linux without installing Chromium's OS
# libraries on the host (docs/container.md). The base image has the browsers
# of one Playwright version; it must be the one playwright-rs drives, which
# crates/webqa-browser/tests/container.rs checks.
FROM mcr.microsoft.com/playwright:v1.64.0-noble

# cargo needs a C linker; openssl-sys (under playwright-rs) needs OpenSSL.
# python3 runs bridge/; the base image has it, but nothing promises it stays.
RUN apt-get update \
	&& apt-get install -y --no-install-recommends build-essential libssl-dev pkg-config python3 \
	&& rm -rf /var/lib/apt/lists/*

# The repository is mounted at /work when the container runs. The tools go
# to /work/.aqua as with scripts/setup.sh; `docker run -v /work/.aqua` puts
# this copy over the host's. Build output stays in a volume at /work/target.
ENV RUSTUP_HOME=/opt/rustup \
	CARGO_HOME=/opt/cargo \
	AQUA_ROOT_DIR=/work/.aqua \
	AQUA_CONFIG=/work/.config/aqua/aqua.yaml \
	AQUA_POLICY_CONFIG=/work/.config/aqua/policy.yaml \
	PLAYWRIGHT_DRIVER_CACHE_DIR=/work/target/playwright-driver \
	PATH=/work/.aqua/bin:/opt/cargo/bin:$PATH
WORKDIR /work

COPY rust-toolchain.toml ./
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
	| sh -s -- -y --no-modify-path --default-toolchain none \
	&& rustup toolchain install

COPY scripts/install-aqua.sh scripts/
COPY .config/aqua .config/aqua/
RUN sh scripts/install-aqua.sh \
	&& aqua policy allow .config/aqua/policy.yaml \
	&& aqua install

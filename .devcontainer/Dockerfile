FROM rust:1.75.0

# Install MDBook and other CLI tools
RUN rustup component add rustfmt clippy && \
    cargo install mdbook --version 0.4.* && \
    cargo install mdbook-admonish --version 1.*
    
# Tell git to trust "dubious" ownership
RUN git config --global --add safe.directory /repository

# Entry directory for non-vscode containers based on this image
WORKDIR /repository
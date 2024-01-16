FROM debian:bookworm

# Locked versions
ARG RUST_VERSION_FULL=1.75.0 \
    MDBOOK_VERSION=0.4.* \
    MDBOOK_ADMONISH_VERSION=1.*

# Update package repositories and existing packages
RUN apt update && \
    apt upgrade -y

# Install basic apt packages
RUN apt install -y \
    git \
    curl \
    gnupg \
    lsb-release \
    zsh \
    build-essential

# Install Docker with socket mounted from host
# Since this is just a Dev Container, mounting the root socket will be safe
ENV DOCKER_HOST=unix:///var/run/docker-host.sock
RUN curl -fsSL https://download.docker.com/linux/debian/gpg | apt-key add - && \
    echo "deb [arch=amd64] https://download.docker.com/linux/debian $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null && \
    apt update && \
    apt install -y docker-ce docker-ce-cli containerd.io && \
    if getent group docker > /dev/null 2>&1; then \
    echo "Docker group exists"; \
    else \
    groupadd docker; \
    fi && \
    usermod -aG docker root

# Install Zsh with Oh My Zsh with plugins and set as default shell
RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" --unattended && \
    git clone https://github.com/zsh-users/zsh-autosuggestions $HOME/.oh-my-zsh/custom/plugins/zsh-autosuggestions && \
    git clone https://github.com/zsh-users/zsh-syntax-highlighting $HOME/.oh-my-zsh/custom/plugins/zsh-syntax-highlighting && \
    git clone https://github.com/zsh-users/zsh-history-substring-search $HOME/.oh-my-zsh/custom/plugins/zsh-history-substring-search && \
    sed -i 's/plugins=(git)/plugins=(git zsh-autosuggestions zsh-syntax-highlighting zsh-history-substring-search history aliases sudo themes docker nmap kubectl)/' $HOME/.zshrc && \
    sed -i 's/ZSH_THEME="robbyrussell"/ZSH_THEME="af-magic"/' $HOME/.zshrc && \
    chsh -s /usr/bin/zsh

# Install Rust with targets and CLI tools
ENV PATH="/root/.cargo/bin:${PATH}"
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_VERSION_FULL} && \
    rustup component add rustfmt clippy && \
    cargo install mdbook --version ${MDBOOK_VERSION} && \
    cargo install mdbook-admonish --version ${MDBOOK_ADMONISH_VERSION}

# Tell git to trust "dubious" ownership
RUN git config --global --add safe.directory /repository

# Entry directory for non-vscode containers based on this image
WORKDIR /repository
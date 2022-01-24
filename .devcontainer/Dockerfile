# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.187.0/containers/rust/.devcontainer/base.Dockerfile

FROM mcr.microsoft.com/vscode/devcontainers/rust:0-1

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install npm && npm install -g newman

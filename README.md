# What is Zeus?

This is a fun project I created for learning Kubernetes.
I use it on my home server to spin up servers for different games easily.
Through the website, one can easily check the status and the address of the server.
Although I use it to spin up game servers, it could be used to toggle any kind of pods.

Zeus is supposed to be run as a pod on a Kubernetes cluster.
The pod consists of two containers: the controller in this repo and the [website](https://github.com/ollivarila/zeus-web).
Additional configuration is required to route traffic but not getting into specifics here.

## Contents

This repository contains the controller created with Axum
It is responsible for creating and destroying pods by communicating with the Kubernetes API.

This project has not been tested and is definitely not production ready.

## Running

You can run the project after cloning with _cargo run_.
You can provide the env variables altough they have defaults.

_Templates directory should be created in the project root if going with the default values_

## Configuration

The app is configurable via the following environment variables

- RUST_LOG (log level)
- TEMPLATE_PATH (path where the app looks for pod templates)
- PORT (app listens on this port)

## Links

- [live website](https://zeus.servegame.com) (Will not always be online because the server might not be running)
- [container image](https://hub.docker.com/repository/docker/ollivarila/zeus-controller/general)

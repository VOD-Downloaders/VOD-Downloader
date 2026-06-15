#!/bin/sh

docker compose -f compose-dev.yaml up flaresolverr fmhy_bridge -d
docker compose -f compose-dev.yaml down fmhy_downloader
docker compose -f compose-dev.yaml up fmhy_downloader --build -d
docker logs fmhy_downloader -f

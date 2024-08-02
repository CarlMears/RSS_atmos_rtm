# ACCESS atmospheric RTM Dockerfile
#
# Build with:
#
# docker build -t access-atmospheric-rtm:latest \
#   --build-arg=build_date=$(date --utc --date="@${SOURCE_DATE_EPOCH:-$(date +%s)}" --rfc-3339='seconds') \
#   --build-arg=source_date=$(git log -1 --pretty=%ci) \
#   --build-arg=version=$(git rev-parse --short HEAD) \
#   --build-arg=revision=$(git symbolic-ref --short HEAD) \
#   .
#
# To run it:
#
# docker run access-atmospheric-rtm:latest -m access_atmosphere.download ...
#
# Or:
#
# docker run access-atmospheric-rtm:latest -m access_atmosphere.process...
#
# Probably a bind-mount volume is needed to store the outputs. For example:
#
# docker run -v $PWD:/data access-atmospheric-rtm:latest -m access_atmosphere.process /data/era5_surface.nc /data/era5_levels.nc /data/rtm_out.nc

FROM ghcr.io/pyo3/maturin:latest AS build

COPY . .
RUN maturin build --release -i "python3.11"

FROM docker.io/library/python:3.11-slim

WORKDIR /root
COPY --from=build \
    /io/target/wheels/*.whl \
    /root/

RUN --mount=type=bind,from=ghcr.io/astral-sh/uv:latest,source=/uv,target=/bin/uv \
    uv pip install --system --no-cache ./access_atmosphere-*-cp311-cp311-manylinux*.whl
ENTRYPOINT [ "/usr/local/bin/python3" ]

ARG version
ARG revision
ARG build_date
ARG source_date
LABEL Maintainer="Richard Lindsley <lindsley@remss.com>" \
    "org.opencontainers.image.title"="ACCESS Atmospheric RTM" \
    "org.opencontainers.image.description"="Microwave atmospheric RTM for ACCESS work" \
    "org.opencontainers.image.created"="$build_date" \
    "org.opencontainers.image.authors"="Richard Lindsley <lindsley@remss.com>" \
    "org.opencontainers.image.vendor"="Remote Sensing Systems" \
    "org.opencontainers.image.url"="http://gitlab.remss.com/access/atmospheric-rtm" \
    "org.opencontainers.image.source"="http://gitlab.remss.com/access/atmospheric-rtm" \
    "org.opencontainers.image.version"="$version" \
    "org.opencontainers.image.revision"="$revision"

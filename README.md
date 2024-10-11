# ACCESS Atmospheric RTM

For the ACCESS project, download ERA5 atmosphere geophysical parameters and apply the radiative transfer model to obtain atmospheric microwave terms.

## Overview

This is a Python package, `access-atmosphere`, that currently performs two main
tasks:

- Download relevant ERA5 data
- Compute atmospheric RTM based on the ERA5 data

The RTM is implemented in Rust and compiled into a Python extension using
[maturin](https://maturin.rs/) and [pyo3](https://pyo3.rs/).

## Installing

The `access-atmosphere` Python package is available on the GitLab-hosted PyPI
server. To install it a new venv on Linux named `venv`:

```bash
python3 -m venv --upgrade-deps venv
source venv/bin/activate
pip install access-atmosphere \
  --index-url http://gitlab.remss.com/api/v4/projects/68/packages/pypi/simple \
  --trusted-host gitlab.remss.com
```

Or on Windows (using Python installed from https://www.python.org, which uses the `py.exe` launcher):

```powershell
# This may need to be run first, as mentioned here: https://docs.python.org/3/library/venv.html
#
# Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

py.exe -m venv --upgrade-deps venv
.\venv\Scripts\Activate.ps1
pip install access-atmosphere `
  --index-url http://gitlab.remss.com/api/v4/projects/68/packages/pypi/simple `
  --trusted-host gitlab.remss.com
```

Wheels are built for Linux and Windows for x86-64 for Python 3.10 and newer. The
Linux wheels are built in two flavors: `manylinux_2_28`, which is for most
common Linux distributions using the GNU C library 2.28 or greater, and
`musllinux_1_2`, which is for Linux distributions using the musl C library 1.2
or greater. For example, use the `manylinux` wheel for Debian but the
`musllinux` wheel for Alpine.

### Alpine instructions

The `musllinux_1_2` wheel works in Alpine Linux. However, the Python
[`netCDF4`](https://github.com/Unidata/netcdf4-python/) and `cftime` packages
are runtime dependencies and `musllinux` wheels are not yet available on PyPI.
For convenience, until they're available, GitLab CI jobs automatically build
`musllinux_1_2` wheels for these two dependencies as well. To install the
project within an Alpine container, such as
[`docker.io/library/python:3.11-alpine`](https://hub.docker.com/_/python):

```sh
# Copy in or download the relevant wheels and then install them
$ pip install \
  access_atmosphere-0.3.1-cp311-cp311-musllinux_1_2_x86_64.whl \
  netCDF4-1.6.4-cp311-cp311-musllinux_1_2_x86_64.whl \
  cftime-1.6.2-cp311-cp311-musllinux_1_2_x86_64.whl
$ python -m access_atmosphere.process ...
```

## Building (for development)

To build the package locally, both [Python](https://www.python.org/) and
[Rust](https://www.python.org/) are required. Rust can be installed using using
[`rustup`](https://rustup.rs/). [Maturin](https://maturin.rs/) is used to build
and package the Python wheel.

On Linux:

```bash
# Install maturin in a virtual environment
python3 -m venv --upgrade-deps .venv
source .venv/bin/activate
pip install maturin

# Build the wheel
maturin build --release
# The wheel is in target/wheels/ and can be installed using "pip install"

# For local development, the wheel can be built and installed in the current venv
maturin develop
```

On Windows (noting that `Set-ExecutionPolicy -ExecutionPolicy RemoteSigned
-Scope CurrentUser` is run first, as mentioned in the [venv
documentation](https://docs.python.org/3/library/venv.html)):

```powershell
py.exe -m venv --upgrade-deps venv
.\venv\Scripts\Activate.ps1
pip install maturin

# Build the wheel. The "abi3" feature is used to avoid looking for the Python
# development libraries since they're bundled in this way.
maturin build --release --features abi3

# For local development, building and installing the wheel in the venv
maturin develop --features abi3
```

Alternately, the GitLab CI automatically builds wheels. Python wheels end with
the following: `-{python tag}-{abitag}-{platform tag}.whl`. The built wheels
have the following possible values for the tags (though not with every
combination):

| Python tag | Description |
| --- | --- |
| `cp310` | CPython 3.10 |
| `cp311` | CPython 3.11 |
| `cp312` | CPython 3.12 |
| `cp313` | CPython 3.13 |

| Abi tag | Description |
| --- | --- |
| (same as Python tag) | Exactly the Python minor version |
| `abi3` | The Python minor version or higher |

| Platform tag | Description |
| --- | --- |
| `win_amd64` | x86_64 Windows |
| `manylinux_2_28_x86_64` | x86_64 Linux with glibc 2.28 and later |
| `musllinux_1_2_x86_64` | x86_64 Linux with musl 1.2 and later |

Following the [Scientific Python ecosystem policy of supported Python
versions](https://scientific-python.org/specs/spec-0000/), Python 3.10 is the
minimum version used.

The [`manylinux_2_28`](https://github.com/pypa/manylinux) policy ensures that it
is compatible with [most Linux
distributions](https://github.com/mayeut/pep600_compliance) starting in about
2019, for example Debian buster and Red Hat Enterprise Linux 8.

The wheels can be downloaded as CI job artifacts, and for every release, are
generated and saved in the [local package
registry](http://gitlab.remss.com/access/atmospheric-rtm/-/packages).

Note that with a wheel built, it's ready to install without needed a local Rust
compiler. To install the wheel in a new venv and download the dependencies from
PyPI:

```bash
python3 -m venv --upgrade-deps .venv
.venv/bin/pip install access-atmosphere-*.whl
```

As another option, a [`Dockerfile`](Dockerfile) is provided. Build it with
`docker` or `podman`:

```bash
podman build -t access_atmosphere -f Dockerfile
```

## Running

The API documentation is built using [pdoc](https://pdoc.dev/docs/pdoc.html) and
hosted at: <http://access.pages.remss.com/atmospheric-rtm/>.

### Downloading ERA5 data

To download data from the [Climate Data
Store](https://cds.climate.copernicus.eu/cdsapp), an account needs to be
registered. Note the UID and API key for later.

To download the ERA5 datasets of interest for some time range, the script can be
run with two environment variables set for CDS authentication:

```bash
env CDS_UID=xxx CDS_API_KEY=xxx python3 -m access_atmosphere.download 2020-01-01 2020-01-31 --out-dir era5
```

For each day, two files are created: one for the surface data and one for the
atmospheric profiles. The netCDF files are written to the directory specified by
`--out-dir`, or the current working directory if it's not specified. Any
existing files are not re-downloaded from CDS.

### Applying the RTM

A Python interface to the RTM is provided in the `access_atmosphere.rtm` module.

As an example usage, the `access_atmosphere.process` module reads in a pair of
ERA5 daily files and runs the RTM for every point (latitude/longitude/time) and
writes the results in a new netCDF file. It uses reference values for the
incidence angles and microwave frequencies. The outputs are on the same
0.25-degree grid that ERA5 uses.

As an example using the 2020-01-01 data downloaded above:

```bash
python -m access_atmosphere.process \
    era5_surface_2020-01-01.nc \
    era5_levels_2020-01-01.nc \
    access_era5_2020-01-01.nc
```

The Rust code uses [Rayon](https://github.com/rayon-rs/rayon) to process each
profile in parallel using a pool of worker threads. By default, this will be as
many threads as there are logical CPUs detected on the machine. To adjust this,
use the `--workers` command-line option or use the `RAYON_NUM_THREADS`
environment variable. For instance, to use exactly 4 threads:

```bash
# Via command-line option
python3 -m access_atmosphere.process --workers 4 ...
# Via environment variable
env RAYON_NUM_THREADS=4 python3 -m access_atmosphere.process ...
```
import nox

# By default just run these basic lint jobs
nox.options.sessions = ["ruff", "mypy", "ruff_format"]


@nox.session
def ruff_format(session: nox.Session) -> None:
    """Check if reformatting is needed"""
    session.install("ruff")
    session.run("ruff", "format", "--diff", "python/")


@nox.session
def ruff(session: nox.Session) -> None:
    """Run ruff"""
    session.install("ruff")
    session.run("ruff", "check", "python/")


@nox.session
def ruff_junit(session: nox.Session) -> None:
    """Run ruff and generate a JUnit report file"""
    session.install("ruff")
    session.run(
        "ruff",
        "check",
        "--quiet",
        "--output-format=junit",
        "--output-file=ruff.junit.xml",
        "python/",
    )


@nox.session
def mypy(session: nox.Session) -> None:
    """Run mypy"""
    session.install("mypy", "numpy", "netCDF4", "cdsapi")
    session.run(
        "mypy",
        "--pretty",
        "--show-error-context",
        "python/",
    )


@nox.session
def mypy_full(session: nox.Session) -> None:
    """Run mypy and generate report files"""
    session.install("mypy", "numpy", "netCDF4", "cdsapi", "lxml")
    session.run(
        "mypy",
        "--pretty",
        "--show-error-context",
        "--junit-xml=mypy.junit.xml",
        "--cobertura-xml-report=.",
        "--lineprecision-report=.",
        "python/",
    )


@nox.session
def stubtest(session: nox.Session) -> None:
    """Run stubtest

    The package needs to be built and installed before stubtest can import it.
    If no arguments are specified, then the package is built here. This requires
    Rust.

    ```
    nox -s stubtest
    ```

    Otherwise, a pre-built wheel can used, which avoids the need for Rust.

    ```
    nox -s stubtest -- *-abi3-*.whl
    ```
    """
    session.install("mypy")
    if session.posargs:
        session.install(*session.posargs)
    else:
        session.log("Rebuilding the access_atmosphere package locally")
        session.install(".")
    session.run(
        "stubtest",
        "--mypy-config-file=pyproject.toml",
        "access_atmosphere.access_atmosphere",
    )


@nox.session
def pdoc(session: nox.Session) -> None:
    """Run pdoc and save output

    The package needs to be built and installed before pdoc can import it. If no
    arguments are specified, then the package is built here. This requires Rust.

    ```
    nox -s pdoc
    ```

    Otherwise, a pre-built wheel can used, which avoids the need for Rust.

    ```
    nox -s pdoc -- *-abi3-*.whl
    ```
    """
    if session.posargs:
        session.install(*session.posargs)
    else:
        session.log("Rebuilding the access_atmosphere package locally")
        session.install(".")
    session.install("pdoc")
    session.run("pdoc", "access_atmosphere", "--output-directory", "public")

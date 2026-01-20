from setuptools import setup, find_packages

setup(
    name="riveroftime",
    version="0.1.0",
    package_dir={"": "src"},
    packages=find_packages(where="src"),
    install_requires=[
        "colorama",
    ],
    entry_points={
        "console_scripts": [
            "riveroftime=riveroftime.cli:main",
        ],
    },
)

from setuptools import find_packages, setup

setup(
    name="riveroftime",
    version="0.1.0",
    package_dir={"": "src"},
    packages=find_packages(where="src"),
    install_requires=[
        "rich",
    ],
    entry_points={
        "console_scripts": [
            "riveroftime=riveroftime.cli:main",
        ],
    },
)

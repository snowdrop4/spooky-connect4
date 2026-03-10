from setuptools import setup
from setuptools_rust import Binding, RustExtension  # type: ignore

setup(
    rust_extensions=[
        RustExtension(
            "spooky_connect4",
            binding=Binding.PyO3,
            debug=False,
            features=["python"],
            rustc_flags=["-Copt-level=3", "-Clto=fat"],
        )
    ],
    data_files=[("", ["spooky_connect4.pyi"])],
    zip_safe=False,
)

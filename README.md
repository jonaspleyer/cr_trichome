# cr_trichome

Package to model trichome growth on the leaf of arabidopsis thaliana.

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/jonaspleyer/cr_trichome/CI.yml?style=flat-square&label=Build)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/jonaspleyer/cr_trichome/sphinx_doc.yml?style=flat-square&label=docs)

# Documentation
Visit the documentation at [jonaspleyer.github.io/cr_trichome/](https://jonaspleyer.github.io/cr_trichome/).
We use `cellular_raza` as the underlying simulation framework.
Its documentation can be seen at [cellular_raza.com](https://cellular_raza.com).

# Usage
To build the package run
```bash
maturin develop
```

Afterwards, the package can be used like so:
```python
import cr_trichome as crt
```

# Testing
To run tests for the rust part of the package use

```bash
cargo test
```

To run python tests, we use pytest

```bash
python -m pytest
```

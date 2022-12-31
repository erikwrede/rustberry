# Python Bindings for Apollo-RS
This is a basic prototype. Use with caution and for testing only!

## build
```shell
pip install maturin 
maturin build --release
```

## test
Adjust the wheel path in `test_parser/pyproject.toml`

```shell
cd test_parser
poetry install
source .venv/bin/activate
python test_rust.py
python test_graphql_core.py
```
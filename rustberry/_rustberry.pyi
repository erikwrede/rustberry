class QueryCompiler:
    def __init__(self) -> None: ...
    def set_schema(self, schema: str) -> None: ...
    def add_executable(self, contents: str) -> None: ...
    def validate(self) -> bool: ...
    def validate_file(self, file_id: int) -> bool: ...
    def add_validate(self, contents: str) -> bool: ...
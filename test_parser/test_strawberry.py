from timeit import timeit

from pyinstrument import Profiler

import strawberry
from strawberry.types.info import Info


@strawberry.type
class Query:
    @strawberry.field
    def foo(self, info: Info) -> None:
        return None

schema = strawberry.Schema(Query)
schema.execute_sync("query { foo }", root_value=Query())

schema = strawberry.Schema(Query)

profiler = Profiler(interval=0.0001)
profiler.start()
N = 10000
duration = timeit(
    lambda: schema.execute_sync("query { foo }", root_value=Query()),
    number=N,
)
print(duration)
profiler.stop()
with open("output.html", "w") as f:
    f.write(profiler.output_html())
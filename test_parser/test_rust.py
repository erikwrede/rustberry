import timeit

from rustberry import QueryCompiler
from util import get_sdl_str, get_query_str


from pyinstrument import Profiler


schema = get_sdl_str()
operation = get_query_str()

compiler = QueryCompiler()
compiler.set_schema(schema)

# Validate the schema so it doesn't interfere with the timing
print("Schema validation result", compiler.validate())


def validate_timing():
    file_id = compiler.add_executable(operation)
    print(file_id)
    validation_errors = compiler.validate_file(file_id) # fixme this still validates the entire schema as wekk
    print("Validation errors:", not validation_errors)



profiler = Profiler(interval=0.0001)
profiler.start()
num = 1
time = timeit.timeit(validate_timing, number=num)
print(f"Parsing & Validation on apollo-rs took an average of {time * 1000 / num} milliseconds ({num} iterations)")

profiler.stop()
with open("output.html", "w") as f:
    f.write(profiler.output_html())
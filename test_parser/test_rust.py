import timeit

from rustberry import QueryCompiler
from util import get_sdl_str, get_query_str


from pyinstrument import Profiler

from graphql.language.ast import FieldNode

schema = get_sdl_str()
operation = get_query_str()

compiler = QueryCompiler()
compiler.set_schema(schema)

# Validate the schema so it doesn't interfere with the timing
print("Schema validation result", compiler.validate())
ast = None
def validate_timing():
    global ast
    file_id = compiler.add_executable(operation)
    #print(file_id)
    validation_errors = compiler.validate_file(file_id)
    print(validation_errors)
    ast = compiler.gql_core_ast_mirror(file_id)
    #print("Validation errors:", not validation_errors)



profiler = Profiler(interval=0.0001)
profiler.start()
num = 100
time = timeit.timeit(validate_timing, number=num)
print(f"Parsing & Validation on apollo-rs took an average of {time * 1000 / num} milliseconds ({num} iterations)")

field_node = ast.definitions[0].selection_set.selections[0]

print("Instance Check", isinstance(field_node, FieldNode))
print(field_node.__class__)
print(ast.definitions[0].selection_set.selections[0])

profiler.stop()

#with open("output.html", "w") as f:
#    f.write(profiler.output_html())

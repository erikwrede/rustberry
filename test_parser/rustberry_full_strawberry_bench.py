import timeit

import strawberry
from strawberry.schema import BaseSchema
from strawberry.schema.schema import DEFAULT_ALLOWED_OPERATION_TYPES
from rustberry import QueryCompiler


@strawberry.type
class Fruit:
    name: str


@strawberry.type
class Query:
    @strawberry.field
    def hello(self, info) -> str:
        return "Hello, world!"

    @strawberry.field
    def greeting(self, info) -> str:
        return f"Hello, ABC!"

    @strawberry.field
    def fruits(self, info) -> list[Fruit]:
        return [Fruit(name="Apple"), Fruit(name="Banana"), Fruit(name="Orange")]


schema = strawberry.Schema(query=Query)

schema_str = schema.as_str()
compiler = QueryCompiler()
compiler.set_schema(schema_str)

compiler.validate()

from strawberry.schema.execute import *


def execute_sync_patch(
        schema: GraphQLSchema,
        *,
        allowed_operation_types: Iterable[OperationType],
        extensions: Sequence[Union[Type[Extension], Extension]],
        execution_context: ExecutionContext,
        execution_context_class: Optional[Type[GraphQLExecutionContext]] = None,
        process_errors: Callable[[List[GraphQLError], Optional[ExecutionContext]], None],
) -> ExecutionResult:
    extensions_runner = ExtensionsRunner(
        execution_context=execution_context,
        extensions=list(extensions),
    )

    with extensions_runner.request():
        # Note: In graphql-core the schema would be validated here but in
        # Strawberry we are validating it at initialisation time instead
        if not execution_context.query:
            raise MissingQueryError()

        with extensions_runner.parsing():
            try:
                if not execution_context.graphql_document:
                    file_id = compiler.add_executable(execution_context.query)
                    execution_context.graphql_document = compiler.gql_core_ast_mirror(file_id)
            except GraphQLError as error:
                execution_context.errors = [error]
                process_errors([error], execution_context)
                return ExecutionResult(
                    data=None,
                    errors=[error],
                    extensions=extensions_runner.get_extensions_results_sync(),
                )

            except Exception as error:  # pragma: no cover
                error = GraphQLError(str(error), original_error=error)

                execution_context.errors = [error]
                process_errors([error], execution_context)
                return ExecutionResult(
                    data=None,
                    errors=[error],
                    extensions=extensions_runner.get_extensions_results_sync(),
                )

        if execution_context.operation_type not in allowed_operation_types:
            raise InvalidOperationTypeError(execution_context.operation_type)

        with extensions_runner.validation():
            compiler.validate_file(file_id)
            if execution_context.errors:
                process_errors(execution_context.errors, execution_context)
                return ExecutionResult(data=None, errors=execution_context.errors)

        with extensions_runner.executing():
            if not execution_context.result:
                result = original_execute(
                    schema,
                    execution_context.graphql_document,
                    root_value=execution_context.root_value,
                    middleware=extensions_runner.as_middleware_manager(),
                    variable_values=execution_context.variables,
                    operation_name=execution_context.operation_name,
                    context_value=execution_context.context,
                    execution_context_class=execution_context_class,
                )

                if isawaitable(result):
                    result = cast(Awaitable[GraphQLExecutionResult], result)
                    ensure_future(result).cancel()
                    raise RuntimeError(
                        "GraphQL execution failed to complete synchronously."
                    )

                result = cast(GraphQLExecutionResult, result)
                execution_context.result = result
                # Also set errors on the execution_context so that it's easier
                # to access in extensions
                if result.errors:
                    execution_context.errors = result.errors

                    # Run the `Schema.process_errors` function here before
                    # extensions have a chance to modify them (see the MaskErrors
                    # extension). That way we can log the original errors but
                    # only return a sanitised version to the client.
                    process_errors(result.errors, execution_context)

    return ExecutionResult(
        data=execution_context.result.data,
        errors=execution_context.result.errors,
        extensions=extensions_runner.get_extensions_results_sync(),
    )

# Monkey Patch execution


def validate_timing():
    execution_context = ExecutionContext(
        query="""
                { hello, greeting, fruits { name } }
                """,
        schema=schema,
        context=None,
        root_value=None,
        variables=None,
        provided_operation_name=None,
    )

    result = execute_sync_patch(
        schema._schema,
        extensions=[],
        execution_context_class=schema.execution_context_class,
        execution_context=execution_context,
        allowed_operation_types=DEFAULT_ALLOWED_OPERATION_TYPES,
        process_errors=BaseSchema.process_errors,
    )



num = 100
time = timeit.timeit(validate_timing, number=num)
print(f"Execution on graphql-core took an average of {time * 1000 / num} milliseconds ({num} iterations)")
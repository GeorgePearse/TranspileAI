"""
Example implementation: Simple math functions

This module demonstrates how to write Python functions that can be tested
via the transpilation testing infrastructure.
"""

import sys
sys.path.append('../../python')
from server import transpile_test


@transpile_test(
    name="add",
    description="Add two numbers",
    is_stateful=False,
    parameter_types=["int", "int"],
    return_type="int",
)
def add(context, a, b):
    """Add two numbers."""
    return a + b


@transpile_test(
    name="multiply",
    description="Multiply two numbers",
    is_stateful=False,
    parameter_types=["int", "int"],
    return_type="int",
)
def multiply(context, a, b):
    """Multiply two numbers."""
    return a * b


@transpile_test(
    name="fibonacci",
    description="Calculate the nth Fibonacci number",
    is_stateful=False,
    parameter_types=["int"],
    return_type="int",
)
def fibonacci(context, n):
    """Calculate the nth Fibonacci number."""
    if n <= 1:
        return n

    a, b = 0, 1
    for _ in range(2, n + 1):
        a, b = b, a + b
    return b


@transpile_test(
    name="counter_increment",
    description="Increment a counter (stateful)",
    is_stateful=True,
    parameter_types=[],
    return_type="int",
)
def counter_increment(context):
    """Increment a counter stored in context state."""
    current = context.state.get("counter", 0)
    new_value = current + 1
    context.update_state("counter", new_value)
    return new_value


@transpile_test(
    name="counter_get",
    description="Get current counter value (stateful)",
    is_stateful=True,
    parameter_types=[],
    return_type="int",
)
def counter_get(context):
    """Get the current counter value from context state."""
    return context.state.get("counter", 0)


@transpile_test(
    name="factorial",
    description="Calculate factorial of a number",
    is_stateful=False,
    parameter_types=["int"],
    return_type="int",
)
def factorial(context, n):
    """Calculate factorial recursively."""
    if n <= 1:
        return 1
    return n * factorial(context, n - 1)


@transpile_test(
    name="is_prime",
    description="Check if a number is prime",
    is_stateful=False,
    parameter_types=["int"],
    return_type="bool",
)
def is_prime(context, n):
    """Check if a number is prime."""
    if n < 2:
        return False
    if n == 2:
        return True
    if n % 2 == 0:
        return False

    for i in range(3, int(n ** 0.5) + 1, 2):
        if n % i == 0:
            return False
    return True

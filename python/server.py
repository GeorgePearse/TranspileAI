#!/usr/bin/env python3
"""
Python gRPC server for transpilation testing.

This server allows executing Python functions over gRPC with support for:
- Stateless function calls
- Stateful execution contexts
- Dynamic function loading
"""

import argparse
import importlib.util
import json
import logging
import sys
import time
import uuid
from concurrent import futures
from pathlib import Path
from typing import Any, Callable, Dict, Optional

import grpc
from grpc_reflection.v1alpha import reflection

# Generated proto imports (will be generated from .proto file)
# For now, we'll add placeholder imports
try:
    import transpile_test_pb2
    import transpile_test_pb2_grpc
except ImportError:
    print("Error: Proto files not generated. Run: python -m grpc_tools.protoc -I../proto --python_out=. --grpc_python_out=. ../proto/transpile_test.proto")
    sys.exit(1)


class ExecutionContext:
    """Manages state for stateful function execution."""

    def __init__(self, context_id: str, initial_state: Optional[str] = None):
        self.context_id = context_id
        self.state: Dict[str, Any] = {}
        if initial_state:
            try:
                self.state = json.loads(initial_state)
            except json.JSONDecodeError:
                logging.warning(f"Invalid initial state JSON for context {context_id}")

    def get_state(self) -> str:
        return json.dumps(self.state, default=str)

    def update_state(self, key: str, value: Any):
        self.state[key] = value


class TranspileTestServiceImpl(transpile_test_pb2_grpc.TranspileTestServiceServicer):
    """Implementation of the TranspileTestService."""

    def __init__(self):
        self.contexts: Dict[str, ExecutionContext] = {}
        self.methods: Dict[str, Callable] = {}
        self.method_metadata: Dict[str, dict] = {}
        logging.info("Python gRPC server initialized")

    def register_function(
        self,
        name: str,
        func: Callable,
        description: str = "",
        is_stateful: bool = False,
        parameter_types: Optional[list] = None,
        return_type: str = "Any",
    ):
        """Register a function that can be invoked via gRPC."""
        self.methods[name] = func
        self.method_metadata[name] = {
            "description": description,
            "is_stateful": is_stateful,
            "parameter_types": parameter_types or [],
            "return_type": return_type,
        }
        logging.info(f"Registered function: {name}")

    def load_module(self, module_path: str):
        """Dynamically load a Python module and register its functions."""
        path = Path(module_path)
        if not path.exists():
            logging.error(f"Module not found: {module_path}")
            return

        spec = importlib.util.spec_from_file_location("dynamic_module", path)
        if spec and spec.loader:
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)

            # Auto-register functions with __transpile_test__ marker
            for name in dir(module):
                obj = getattr(module, name)
                if callable(obj) and hasattr(obj, "__transpile_test__"):
                    metadata = obj.__transpile_test__
                    self.register_function(
                        name=metadata.get("name", name),
                        func=obj,
                        description=metadata.get("description", ""),
                        is_stateful=metadata.get("is_stateful", False),
                        parameter_types=metadata.get("parameter_types", []),
                        return_type=metadata.get("return_type", "Any"),
                    )
            logging.info(f"Loaded module: {module_path}")

    def CreateContext(self, request, context):
        """Create a new execution context."""
        context_id = str(uuid.uuid4())
        exec_context = ExecutionContext(context_id, request.initial_state)
        self.contexts[context_id] = exec_context

        logging.info(f"Created context: {context_id}")
        return transpile_test_pb2.CreateContextResponse(
            context_id=context_id, success=True, error=""
        )

    def InvokeMethod(self, request, context):
        """Invoke a registered method."""
        start_time = time.perf_counter()

        try:
            # Get the function
            if request.method_name not in self.methods:
                return transpile_test_pb2.InvokeMethodResponse(
                    success=False,
                    error=f"Method not found: {request.method_name}",
                )

            func = self.methods[request.method_name]

            # Parse arguments
            try:
                args = json.loads(request.arguments) if request.arguments else {}
            except json.JSONDecodeError as e:
                return transpile_test_pb2.InvokeMethodResponse(
                    success=False, error=f"Invalid JSON arguments: {e}"
                )

            # Get context if needed
            exec_context = None
            if request.context_id:
                if request.context_id not in self.contexts:
                    return transpile_test_pb2.InvokeMethodResponse(
                        success=False, error=f"Context not found: {request.context_id}"
                    )
                exec_context = self.contexts[request.context_id]

            # Execute function
            if exec_context and self.method_metadata[request.method_name]["is_stateful"]:
                # Pass context to stateful functions
                result = func(exec_context, **args)
            else:
                # Call stateless functions normally
                result = func(**args)

            # Calculate execution time
            execution_time_us = int((time.perf_counter() - start_time) * 1_000_000)

            # Serialize result
            result_json = json.dumps(result, default=str)

            metadata = transpile_test_pb2.ExecutionMetadata(
                execution_time_us=execution_time_us,
                memory_bytes=0,  # TODO: Implement memory tracking
                runtime="python",
            )

            logging.debug(f"Executed {request.method_name} in {execution_time_us}us")
            return transpile_test_pb2.InvokeMethodResponse(
                success=True, result=result_json, error="", metadata=metadata
            )

        except Exception as e:
            logging.error(f"Error executing {request.method_name}: {e}", exc_info=True)
            return transpile_test_pb2.InvokeMethodResponse(
                success=False, error=str(e)
            )

    def InspectState(self, request, context):
        """Inspect the state of a context."""
        if request.context_id not in self.contexts:
            return transpile_test_pb2.InspectStateResponse(
                success=False, error=f"Context not found: {request.context_id}"
            )

        exec_context = self.contexts[request.context_id]
        return transpile_test_pb2.InspectStateResponse(
            success=True, state=exec_context.get_state(), error=""
        )

    def DestroyContext(self, request, context):
        """Destroy an execution context."""
        if request.context_id in self.contexts:
            del self.contexts[request.context_id]
            logging.info(f"Destroyed context: {request.context_id}")
            return transpile_test_pb2.DestroyContextResponse(success=True, error="")
        else:
            return transpile_test_pb2.DestroyContextResponse(
                success=False, error=f"Context not found: {request.context_id}"
            )

    def ListMethods(self, request, context):
        """List available methods."""
        methods = []
        for name, metadata in self.method_metadata.items():
            if request.prefix and not name.startswith(request.prefix):
                continue

            method_info = transpile_test_pb2.MethodInfo(
                name=name,
                description=metadata["description"],
                is_stateful=metadata["is_stateful"],
                parameter_types=metadata["parameter_types"],
                return_type=metadata["return_type"],
            )
            methods.append(method_info)

        return transpile_test_pb2.ListMethodsResponse(methods=methods)


def transpile_test(**metadata):
    """Decorator to mark functions for transpilation testing."""

    def decorator(func):
        func.__transpile_test__ = metadata
        return func

    return decorator


def serve(port: int, module_path: Optional[str] = None):
    """Start the gRPC server."""
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    service = TranspileTestServiceImpl()

    if module_path:
        service.load_module(module_path)

    transpile_test_pb2_grpc.add_TranspileTestServiceServicer_to_server(service, server)

    # Enable reflection for debugging with grpcurl
    SERVICE_NAMES = (
        transpile_test_pb2.DESCRIPTOR.services_by_name["TranspileTestService"].full_name,
        reflection.SERVICE_NAME,
    )
    reflection.enable_server_reflection(SERVICE_NAMES, server)

    server.add_insecure_port(f"[::]:{port}")
    server.start()

    logging.info(f"Python gRPC server started on port {port}")
    print(f"Python gRPC server listening on port {port}")

    try:
        server.wait_for_termination()
    except KeyboardInterrupt:
        logging.info("Shutting down server...")
        server.stop(0)


def main():
    parser = argparse.ArgumentParser(description="Python gRPC Test Server")
    parser.add_argument("--port", type=int, default=50051, help="Server port")
    parser.add_argument("--module", type=str, help="Python module to load")
    parser.add_argument("--verbose", action="store_true", help="Enable debug logging")

    args = parser.parse_args()

    logging.basicConfig(
        level=logging.DEBUG if args.verbose else logging.INFO,
        format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    )

    serve(args.port, args.module)


if __name__ == "__main__":
    main()

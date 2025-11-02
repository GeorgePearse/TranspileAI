#!/bin/bash
# Generate Python gRPC stubs from proto files

cd "$(dirname "$0")"

python3 -m grpc_tools.protoc \
    -I../proto \
    --python_out=. \
    --grpc_python_out=. \
    ../proto/transpile_test.proto

echo "Proto files generated successfully!"

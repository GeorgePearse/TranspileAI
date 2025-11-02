# TranspileAI

An AI-powered code transpilation system for rewriting packages across different programming languages.

## Overview

TranspileAI aims to leverage modern AI techniques to translate code between programming languages while maintaining functionality, idioms, and best practices. This project builds upon decades of program synthesis and code translation research.

## Historical Context: Program Synthesis Before LLMs

Before the era of Large Language Models, program synthesis and code translation relied on symbolic methods, constraint solvers, and formal verification techniques. These approaches offered strong correctness guarantees but were limited in scope.

### Key Pre-LLM Approaches

#### Sketch-Based Synthesis
- **[Introduction to Program Synthesis](https://people.csail.mit.edu/asolar/SynthesisCourse/Lecture1.htm)** - Armando Solar-Lezama's foundational course on program synthesis
- **[The Sketching Approach to Program Synthesis](https://people.csail.mit.edu/asolar/papers/Solar-Lezama09.pdf)** - Pioneering work by Armando Solar-Lezama
- **[Program Synthesis by Sketching (PhD Thesis)](https://people.csail.mit.edu/asolar/papers/thesis.pdf)** - Solar-Lezama's doctoral thesis from UC Berkeley (2008)

**Core Concept**: A sketch is a syntactic template with "holes" that a synthesizer fills in. Programmers provide a partial program structure, and the system searches for valid completions that satisfy given constraints.

#### Solver-Aided Programming
- **[Rosette](https://docs.racket-lang.org/rosette-guide/)** - A solver-aided programming language that extends Racket
- **[Growing Solver-Aided Languages with Rosette](https://dl.acm.org/doi/10.1145/2509578.2509586)** - Emina Torlak and Rastislav Bodik
- **[Building Your First Program Synthesizer](https://blog.sigplan.org/2019/11/26/building-your-first-program-synthesizer/)** - SIGPLAN tutorial

**Core Concept**: Uses SMT (Satisfiability Modulo Theories) solvers like Z3 to reason about program properties, verify correctness, and synthesize code that meets specifications.

#### Traditional Rule-Based Transpilation
- **[Source-to-Source Compiler](https://en.wikipedia.org/wiki/Source-to-source_compiler)** - Wikipedia overview
- Relied on handcrafted transformation rules and pattern matching
- Parse input → Create abstract representation → Apply transformations → Generate target code
- Limited by the need for explicit rules for every construct

### Key Differences from Modern Approaches

| Aspect | Pre-LLM (Symbolic) | LLM-Based |
|--------|-------------------|-----------|
| **Guarantees** | Formal correctness proofs | Probabilistic, best-effort |
| **Scope** | Limited to small programs | Handles larger codebases |
| **Technique** | Constraint solving, search | Pattern recognition, statistical |
| **Speed** | Can be slow for complex programs | Generally fast |
| **Idiomacy** | Often unidiomatic output | Can produce idiomatic code |

## Modern Approach: LLM-Powered Transpilation

TranspileAI combines the best of both worlds:
- Leverages LLMs for understanding code semantics and generating idiomatic translations
- Incorporates verification and testing to ensure correctness
- Uses formal methods where applicable for critical components

## Architecture

```
TranspileAI/
├── proto/                           # gRPC service definitions
│   └── transpile_test.proto        # Cross-language test protocol
├── python/                          # Python test server
│   ├── server.py                   # gRPC server implementation
│   ├── requirements.txt            # Python dependencies
│   └── generate_proto.sh           # Proto code generation
├── rust/                            # Rust test server
│   ├── src/
│   │   ├── server.rs               # gRPC server implementation
│   │   └── examples.rs             # Example function registry
│   ├── Cargo.toml                  # Rust dependencies
│   └── build.rs                    # Proto code generation
├── test-runner/                     # Test orchestration
│   ├── src/main.rs                 # Test runner CLI
│   ├── test-defs/                  # YAML test definitions
│   │   └── simple_math.yaml        # Example tests
│   └── Cargo.toml                  # Dependencies
└── examples/                        # Example implementations
    └── simple_math/
        ├── impl.py                 # Python implementation
        └── impl.rs                 # Rust implementation
```

## Cross-Language Testing Infrastructure

TranspileAI uses a **gRPC-based testing infrastructure** to validate that transpiled code produces identical behavior across different language implementations. This approach ensures correctness by running the same tests against both the original and transpiled code.

### Why gRPC?

After consulting multiple AI models (GPT-5-Pro, Gemini-2.5-Pro, Grok-4), we chose gRPC with Protocol Buffers for:

1. **Strong Type Safety**: Protobuf's explicit types (int32, int64, float, etc.) catch transpilation bugs like integer overflow that REST/JSON would miss
2. **Single Source of Truth**: `.proto` files define the contract that all language implementations must follow
3. **State Management**: Built-in support for stateful testing via context IDs
4. **Extensibility**: Adding new languages is straightforward - generate stubs from existing `.proto` files
5. **Performance**: Binary protocol with minimal overhead

### Testing Architecture

```
┌─────────────────┐
│  Test Runner    │  ← Reads YAML test definitions
└────────┬────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌────────┐ ┌────────┐
│ Python │ │  Rust  │  ← Both implement same gRPC service
│ Server │ │ Server │
└────────┘ └────────┘
    │         │
    └────┬────┘
         ▼
   Compare Results
```

### Quick Start

#### 1. Install Dependencies

**Python:**
```bash
cd python
pip3 install -r requirements.txt
./generate_proto.sh
```

**Rust:**
```bash
cd rust
cargo build
```

**Test Runner:**
```bash
cd test-runner
cargo build --release
```

#### 2. Start Test Servers

**Terminal 1 - Python Server:**
```bash
cd python
python3 server.py --port 50051 --module ../examples/simple_math/impl.py
```

**Terminal 2 - Rust Server:**
```bash
cd rust
cargo run --bin test-server -- --port 50052
```

#### 3. Run Tests

**Terminal 3 - Test Runner:**
```bash
cd test-runner
cargo run --release -- --suite test-defs/simple_math.yaml
```

Example output:
```
================================================================================
Test Suite: Simple Math Functions
================================================================================

  ✓ add_positive_numbers
    ⏱  Python: 152μs | Rust: 89μs
    Result: 8

  ✓ fibonacci_20
    ⏱  Python: 2847μs | Rust: 1203μs
    Result: 6765

  ✓ is_prime_97
    ⏱  Python: 1821μs | Rust: 743μs
    Result: true

================================================================================
Summary: 25/25 passed
================================================================================
```

### Writing Tests

Tests are defined in YAML format:

```yaml
name: My Test Suite
description: Tests for my transpiled code

servers:
  python:
    host: localhost
    port: 50051
  rust:
    host: localhost
    port: 50052

tests:
  - name: test_add
    description: Add two numbers
    method: add
    arguments:
      a: 5
      b: 3
    expected: 8

  - name: test_stateful_counter
    description: Test stateful operations
    method: counter_increment
    stateful: true
    initial_state: '{"counter": 0}'
    arguments: {}
    expected: 1
```

### Implementing Functions

**Python** (`@transpile_test` decorator):
```python
from server import transpile_test

@transpile_test(
    name="add",
    description="Add two numbers",
    is_stateful=False,
    parameter_types=["int", "int"],
    return_type="int",
)
def add(context, a, b):
    return a + b
```

**Rust** (register in examples module):
```rust
server.register_function(
    "add",
    |_ctx, args| {
        let a = args["a"].as_i64().ok_or("Missing 'a'")?;
        let b = args["b"].as_i64().ok_or("Missing 'b'")?;
        Ok(json!(a + b))
    },
    "Add two numbers",
    false,  // is_stateful
    vec!["int".to_string(), "int".to_string()],
    "int",
);
```

### Key Features

- **Stateless Functions**: Pure functions with no side effects
- **Stateful Operations**: Functions that maintain state across calls using context IDs
- **Test Isolation**: Each test gets its own execution context
- **Performance Metrics**: Execution time tracking for both implementations
- **Language-Agnostic**: Same tests run against all language implementations
- **Property-Based Testing**: Integration with Hypothesis (Python) and proptest (Rust) planned

### Debugging

List available methods:
```bash
grpcurl -plaintext localhost:50051 transpile_test.TranspileTestService/ListMethods
```

Manually invoke a function:
```bash
grpcurl -plaintext -d '{"method_name": "add", "arguments": "{\"a\": 5, \"b\": 3}"}' \
  localhost:50051 transpile_test.TranspileTestService/InvokeMethod
```

## Getting Started

```bash
# Build the transpiler (planned)
cargo build

# Run cross-language tests
cd test-runner
cargo run --release -- --suite test-defs/simple_math.yaml

# Start interactive testing
cd python && python3 server.py --module ../examples/simple_math/impl.py
```

## Supported Languages (Planned)

- Python ↔ Rust
- JavaScript/TypeScript ↔ Rust
- Java ↔ Rust
- Go ↔ Rust
- And more...

## Dependency Tree Handling

One of the most challenging aspects of code transpilation is handling the **entire dependency tree**. When transpiling a package, you can't just translate the top-level code - you need to handle all the dependencies it imports, and their dependencies, recursively down the tree.

### The Dependency Challenge

Consider transpiling a Python package to Rust:

```
your_package/
├── main.py (imports numpy, requests, custom_utils)
│
└── Dependencies:
    ├── numpy (imports multiarray, ufunc, linalg...)
    │   └── Sub-dependencies: mkl, openblas...
    ├── requests (imports urllib3, certifi, charset_normalizer...)
    │   └── Sub-dependencies: idna, PySocks...
    └── custom_utils (imports pandas, scipy...)
        └── Sub-dependencies: pytz, dateutil...
```

**Naive approach**: Transpile everything → Massive effort, most code unused
**Smart approach**: Transpile only what's actually called → Minimal, efficient

### Selective Dependency Rewriting

TranspileAI uses **static analysis and runtime profiling** to identify which parts of dependencies are actually required:

1. **Call Graph Analysis**: Build a complete call graph from your entry point
2. **Dead Code Elimination**: Identify functions/classes that are never invoked
3. **Minimal Transpilation**: Only rewrite the subset of dependency code that's actually used
4. **Native Bindings**: Map to native libraries where possible (e.g., `numpy` → `ndarray`, `requests` → `reqwest`)

### Example: Minimal Dependency Rewrite

If your Python code only uses `numpy.array()` and `numpy.dot()`:

```python
# Original Python
import numpy as np

def compute(data):
    arr = np.array(data)
    return np.dot(arr, arr.T)
```

TranspileAI would:
- ✅ Transpile `numpy.array()` → map to `ndarray::Array::from_vec()`
- ✅ Transpile `numpy.dot()` → map to `ndarray` dot product
- ❌ Skip transpiling unused numpy functions (fft, linalg.svd, random, etc.)
- ✅ Result: Minimal Rust code with native performance

### Integration with tsrs

For TypeScript/JavaScript → Rust transpilation, this project integrates with **[tsrs](https://github.com/GeorgePearse/tsrs)**, which handles:

- TypeScript type system mapping to Rust types
- npm dependency resolution and selective inclusion
- JavaScript runtime feature detection (async/await, Promises, etc.)
- Bundler integration for tree-shaking before transpilation

### Dependency Resolution Strategies

1. **Direct Mapping**: `requests` (Python) → `reqwest` (Rust), `axios` (JS) → `reqwest` (Rust)
2. **Polyfill Generation**: For language-specific features not available in target
3. **Inline Vendoring**: Copy minimal required code from dependencies
4. **Foreign Function Interface (FFI)**: Keep critical dependencies in original language via FFI when transpilation is impractical

### Challenges & Solutions

| Challenge | Solution |
|-----------|----------|
| **Dynamic imports** | Static analysis + runtime profiling to capture all code paths |
| **Reflection/metaprogramming** | Partial evaluation and code specialization |
| **C extensions** | FFI bindings or rewrite in unsafe Rust |
| **Version conflicts** | Dependency pinning and compatibility layers |
| **Circular dependencies** | Dependency graph restructuring |

### Future Work

- **Automated dependency mapping**: LLM-powered matching of equivalent libraries across languages
- **Incremental transpilation**: Only re-transpile changed dependencies
- **Dependency caching**: Reuse previously transpiled dependency code
- **Community library mappings**: Crowdsourced database of library equivalents (e.g., `pandas` → `polars`)

**Note**: Handling dependencies is an active area of development. The gRPC testing infrastructure ensures that any transpiled dependency code behaves identically to the original implementation.

## Research Resources

### Classic Program Synthesis
- [Armando Solar-Lezama's Homepage](https://people.csail.mit.edu/asolar/) - MIT CSAIL
- [UC San Diego CSE 291: Program Synthesis](https://github.com/nadia-polikarpova/cse291-program-synthesis/wiki/Project-Ideas)
- [Building a Program Synthesizer](https://www.cs.utexas.edu/~bornholt/post/building-synthesizer.html) - James Bornholt

### Modern Neural Approaches
- [Code Translation with Compiler Representations](https://arxiv.org/pdf/2207.03578)
- [Verified Code Transpilation with LLMs](https://arxiv.org/html/2406.03003v1)
- [Guess & Sketch: Language Model Guided Transpilation](https://arxiv.org/html/2309.14396v2)

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

MIT

## Acknowledgments

This project stands on the shoulders of giants in the program synthesis community, particularly the foundational work of Armando Solar-Lezama, Emina Torlak, Rastislav Bodik, and many others who pioneered synthesis techniques before the LLM era.

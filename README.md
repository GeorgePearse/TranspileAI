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
├── src/
│   ├── main.rs          # Entry point
│   ├── parser/          # Language-specific parsers
│   ├── analyzer/        # Code analysis and understanding
│   ├── translator/      # Translation engine
│   └── validator/       # Output validation and testing
├── tests/               # Test suites
└── examples/            # Example translations
```

## Getting Started

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run the transpiler
cargo run -- --help
```

## Supported Languages (Planned)

- Python ↔ Rust
- JavaScript/TypeScript ↔ Rust
- Java ↔ Rust
- Go ↔ Rust
- And more...

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

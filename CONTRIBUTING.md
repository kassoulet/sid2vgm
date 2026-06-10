# Contributing to sid2vgm

Thank you for your interest in contributing to `sid2vgm`! We welcome all contributions, including bug reports, feature requests, documentation improvements, and code changes.

## Getting Started

1.  **Fork the repository** and clone your fork locally.
2.  **Install dependencies**: Ensure you have the Rust toolchain and `libsidplayfp` development headers installed.
3.  **Set up pre-commit hooks**: We use `pre-commit` to enforce code quality. Run the following command:
    ```bash
    pip install pre-commit
    pre-commit install
    ```
4.  **Create a branch** for your changes.

## Development Workflow

-   **Code Style**: We use `cargo fmt` to enforce code style. Ensure your code is formatted before submitting.
-   **Linting**: We use `cargo clippy` to check for common issues. Please fix all clippy warnings.
-   **Testing**: Run `cargo test` to ensure all tests pass, including the batch fidelity tests.

## Submitting Changes

1.  **Commit your changes** using descriptive commit messages (following [Conventional Commits](https://www.conventionalcommits.org/)).
2.  **Push your branch** to your fork.
3.  **Open a Pull Request** against the `main` branch of this repository.

Please make sure your changes are well-tested and documented.

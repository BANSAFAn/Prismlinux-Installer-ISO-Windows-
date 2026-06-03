# Contributing to Prism Linux Installer

Thank you for your interest in contributing to the Prism Linux Installer project! Follow these guidelines to ensure a smooth, efficient, and consistent development workflow.

## Code Style & Brand Rules

To preserve the minimalist, flat visual language of Prism Linux:
1. **No Gradients**: All UI components must use solid colors matching the Prism theme palette. Do not introduce linear or radial gradients.
2. **No Emojis**: Emojis are strictly prohibited in user-facing UI copy and CSS styles.
3. **No External Icon Libraries**: Do not install or import external icon packs (like Lucide or FontAwesome). Instead, define custom SVG paths inline or as clean Vue components.
4. **No Code Comments**: Do not write comments in any source files (Rust, TypeScript, Vue, or CSS). Code must be clean, readable, and self-documenting.

## Development Workflow

### 1. Run Locally
Prerequisites: Node.js (v20+) and Rust (stable toolchain).

```bash
npm install
npm run dev
```

### 2. Testing Your Changes
Before submitting your code:
- Ensure the Rust backend compiles cleanly with:
  ```bash
  cargo check --manifest-path src-tauri/Cargo.toml
  ```
- Verify the Vue frontend builds successfully without TypeScript or bundler errors:
  ```bash
  npm run build
  ```

### 3. Submitting Changes (Git Workflow)
We enforce an automated branch-to-PR integration pipeline:
- Create a feature or fix branch from `main` (e.g. `feat/my-new-feature` or `fix/issue-description`).
- Commit and push your changes to your remote branch.
- **Automated Pull Request**: The CI/CD bot will automatically detect your push and create (or update) a Pull Request targeting `main`. The PR body will automatically list all the commit messages from your branch.
- Do not push directly to `main`.
- Once the Pull Request is reviewed, approved, and merged into `main`, the CI/CD pipeline will automatically build production binaries for Windows and Linux, publishing a GitHub Release draft with the attached installers.

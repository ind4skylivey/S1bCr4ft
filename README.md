# S1bCr4ft

<div align="center">

<img src="assets/banner.png" alt="S1bCr4ft Banner" width="100%">

**Declarative System Configuration for Arch Linux**

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-Phase%201-yellow.svg)](https://github.com/S1b-Team/S1bCr4ft)

*Security-first, reproducible, auditable system setup for red teamers and security researchers*

[Vision](#-vision) • [Current Status](#-current-status) • [Roadmap](#-roadmap) • [Installation](#-installation) • [Documentation](#-documentation)

</div>

---

## 🎯 Vision

S1bCr4ft aims to be a **declarative system configuration framework** for Arch Linux. Think of it as bringing NixOS-style reproducibility to Arch, but without forcing you to learn Nix. The goal is simple: write your system configuration once in YAML, then recreate it anywhere, anytime, with complete confidence.

### Planned Features

When this project reaches its full potential, here's what it will offer:

- 🔒 **Security-First Architecture** - GPG signing, audit trails, sandboxed hooks
- 📦 **57+ Pre-built Modules** - Red team, malware analysis, AI/ML, window managers, development
- 🔄 **Reproducible Builds** - Same config = same system, every time
- 🛡️ **Hardening Built-in** - Kernel hardening, AppArmor/SELinux templates
- 🚀 **Fast & Efficient** - Parallel installation, minimal overhead
- **YAML Configuration** - Simple, human-readable configs
- **Module System** - Dependency resolution, conflict detection, versioning
- **Package Management** - Wrapper around pacman/paru/yay with atomic transactions
- **Backup/Rollback** - Timeshift integration, point-in-time recovery
- **Audit Logging** - Immutable, GPG-signed logs of all changes
- **Lua Scripting** - Pre/post-sync hooks for custom logic
- **Multiple Interfaces** - CLI, TUI, and REST API

---

## 🚦 Current Status

**Reality check:** This is alpha software. The foundation is solid and secure, but the main feature - actually installing packages - is still being built. 

### What Works Today

| Feature | Status | Details |
|:--------|:------:|:--------|
| **Configuration System** | ✅ | Load, validate, parse YAML configs |
| **Module Definitions** | ✅ | 57 YAML modules defined and ready |
| **Security Infrastructure** | ✅ | Command injection prevention, Lua sandbox |
| **CLI Framework** | ✅ | `init`, `validate`, `status` work great |
| **Testing** | ✅ | 60 tests passing, ~70% coverage |

### What's Still Cooking

| Feature | Status | Details |
|:--------|:------:|:--------|
| **Package Installation** | 🚧 | Detection works, installation stubbed |
| **`s1bcr4ft sync`** | 🚧 | Parses config, shows preview only |
| **TUI Interface** | 🚧 | Skeleton with demo data |
| **Backup/Rollback** | ⏳ | Structure exists, not wired up |
| **GPG Signing** | 🚧 | Infrastructure ready |
| **REST API** | 🚧 | Endpoints defined |

**Bottom line:** You can create configurations and validate them today, but you can't actually install anything yet. If you need something production-ready, check back in a few months.

---

## 🛣️ Roadmap

We're building this in phases. No dates promised - we're building it right, not fast.

### Phase 1: Foundation ✅ **COMPLETE**
The groundwork is done and tested:
- Core configuration engine (YAML parsing, validation)
- Security infrastructure (command injection prevention, Lua sandbox)
- 57 module definitions ready to go
- CLI framework with basic commands
- 60 tests passing

### Phase 2: Package Management 🚧 **IN PROGRESS**
This is the big one - actually installing stuff:
- Real pacman/paru/yay integration
- Module execution engine
- Working `s1bcr4ft sync` command
- Dependency resolution

### Phase 3: System Integration ⏳ **PLANNED**
Making it safe to experiment:
- Timeshift backup/rollback
- Dotfile management
- Configuration snapshots

### Phase 4: User Experience ⏳ **PLANNED**
Making it pretty and easy:
- Complete TUI with real data (not demo)
- Interactive module browser
- Better error messages

### Phase 5: Enterprise Features ⏳ **PLANNED**
For the power users:
- REST API completion
- Remote management
- Multi-machine orchestration

---

## 📦 Installation

### Method 1: From Source (Current)

```bash
git clone https://github.com/S1b-Team/S1bCr4ft.git
cd S1bCr4ft
cargo install --path crates/s1bcr4ft-cli
```

### Method 2: AUR (Coming Soon)

```bash
yay -S s1bcr4ft-git
```

---

## 🚀 Quick Start

### 1. Initialize a New Project

```bash
s1bcr4ft init my-arch-setup
cd my-arch-setup
```

### 2. Edit Configuration

```yaml
# config.yml
version: "1.0"
name: "my-arch-setup"
description: "My custom Arch Linux configuration"

modules:
  - core/base-system
  - core/bootloader
  - development/languages/rust
  - linux-optimization/terminal-config/zsh
  - security/hardening/kernel-hardening

options:
  auto_backup: true
  parallel_install: true
```

### 3. Validate (Works Today)

```bash
# Check if your config is valid
s1bcr4ft validate

# See what would be installed (preview only)
s1bcr4ft sync --dry-run
```

### 4. Install (Coming in Phase 2)

```bash
# This will actually install packages once Phase 2 is done
s1bcr4ft sync
```

---

## 📚 Documentation

### Available Commands

| Command | Status | Description |
|:--------|:------:|:------------|
| `s1bcr4ft init <name>` | ✅ | Initialize new project |
| `s1bcr4ft validate` | ✅ | Validate configuration |
| `s1bcr4ft status` | ✅ | Show system status |
| `s1bcr4ft sync --dry-run` | ✅ | Preview changes |
| `s1bcr4ft sync` | 🚧 | Apply configuration (Phase 2) |
| `s1bcr4ft module list` | 🚧 | Browse modules (Phase 2) |
| `s1bcr4ft module search` | 🚧 | Search modules (Phase 2) |
| `s1bcr4ft rollback <id>` | ⏳ | Rollback to backup (Phase 3) |
| `s1bcr4ft audit` | ⏳ | View audit log (Phase 3) |
| `s1bcr4ft health` | ⏳ | System health check (Phase 3) |

### Module Categories

We have 57 module definitions ready to go:

- **core/** - Base system, bootloader, kernel
- **development/** - Languages (Rust, Python, Go), tools (Docker, Git)
- **security/** - Hardening, firewalls, VPNs, AppArmor
- **red-team/** - Reconnaissance, exploitation, C2 frameworks
- **malware-analysis/** - Static/dynamic analysis, sandboxing
- **linux-optimization/** - Terminal configs, window managers, dotfiles
- **ai-ml/** - Ollama, CUDA, TensorFlow, PyTorch

### Example Configurations

- [Basic Arch](examples/basic-arch/config.yml) - Minimal setup
- [Developer Workstation](examples/developer-workstation/config.yml) - Hyprland + modern dev tools
- [Red Team Workstation](examples/red-team-workstation/config.yml) - Full C2 + exploitation
- [Malware Analysis Lab](examples/malware-analysis-lab/config.yml) - Isolated analysis environment

---

## 🏗️ Architecture

```
S1bCr4ft/
├── crates/
│   ├── s1bcr4ft-core/      # Core engine (parser, modules, packages)
│   ├── s1bcr4ft-cli/       # Command-line interface
│   ├── s1bcr4ft-tui/       # Terminal UI (Phase 5)
│   ├── s1bcr4ft-api/       # REST API (Phase 5)
│   └── s1bcr4ft-security/  # Security modules & presets
├── modules/                # Pre-built module library (57 ready)
│   ├── core/
│   ├── development/
│   ├── security/
│   ├── red-team/
│   └── malware-analysis/
└── examples/               # Example configurations
```

---

## ❓ FAQ

**Q: Can I actually use this to install packages right now?**  
A: Not yet. You can create configurations and validate them, but the actual installation happens in Phase 2. If you run `s1bcr4ft sync` today, it'll show you what *would* be installed, but won't actually do it.

**Q: Is this just another Ansible/Chef/Puppet clone?**  
A: Nope. Those are general-purpose tools that happen to work on Arch. S1bCr4ft is Arch-native - it understands pacman, AUR, and Arch-specific quirks out of the box. Plus it brings NixOS-style reproducibility without forcing you to learn Nix.

**Q: Why should I care about "declarative configuration"?**  
A: Instead of remembering what packages you installed and what configs you tweaked, you write it down once. Then you can recreate your exact setup on a new machine in minutes, roll back when something breaks, and share configurations with your team.

**Q: Is it safe to run on my main system?**  
A: In its current state, yes - because it doesn't actually change anything yet. Once Phase 2 hits and it starts actually installing packages, treat it like any other system tool: test in a VM first.

**Q: Will this work on distros other than Arch?**  
A: Short answer: no. Long answer: the architecture could theoretically support other distros, but the focus is 100% on Arch Linux. We use pacman, AUR helpers, and Arch-specific paths. If you want Ubuntu support, fork it.

**Q: How is this different from just using pacman/paru directly?**  
A: Three things: reproducibility (same config = same system), auditability (see exactly what changed and when), and rollback (undo changes when they break stuff). Plus you get to define your entire system in one YAML file instead of running 50 different commands.

**Q: Can I contribute?**  
A: The code is open source, so technically yes. But honestly, the core architecture is still settling. If you're itching to help, testing the `init` and `validate` commands and reporting bugs is super valuable right now.

**Q: When will this be production-ready?**  
A: When Phase 2 is done and we've battle-tested the package installation. Could be months. If you need something today, use Ansible or just write a shell script.

**Q: Why GPL-3.0 instead of MIT?**  
A: Because this is a system-level tool that touches security-critical stuff. GPL ensures derivatives stay open source. If you want to build a proprietary fork, you'll need to negotiate a commercial license.

---

## 📄 License

S1bCr4ft is released under the [GPL-3.0 License](LICENSE).

---

<div align="center">

<img src="assets/ending.png" alt="S1bCr4ft Ending" width="100%">

**Created by [ind4skylivey](https://github.com/ind4skylivey) • Maintained by [S1BGr0up](https://github.com/S1b-Team)**

⭐ Star us on GitHub if you find S1bCr4ft useful!

</div>

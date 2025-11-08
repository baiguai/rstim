# RSTIM - Vim-like Notes Editor

A terminal-based notes editor inspired by Vim, written in Rust.

## Features

### Current Implementation
- **Tree Mode**: Initial mode for managing the note tree structure
- **Black background UI** with dim gray panel separators
- **Three-panel layout**:
  - Left: Treeview (30% width)
  - Right: Editor (70% width) 
  - Bottom: Status line (3 lines height)
- **Node management**:
  - `a` - Add sibling node (at same level as selected)
  - `A` - Add child node (as child of selected)
  - `q` - Quit application
- **Visual indicators**:
  - 📁 for folders (nodes without content)
  - 📄 for notes (nodes with content)
  - Highlighted selection in treeview
  - Current mode display in status line

### UI Design
- Black background throughout
- No panel titles
- Dim gray borders separating panels
- Status line shows current mode (TREE/NORMAL/INSERT) and node count

## Usage

```bash
cargo run
```

## Controls (Tree Mode)
- `a` - Create sibling node
- `A` - Create child node  
- `q` - Quit

## Development

```bash
cargo build
cargo test
```

## Planned Features
- Navigation in treeview (j/k, h/l)
- Node renaming
- Node deletion
- Editor mode with Vim-like text editing
- Resizable treeview panel
- File persistence
- More Vim-like modes and bindings
# Family Tree Creator

A modern, interactive family tree visualization application built with Rust and egui.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-2024-orange.svg)

English | [日本語](README_ja.md)

## 📸 Screenshots

![Main Canvas View](screenshots/main-view.png)
*Interactive canvas showing persons, families, and events across 4 generations*

![Person Editor](screenshots/person-editor.png)
*Person management with relationship editor and inline editing*

![Family Groups](screenshots/family-groups.png)
*Family group editor with color customization and member management*

![Events](screenshots/events.png)
*Event management with timeline events and person relations*

## 🌟 Features

### Core Functionality
- **Interactive Canvas**: Drag-and-drop nodes to arrange your family tree
- **Pan & Zoom**: Navigate large family trees with smooth pan and zoom controls (Ctrl+Wheel)
- **Visual Relationships**: Clear lines connecting parents, children, and spouses
- **Multi-generational**: Support for unlimited generations

### Person Management
- **Rich Profiles**: Name, gender, birth/death dates, and custom memos
- **Deceased Status**: Special markers (†) for deceased family members
- **Age Calculation**: Automatic age calculation for living and deceased persons
- **Relationship Tracking**: Parents (biological/adoptive), children, and spouses
- **Editable Relations**: Edit relationship kinds (biological, adoptive, etc.) and spouse memos inline
- **Smart Positioning**: New persons appear at the top-left of the current canvas view

### Family Groups
- **Color-coded Groups**: Organize people into families with custom colors
- **Visual Boundaries**: Family groups displayed with colored backgrounds
- **Flexible Membership**: People can belong to multiple families

### Event Management
- **Timeline Events**: Create events with names, dates, descriptions, and custom colors
- **Event-Person Relations**: Link events to people with visual connections
- **Flexible Arrows**: Choose between lines, arrows to person (→), or arrows from person (←)
- **Smart Positioning**: New events appear at the top-left of the current canvas view
- **Draggable Nodes**: Move event nodes independently on the canvas

### User Interface
- **Bilingual Support**: Japanese and English interface
- **Four-panel Layout**: Persons, Families, Events, and Settings tabs
- **Grid Alignment**: Optional grid display with snap-to-grid functionality
- **Dark/Light Themes**: Follows system theme preferences

### Data Management
- **JSON Storage**: Save and load family trees in human-readable JSON format
- **Sample Data**: Pre-populated example family tree available at `examples/sample.json` (13 people, 4 generations)
- **Position Memory**: Manual node positions are preserved
- **Comprehensive Tests**: 40 unit tests covering core functionality

## 🚀 Getting Started

### Prerequisites

- **Operating System**: Windows (currently supported)
  - Other platforms (macOS, Linux) may work but are not officially tested
- [Rust](https://www.rust-lang.org/tools/install) (2024 edition or later)
- Cargo (comes with Rust)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/family-tree-creator.git
cd family-tree-creator
```

2. Build and run:
```bash
cargo run --release
```

### Quick Start

1. **Load Sample Data**: Enter `examples/sample.json` in the file path field and click "Load" to see an example family tree with 13 people across 4 generations
2. **Add a Person**: Click "➕ Add New Person" in the Persons tab
3. **Edit Relationships**: Select a person and use the relationship controls to add parents, children, or spouses
4. **Arrange Nodes**: Drag nodes on the canvas to position them
5. **Save Your Work**: Enter a filename and click "Save"

## 📖 Usage Guide

### Creating Your Family Tree

#### Adding People

1. Navigate to the **Persons** tab
2. Click **➕ Add New Person**
3. Fill in the details:
   - **Name**: Person's full name
   - **Gender**: Male, Female, or Unknown
   - **Birth Date**: Format `YYYY-MM-DD` (e.g., `1990-05-15`)
   - **Deceased**: Check if person is deceased
   - **Death Date**: Format `YYYY-MM-DD` (only if deceased)
   - **Memo**: Additional notes
4. Click **Update** to save

#### Adding Relationships

1. Select a person from the list
2. Scroll to the **Relations** section
3. View existing relationships:
   - **Parents**: Father, mother, or other parents with relationship kind
   - **Spouses**: With marriage date/memo
   - Click ✏️ button to edit relationship kind or spouse memo
   - Click ❌ button to remove the relationship
4. Add new relationships:
   - **Add Parent**: Select parent and specify relationship kind (biological/adoptive)
   - **Add Child**: Select child and specify relationship kind
   - **Add Spouse**: Select spouse and add marriage date/memo

#### Creating Family Groups

1. Navigate to the **Families** tab
2. Click **➕ Add New Family**
3. Enter a family name and choose a color
4. Select family members from the dropdown
5. Click **Update** to save

#### Adding Events

1. Navigate to the **Events** tab
2. Click **➕ Add New Event**
3. Fill in the details:
   - **Name**: Event name
   - **Date**: Format `YYYY-MM-DD` (optional)
   - **Description**: Event details
   - **Color**: Choose a custom color
4. Click **Add** to create the event
5. Link people to events:
   - Select the event
   - Choose a person from the dropdown
   - Select relation type: Line, Arrow → Person, or Arrow ← Person
   - Add optional memo
   - Click **Add** to create the link

### Canvas Controls

- **Pan**: Click and drag on empty canvas space
- **Zoom**: Hold Ctrl and scroll mouse wheel
- **Move Node**: Click and drag any person node
- **Select Person**: Click a node to select and edit

### Settings

Access the **Settings** tab to configure:
- **Language**: Switch between Japanese (日本語) and English
- **Grid**: Toggle grid display and adjust grid size
- **Layout**: Reset all positions to auto-calculated layout

#### Settings File (TOML)

- **Path**: `.family-tree-creator/settings.toml`
- **Load timing**: Loaded automatically on app startup
- **Save timing**: Saved automatically when values in the Settings tab change
- **Missing file**: Defaults are used if the file does not exist

The file stores the following keys:

```toml
language = "Japanese"          # Japanese | English
show_grid = true                # true | false
grid_size = 50.0                # 10.0 - 200.0
node_color_theme = "Default"   # Default | HighContrast
```

## 🏗️ Project Structure

```
family-tree-creator/
├── src/
│   ├── main.rs       # Application entry point
│   ├── app.rs        # Main application state
│   ├── ui/           # UI modules (trait-based)
│   │   ├── persons_tab.rs
│   │   ├── families_tab.rs
│   │   ├── events_tab.rs
│   │   ├── settings_tab.rs
│   │   └── canvas.rs
│   └── core/         # Core domain logic
│       ├── tree.rs   # Data model
│       ├── layout.rs # Layout engine
│       └── i18n.rs   # Internationalization
├── examples/
│   └── sample.json   # Sample family tree data
├── Cargo.toml
└── README.md
```

The application uses a **trait-based UI architecture** where each UI component (persons tab, families tab, settings, canvas) implements a trait on the `App` struct, keeping UI code modular and maintainable. Core domain logic is separated in the `core/` module.

## 🛠️ Technical Stack

- **Language**: Rust 2024 edition
- **GUI Framework**: [egui](https://github.com/emilk/egui) 0.33.3 / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) 0.33.3
- **Serialization**: [serde](https://serde.rs/) with JSON
- **ID Generation**: [uuid](https://github.com/uuid-rs/uuid)

### Key Dependencies

```toml
[dependencies]
eframe = "0.33.3"
egui = "0.33.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.11", features = ["v4", "serde"] }
```

## 📊 Data Format

Family trees are saved as JSON files with the following structure:

```json
{
  "persons": {
    "uuid": {
      "id": "uuid",
      "name": "John Doe",
      "gender": "Male",
      "birth": "1990-05-15",
      "deceased": false,
      "death": null,
      "memo": "Notes here",
      "position": [100.0, 200.0]
    }
  },
  "edges": [
    {
      "parent": "parent-uuid",
      "child": "child-uuid",
      "kind": "biological"
    }
  ],
  "spouses": [
    {
      "person1": "person1-uuid",
      "person2": "person2-uuid",
      "memo": "1990"
    }
  ],
  "families": [
    {
      "id": "family-uuid",
      "name": "Family Name",
      "members": ["uuid1", "uuid2"],
      "color": [200, 150, 100]
    }
  ]
}
```

## 🔮 Roadmap

See [TODO.md](TODO.md) for planned features, including:

- BC/AD notation for ancient family trees
- Enhanced date input (year-only format)
- Import/Export to GEDCOM format
- Photo attachments for people
- Timeline view
- Print-friendly layouts

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- [egui](https://github.com/emilk/egui) - Immediate mode GUI framework
- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) - Application framework for egui
- Rust community for excellent tooling and libraries

## 📧 Contact

Project Link: [https://github.com/yourusername/family-tree-creator](https://github.com/yourusername/family-tree-creator)

---

**Note**: This is an MVP (Minimum Viable Product) version. More features are planned for future releases!

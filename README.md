# 📔 Codexi CLI

**A high-integrity, anchor-based financial ledger built in Rust.**
> 🌐 [codexi.ethal.fr](https://codexi.ethal.fr)

![Rust](https://img.shields.io/badge/Rust-1.90.0-c5a059?logo=rust&style=flat-square)![License](https://img.shields.io/badge/License-MIT-gray?style=flat-square) ![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20macOS%20%7C%20Windows-black?style=flat-square)

---

## 📔 Description

Codexi is a robust, command-line personal finance management application built in Rust. It focuses on maintaining an accurate, auditable, and secure ledger of transactions through a system of anchor-based integrity checks and automatic archival.

---

## ✨ Features

* **Anchor-Based Integrity:** Ensures transaction history is tamper-proof by checking operation dates against system anchors (`INIT`, `CLOSE`, `ADJUST`).
* **Auditable Closing:** Periods can be formally closed, archiving all transactions into external files (`.cld`) while replacing them with a single **Carried Forward Balance** in the active ledger, ensuring financial traceability.
* **Data Security:** Supports full data backup to external ZIP archives, and restoration from these archives, securing both the active ledger and all historical archives.
* **Snapshot Recovery:** Offers internal snapshot functionality for quick rollback before risky operations (like bulk imports).

---

## 🚀 Installation

### Prerequisites

You need to have **Rust** and **Cargo** installed (v1.90.0 recommended).

### Build from Source

1.  Clone the repository:
    ```bash
    git clone [https://github.com/ethal/codexi.git](https://github.com/ethal/codexi.git)
    cd codexi
    ```
2.  Build and run the application using Cargo:
    ```bash
    cargo build --release
    ./target/release/codexi [COMMAND]
    ```
    *(Note: For simplicity, all subsequent commands assume you run them via `./target/release/codexi`)*

---

## 📖 Usage

### Core Operations

| Command | Description | Example |
| :--- | :--- | :--- |
| `init [amount] [date]` | Initialize the codexi with a initial amount. | `codexi init 150.00 2026-01-01` |
| `credit [date] [amount] [description]` | Adds funds to the ledger. | `codexi credit 2025-11-02 1500.00 Monthly Salary` |
| `debit [date] [amount] [description]` | Records an expense. | `codexi debit 2025-11-02 34.50 Grocery` |
| `search [Criteria]` | Displays the active transaction ledger with cumulative balances as per search criteria or all active transactions if no criteria | `codexi search` |

### Report Commands

| Command | Description | Example |
| :--- | :--- | :--- |
| `report balance [Criteria]` | Displays the balance of the active transaction ledger. | `codexi report balance` |
|` report resume` | Displays a resume of the active transaction ledger. | `codexi report resume` |

### System Commands

These commands manage the integrity and security of the ledger.

#### 1. Period Closing and Archival

| Command | Description | Example |
| :--- | :--- | :--- |
| `system close [date]` | Archives transactions and replaces them with a Carried Forward Balance entry (`CLOSE`). | `codexi system close 2025-11-30` |
| `system list` | Lists all closed archive files (`.cld`) in the data directory. | `codexi system list` |
| `system view [filename]` | Displays the operations contained within a specific archive file. | `codexi system view codexi_2025-11-30.cld` |

#### 2. Backup and Restore

Backups are created as compressed ZIP files containing the active ledger (`codexi.dat`) and all historical archives (`archives/`).

| Command | Description | Example |
| :--- | :--- | :--- |
| `system backup` | Creates a full backup ZIP file. Stores it in your system's **Documents** folder by default. | `codexi system backup` |
| `system backup --target-dir [path]` | Creates a full backup ZIP file at the specified location. | `codexi system backup --target-dir /media/usb/my_codexi.zip` |
| `system restore [path_to_zip]` | Restores the active ledger and archives from a backup ZIP file. **⚠️ Warning: This will overwrite current data.** | `codexi system restore /home/user/my_backup.zip` |

#### 3. Snapshots (Quick Recovery)

Snapshots are lightweight backups of **only** the active `codexi.dat` file, primarily used for quick rollback.

| Command | Description | Example |
| :--- | :--- | :--- |
| `data snapshot` | Creates a timestamped copy of the current `codexi.dat` file. (Used before `import` or bulk changes). | `codexi data snapshot` |
| `data list-snapshot` | Lists all available snapshots in the internal directory. | `codexi data list-snapshot` |
| `data restore-snapshot [filename]` | Restores the active ledger from a specific snapshot file. | `codexi data restore-snapshot codexi_20251208_101727.snp` |

---

## 🛡️ Data Integrity Workflow

Codexi manages your data through three distinct layers of safety:
```text
[ Active Operations ] --(snapshot)--> [ snapshots/ (.snp) ]
           |
     (system close)
           v
 [ archives/ (.cld) ] --(system backup)--> [ Full_Backup.zip ]
```

---

## 🗃️ Data Location

Codexi uses standard OS directories for storing its data to ensure compatibility and ease of access.

* **Active Ledger:** `codexi.dat`
* **Archives:** `[Data Directory]/archives/`
* **Snapshots:** `[Data Directory]/snapshots/`

The exact data directory path varies by OS:

| OS | Path |
| :--- | :--- |
| **Linux** | `~/.local/share/fr.ethal.codexi/` |
| **macOS** | `~/Library/Application Support/fr.ethal.codexi/` |
| **Windows**| `%AppData%\Roaming\fr.ethal.codexi\` |

---

## 🤝 Contributing

Contributions, bug reports, and feature requests are welcome! Feel free to open an issue or submit a pull request on GitHub.

## 📄 License

This project is licensed under the MIT License.

## 📬 Author

    ethal <ethal@ethal.fr>

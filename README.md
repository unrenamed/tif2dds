<h1 align="center">tif2dds</h1>

<p align="center">
    A Rust command line utility that automates <b>.TIF</b> and <b>.PNG</b> conversion to <b>.DDS</b> with format prompts.
</p>

<div align="center">
  <a href="https://www.gnu.org/licenses/agpl-3.0">
    <img src="https://img.shields.io/badge/License-AGPL_v3-blue.svg" alt="License: AGPL v3">
  </a>
  <a href="https://github.com/unrenamed/tif2dds/actions/workflows/build.yml">
    <img src="https://github.com/unrenamed/tif2dds/actions/workflows/build.yml/badge.svg?branch=main" alt="Build Status">
  </a>
</div>

## Core Features

- **Effortless Installation**: Get up and running with minimal setup.
- **No Extra UI**: Simply select your files, right-click, and convert - done!
- **Suffix-based format mapping**: If an image lacks alpha channel information, the tool will prompt you for clarification.
- **Automatic TIF to PNG Conversion**: The tool automatically converts .TIF to temporary .PNG files to avoid issues with the NVIDIA Texture Tools Exporter.

## Prerequisites

- Windows 10 or later
- [NVIDIA Texture Tools Exporter](https://developer.nvidia.com/texture-tools-exporter) installed

## Downloading a release

Pre-built executables of `tif2dds` are available via [GitHub Releases](https://github.com/unrenamed/tif2dds/releases). These binaries are automatically generated with every tagged commit.

> :warning: Currently supports **only Windows x86_64/ARM64**.

## Installation

It is recommended to create a separate folder for the `tif2dds` related files. You can choose any location that suits you.

Once you have downloaded the executable, follow these steps to complete the installation:

1. Place the `.exe` file in the folder created before.

2. Open `PowerShell` as administrator.

3. Use `cd` to navigate to the folder where the `.exe` file is located.

4. Run `.\tif2dds.exe install`

5. During the first installation, you will be prompted to provide the path to the **folder** containing the NVIDIA Texture Tools Exporter file. This file should be named `nvtt_export.exe`. You can either enter the path manually or copy and paste it from the file explorer.

If you have followed these steps correctly, you should see an output similar to the one below in your terminal:

```bash
✔ Please enter the path to the folder containing Nvidia Texture CLI: · C:\path\to\cli
Configuration file created successfully at C:\tif2dds\tif2dds_config.ini
Registering the context menu options...
Successfully added context menu for file type '.tif'.
Successfully added context menu for file type '.png'.
Installation process completed successfully!
```

Did you encounter any issues during the installation process? Feel free to open a new issue here: https://github.com/unrenamed/tif2dds/issues.

## How it works

**Installation and Context Menu:**

During installation, the tool adds a "Convert to DDS" option to the right-click context menu for .TIF and .PNG files in Windows Explorer.

**File Selection:**

The user right-clicks on one or more .TIF or .PNG files and selects "Convert to DDS" to begin the conversion process. Each selected file opens in a separate PowerShell window for conversion, which closes automatically after processing, freeing up memory

**Alpha Channel Prompt:**

If an image lacks an alpha channel, the tool prompts the user to choose the appropriate .DDS format (e.g., BC1 or BC3).

**Conversion Process:**

Nvidia's Texture Tools Exporter CLI is used to convert the files with selected settings, such as compression and mipmaps

**Output Location:**

The converted .DDS files are saved in the same directory as the original files, which remain unchanged.

### Suffix-based Format Mapping

The tool automatically assigns specific formats to files based on their suffixes, streamlining the process of format selection:

| Format  |                 Suffix                 |
| ------- | :------------------------------------: |
| **BC1** |                `\_dirt`                |
| **BC4** | `\_ao`, `\_rg`, `\_mt`, `\_hm`, `\_lm` |
| **BC5** |                 `\_nm`                 |

### Handling Unsupported Suffixes

If a file does not have a supported suffix, the tool will prompt you to manually select an appropriate .DDS format (e.g., BC1 or BC3) through a PowerShell window.

## Key Libraries:

- [dialoguer](https://crates.io/crates/dialoguer): Interactive prompts.
- [ini](https://crates.io/crates/rust-ini): INI file parsing.
- [clap](https://crates.io/crates/clap): CLI argument handling.
- [winreg](https://crates.io/crates/winreg): Accessing MS Windows registry.

## License

`tif2dds` is open-source under the GNU Affero General Public License Version 3 (AGPLv3) or any later version. You can [find it here](https://github.com/unrenamed/tif2dds/blob/main/LICENSE.md).

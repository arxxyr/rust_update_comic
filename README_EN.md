# Folder Compression Tool

## Introduction

This tool is a command-line program written in Rust, designed to compress all subfolders in a specified source directory and save them to a target directory. If the target directory already contains a corresponding compressed file and the modification date of the source folder is different from the existing compressed file, the folder will be recompressed.

## Features

- Traverses all subfolders in the specified source directory.
- Determines the need for compression based on the modification date of the folder.
- Generates or updates compressed files in the target directory.
- Optional automatic shutdown feature to turn off the computer after completing all compression tasks.

## Usage

1. Clone the repository:

    ```
    git clone https://github.com/arxxyr/rust_update_comic.git
    ```

2. Compile the project:

    ```
    cd rust_update_comic
    cargo build --release
    ```

    The compiled executable file will be located in the `target/release` directory.

3. Run the program:

    ```
    ./target/release/rust_update_comic --config=config.yaml
    ```

    `config.yaml` with your configuration file.

## Configuration

The `config.yaml` file should contain the following fields:

```yaml
source: "Path to Source Directory"
target: "Path to Target Directory"
shutdown: false  # or true to shut down the computer upon completion

```

## Development

This project is developed using the Rust programming language. If you are not familiar with Rust, you can visit the [Rust Official Website](https://www.rust-lang.org/) for more information.

## External Dependencies

- This program relies on the 7-Zip command-line tool `7z`. Please ensure it is installed on your system.
  - Ubuntu/Debian: `sudo apt-get install p7zip-full`
  - Fedora/RHEL: `sudo dnf install p7zip p7zip-plugins`
  - Windows: Download and install from the [7-Zip Official Website](https://www.7-zip.org/) 

After installation, you can verify the installation of `7z` by running it in the command line.

## License
This project is licensed under the MIT License. For more details, see the `LICENSE` file.

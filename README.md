# 文件夹压缩工具

## 简介

这个工具是一个 Rust 编写的命令行程序，用于压缩指定源目录中的所有子文件夹，并将它们保存到目标目录中。如果目标目录中已存在相应的压缩文件，且源文件夹的修改日期不同于现有压缩文件的修改日期，则重新压缩该文件夹。

## 功能

- 遍历指定源目录中的所有子文件夹。
- 基于文件夹的修改日期决定是否需要压缩。
- 生成或更新目标目录中的压缩文件。
- 可选的自动关机功能，在完成所有压缩任务后关闭计算机。

## 使用

1. 克隆仓库：

    ```bash
    git clone https://github.com/arxxyr/rust_update_comic.git
    ```

2. 编译项目：

    ```bash
    cd rust_update_comic
    cargo build --release
    ```

    编译好的可执行文件将位于 `target/release` 目录下。

3. 运行程序：

    ```bash
    ./target/release/rust_update_comic.exe --config=config.yaml
    ```

    `config.yaml` 为配置文件。

## 配置

配置文件 `config.yaml` 应包含以下字段：

```yaml
source: "源目录路径"
target: "目标目录路径"
shutdown: false  # 或 true，以在完成后关闭计算机
```

## 开发

这个项目使用 Rust 编程语言开发。如果您对 Rust 不熟悉，可以访问 [Rust 官网](https://www.rust-lang.org/) 了解更多信息。

## 外部依赖

- 本程序依赖于 7-Zip 命令行工具 `7z`。请确保将其安装在您的系统上。
  - Ubuntu/Debian: `sudo apt-get install p7zip-full`
  - Fedora/RHEL: `sudo dnf install p7zip p7zip-plugins`
  - Windows: 从 [7-Zip 官网](https://www.7-zip.org/) 下载并安装

在安装后，您可以通过在命令行中运行 `7z` 来验证其安装。

## 许可

此项目使用 MIT 许可证。有关详细信息，请参阅 `LICENSE` 文件。

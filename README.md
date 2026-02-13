# Portal

Hiverra Portal: A lightweight CLI tool to transfer files between devices locally or remotely.

# Overview

This project, "Portal" (also referred to as "Hiverra Portal" in some contexts), is a command-line tool designed for secure file transfer between devices. It operates locally and is configured for private, individual use.

# What this project does

Portal allows users to send files from one device to another. It can also be set up to receive files. The primary features include:

- **File Upload/Sharing:** Users can upload single or multiple files. This generates a link for others to download.
- **Secure Transfer:** All file transfers are conducted over HTTPS.
- **Responsive Interface:** The web interface (if applicable) is designed to work on various devices like phones, tablets, and desktops.
- **Status Messages:** The application provides feedback on the success or failure of operations.

Future planned features aim to enhance usability and functionality, such as https transfers, file previews, folder organization, search capabilities, personalized links, automatic file expiration, and notifications.

# Who it is for

Portal is intended for individuals who need a private and secure way to share files without relying on third-party services that might compress files or limit quality. It's suitable for personal use or small-scale file sharing.

# How to run or use it

The primary interface for Portal is a command-line tool built in Rust.

Here are the common commands:

- **Sending a file:**
    - `portal send <file_path>`: To send a specific file.
    - If no file is specified, it prompts the user to select a file.
    - You can also specify an `address` and `port` for the receiver: `portal send --address <IP_ADDRESS> --port <PORT> <file_path>`.

- **Receiving a file:**
    - `portal receive`: This command puts the application in listening mode to receive files.
    - You can specify a `port` to listen on: `portal receive --port <PORT>`.

- **Updating the application:**
    - `portal update`: This command is intended to update Portal to the latest version.

- **Configuration management:**
    - `portal config setup`: This command initiates an interactive setup process to configure the application, such as setting a username and default port.
    - `portal config set <key> <value>`: Allows setting specific configuration options (e.g., `portal config set port 8080`).
    - `portal config show <key>`: Displays the value of a specific configuration setting.
    - `portal config list`: Lists all current configuration settings.

The project also includes build workflows (`.github/workflows/`) that suggest it can be built for different platforms (e.g., Android via Termux, Linux, macOS, Windows). The specific installation method for compiled binaries would depend on how these workflows are triggered and what artifacts they produce.

# Notes

- **Initial Setup:** The application requires initial configuration, likely through `portal config setup`, before it can be fully used.
- **Receiver IP/Port:** When sending files, you need to know the IP address and port of the receiving device.
- **HTTPS vs. Local:** Portal primarily operates over local network transfers using TCP. The implementation of HTTPS for the web interface is not fully supported.
- **"Portal Quarantine Strategy":** This implies a multi-stage process for handling received files, including staging, extension verification, and potentially external scanning.

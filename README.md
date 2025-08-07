# Main System

This repository contains the core system for running ICT Union, organized into
several components with distinct responsibilities:

- **Gray Whale:** Handles database migrations.
- **Orca:** Backend for registration and member management.
- **Melon Head:** Frontend for registration and member management.
- **Keycloak:** Manages authentication and user identities.

Separating these components allows for more precise permission control and
a modular architecture.

## Prerequisites

- **WSL (on Windows):** Windows Subsystem for Linux, preferably with a
distribution like Ubuntu, is required to run the Nix Package Manager
development environment. For instructions on how to install WSL with an Ubuntu
distribution, please refer to [How to install Linux on Windows with WSL](https://learn.microsoft.com/en-us/windows/wsl/install).

---

## Setup and Run

The following instructions assume they will be followed step-by-step in the
order they are written from the root of the project source tree, including the
`cd` commands that use relative paths. If you have cloned the repository on the
host system, the sources must be cloned again in the target environment (e.g.,
a WSL or Docker container). If you need to interrupt the setup, you may need to
redo some of the previous steps. For example, if the host system is rebooted
during the frontend setup, the backend will need to be launched again before
you can proceed.

### Environment Setup

<details>
<summary>Click for <b>Windows with WSL</b> specific instructions</summary>

1. **Open your WSL terminal.**

2. **Install Docker:**
    ```sh
    sudo apt-get update
    sudo apt-get install -y docker.io
    ```

3. **Start Docker and grant permissions:**
    ```sh
    sudo systemctl start docker
    sudo usermod -aG docker $USER
    newgrp docker
    ```
    *You may need to restart your terminal for the group change to take effect.*

</details>

<details>
<summary>Click for <b>Ubuntu</b> specific instructions</summary>

1. **Install Docker:**
    ```sh
    sudo apt-get update
    sudo apt-get install -y docker.io
    ```

2. **Ensure the Docker daemon is running** and that your user can run Docker
    commands (usually by being in the `docker` group).
    ```sh
    sudo systemctl status docker.service
    ```
</details>

### Install Nix Package Manager

1. **Install Nix Package Manager:**
    ```sh
    sh <(curl --proto '=https' --tlsv1.2 -L https://nixos.org/nix/install) --no-daemon
    ```
    *This is required for operating systems other than NixOS.*
    *If you are running a WSL shell inside VSCode, the session needs to be
    restarted. Opening a new terminal is not enough; restarting VSCode does the
    trick.*

2. **Enable Nix Flakes (on Windows with WSL):**
    ```sh
    mkdir -p  ~/.config/nix/
    echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
    ```
    *For other environments, follow the instructions on the
    [NixOS Wiki for Flakes](https://nixos.wiki/wiki/flakes).*
    
The current terminal can be closed now.

### Start PostgreSQL, Keycloak, and Migrate the Database

In a **new terminal**:

1. **Enter the Nix development environment:**
    ```sh
    nix develop
    ```

2. **Start PostgreSQL and Keycloak:**
    ```sh
    make up
    ```
    *Keep this terminal running.*

The PostgreSQL database will be running at port 5432.

The Keycloak will be available at [http://localhost:8180](http://localhost:8180).

In a **new terminal**:

1. **Enter the Nix development environment:**
    ```sh
    nix develop
    ```

2. **Copy the migration tool configuration**
    ```sh
    cp gray-whale/refinery.example.toml gray-whale/refinery.toml
    ```

3. **Run Gray Whale migrations for Orca**
    ```sh
    make migrate
    ```

4. **Create a development Orca admin user in Keycloak**
    ```sh
    sudo apt-get install -y jq
    ./scripts/create-dev-user.sh <username> <password> [email] [firstName lastName]
    ```
    *Replace `<username>` and `<password>` with your desired credentials.*

The current terminal can be closed now.

### Run the Backend (Orca)

In a **new terminal**:

1. **Navigate to the `orca` directory**
    ```sh
    cd orca
    ```
    *This is necessary because Orca reads files from the directory where
    `cargo run` is launched.*

2. **Enter the Orca-specific Nix shell**
    ```sh
    nix develop .#orca
    ```

3. **Copy the configuration file**
    ```sh
    cp Rocket.example.toml Rocket.toml
    ```

4. **Run the backend server**
    ```sh
    cargo run
    ```
    *Keep this terminal running.*

The backend will be available at [http://localhost:8000](http://localhost:8000).

### Run the Frontend (Melon Head)

In a **new terminal**:

1. **Navigate to the `melon-head` directory**
    ```sh
    cd melon-head
    ```

2. **Enter the Melon-Head-specific Nix shell**
    ```sh
    nix develop .#melon-head
    ```

3. **Copy the configuration file**
    ```sh
    cp config.example.json config.json
    ```

4. **Install dependencies, build the frontend, and start the development server**
    ```sh
    npm install
    npm run build
    npm start
    ```

The frontend will be available at [http://localhost:1234](http://localhost:1234).

### Running Backend Tests

In a **new terminal**:

1. **Enter the Nix development environment:**
    ```sh
    nix develop
    ```

2. **Run the tests:**
    ```sh
    cargo test
    ```

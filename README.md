# nixv

A cli tool in rust to give insights to nix commands like [build ,develop ,nix-build ,nix-shell]

## Prerequisites

- Nix
- A Project with nix support

## Instalation

### To build and install using nix

```BASH
nix build github:eswar2001/nixv
echo `pwd`/result/bin/
```

then add the above path in bashrc or run the following command

```BASH
nix-env -iA ./result
```

### To build and install using cargo

```BASH
git clone https://github.com/eswar2001/nixv
cd nixv && cargo build --release
```

then add the above path in bashrc or run the following command

## Usage

```BASH
# to get insights for nix build
nixv build [args]
# to get insights for nix develop
nixv develop [args]
# to get insights for nix-shell
nixv-shell [args]
# to get insights for nix-build
nixv-build [args]
```

To toggle logging level use ENV [RUST_LOG]  
Possible values [ error , warn , info , debug , trace]

```BASH
export RUST_LOG=info 
```

To dump logs to files set ENV [DUMP_LOGS]

```BASH
export DUMP_LOGS=true
```

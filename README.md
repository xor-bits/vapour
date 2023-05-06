# vapour

general purpose cli tool for Steam related tasks

## Install
```bash
git clone https://github.com/xor-bits/vapour
cd vapour
cargo install --path .
```

## Usage
```bash
# get the proton pfx path for games that have 'civ' in their names
vapour compat-data 'civ' -d
# example output:
# Sid Meier's Civilization VI:
# - /home/username/.local/share/Steam/steamapps/compatdata/289070/pfx/drive_c

# appid => name translation
vapour app-id 40
# outputs:
# 40:
# - Deathmatch Classic

# piping
vapour compat-data 'civ' | head -n 1
# example output:
# /home/username/.local/share/Steam/steamapps/compatdata/289070/pfx/drive_c
```

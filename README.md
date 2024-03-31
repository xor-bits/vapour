# vapour

general purpose cli tool for Steam related tasks

## Install
```bash
cargo install --locked --git https://github.com/xor-bits/vapour
```

## Usage

### get app-id of installed games matching regex 'factor'

```console
$ vapour id-of -i factor
  Finished opening database
  Finished loading Steam libraryfolders.vdf
Factorio: 427520
Satisfactory: 526870
```

### get compatdata directory of games matching regex 'civ'
```console
$ vapour compat-data rocket
  Finished opening database
  Finished loading Steam libraryfolders.vdf
Rocket League: /home/username/.local/share/Steam/steamapps/compatdata/252950
```

### appid => name translation
```console
vapour name-of 40
  Finished opening database
Deathmatch Classic
```

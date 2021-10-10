<h1><p align="center"> Good Boy 🐶 </p></h1>

<p align="center"> 
    A Game Boy emulator in Rust <br />
    <small> Sound not included </small>
</p>

<div align="center">
    <img src="assets/showcase/pokemon_red.png" width="420px" />
    <br />
    <small> Game: Pokémon Red </small>
</div>

## Features

This emulator is not perfect and lack some features. Core features that are worthy listing:

- Pass Blargg's cpu_instrs tests
- Needs a little more improvements to pass dmg_acid test, but is still doing well
- Joypad implemented
- Support to MBC0, MBC1, MBC3 (without timer) and MBC5 (without rumble) cartridge

There are also some non-core features that i implemented just for fun:

- FPS counter overlay
- Support for multiple color schemes

Planned features:

- CLI arguments
- Run on browser
- Quicky save/restore state
- Implement MBC3 timer
- SGB support (Super Game Boy)
- CGB support (Game Boy Color)
- Sound support
- A better FPS overlay (maybe)

## Controls

Basic keyboard bindings

| Keyboard Key       | Game Boy Button |
| ------------------ | :-------------: |
| <kbd>Z</kbd>       |        A        |
| <kbd>X</kbd>       |        B        |
| <kbd>↵ Return<kbd> |      Start      |
| <kbd>Space<kbd>    |     Select      |
| <kbd>←</kbd>       |      Left       |
| <kbd>→</kbd>       |      Right      |
| <kbd>↑</kbd>       |       Up        |
| <kbd>↓</kbd>       |      Down       |

Other bindings:

| Keyboard Key                        |        Action         |
| ----------------------------------- | :-------------------: |
| <kbd>F1</kbd>                       | Disable the FPS limit |
| <kbd>Tab</kbd>                      |   Next Colorscheme    |
| <kbd>⇧ Shift</kbd> + <kbd>Tab</kbd> | Previous Colorscheme  |
| <kbd>Ctrl</kbd> + <kbd>Q</kbd>      |         Exit          |

## See it in action

### Changing color schemes on the fly!

<div align="center">
    <img src="assets/showcase/changing_color_schemes.gif" width="420px" />
    <br />
    <small> Game: The Legend of Zelda: Link's Awekening </small>
</div>

### In-game screenshots

<div align="center">
    <table>
    <tr>
        <td>
            <img src="assets/showcase/dr_mario.png" width="360px" />
            <br />
            <small> <p align="center"> Game: Dr Mario </p> </small>
        </td>
        <td>
            <img src="assets/showcase/disco_elysium.png" width="360px" />
            <br />
            <small> <p align="center"> Game: Disco Elysium (Demake) </p> </small>
        </td>
        <td>
            <img src="assets/showcase/mario_land_2.png" width="360px" />
            <br />
            <small> <p align="center"> Super Mario Land 2: 6 Golden Coins </p> </small>
        <td>
    </tr>
    </table>
</div>

## How do i run it on my machine?

First, clone and cd to the project (Duh):

```sh
git clone https://github.com/sixels/goodboy && cd goodboy
```

It is supposed to be cross platform (however I tested it only on Linux). Setup Rust and Cargo then build the project:

```sh
cargo build --release
```

run with:

```sh
./target/release/sixels_gb PATH/TO/ROM.gb
```

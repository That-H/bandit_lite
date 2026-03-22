[bandit]: https://github.com/That-H/bandit
[crossterm]: https://crates.io/crates/crossterm
# Bandit Lite

Bandit Lite is a game about reflecting and manipulating light made using pure rust. It uses [bandit][bandit] to handle the game's state, and [crossterm][crossterm] as
the terminal frontend.

## Gameplay

The game is an entirely grid based puzzle game where the only objective is to move to the exit (a green exclamation mark). Doing so
will complete the puzzle. Of course, there will be things preventing this from being trivial. The specific behaviour of puzzle elements
is up to the player to discover.

### Controls

Movement can be achieved using one of the below schemes:
- WASD
- Arrow Keys
- HJKL (for the vim users among us)

Additionally, the U key can be used to undo the previous movement, and the escape key will bring up a pause menu during a puzzle.

## Menus

Menus can be navigated using one of the schemes listed in [controls](#Controls), with the enter or space key additionally being required
to select menu options.

### Entry Boxes

These must first be selected with enter or space. Text entry will then occur, with the backspace button deleting characters and
the arrow keys causing the cursor to move. Once the desired text has been entered, pressing enter or the escape key will deselect 
the entry box.

### Puzzle Editor

Once a puzzle pack and a puzzle within that have been selected, puzzle editing will begin. In this state, the cursor is a blue 
highlight on a puzzle grid. This cursor may be moved using one of the control schemes. Once the user is happy with the current puzzle,
they may save it using the menu brought up using the escape key. It is then safe to quit the editor.

#### Editor Controls

The following keybinds are used for puzzle manipulation in the editor:
- Y and I: rotates the current object.
- Enter: places the current object at the cursor.
- Backspace: deletes the object at the cursor.
- O: Brings up the object selector.
- M: Toggles whether the object at the cursor can be pushed by the player. This will highlight it grey. Some objects (such as walls) can't be toggled. 
- Escape: Brings up the editor pause menu, where one may save the current puzzle state, test the puzzle, return to editing, or quit the editor,
which will erase the current puzzle state. If the puzzle state was saved, then it will remain as such.

## Running The Game

To run the game, a release can be directly installed from the repository's releases tab, which will contain a binary file.
Alternatively, the game can be built from source using Cargo. See [Building From Source](#Building-From-Source).
> Note: It is not recommended to versions before 0.3.0 due to puzzle formatting issues.

### Building From Source

As the game developed in Rust using Cargo, this can naturally be used to build the game. Simply clone the repository using git (or otherwise 
obtain a copy of the source files), then run the following Cargo command in the directory where the source files are located to build the source:

    cargo run --release

If you do not have Cargo installed, see its [github repository](https://github.com/rust-lang/cargo).


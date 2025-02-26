# Arash's fork of evremap

After having had issues with the original implementation in the sense that the modifier key usage didn't work if one typed too fast, I decided to look into the bug and got interested in the code base. I decided to fork it to learn more about rust and solve the issue for myself. But since I changed the code too much I gave up on making it possible to merge back in.

The main difference is the change and simplification of the `[[remap]]` type (and config). It now only takes a single in and out key (instead of lists), but instead introduces a list of modifiers that must be set for the conversion to happen.  I think this is what you would like if you like me see evremap as a replacement for writing custom but small keymappings. 

Additional changes:

* Restructured code base (modularity, unit tests, etc.)
* Removed DualRole

The original code is at: <https://github.com/wez/evremap>

## Dump of some commands I've used

```{bash}
# Random cargo commands
cargo test
cargo build
cargo build --release

# Running the tool
sudo ~/repos/evremap/target/release/evremap remap ~/dotfiles/swedish_colemak/arash_swedish_colemak.toml

# Debugging keys and stuff
sudo ~/repos/evremap/target/release/evremap list-devices
sudo ~/repos/evremap/target/release/evremap debug-events --device-name='AT Translated Set 2 keyboard'
```
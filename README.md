# Pitchplay - The Crappy Note Trainer™

A bare bones MIDI-based game to help you practice reading notes on a staff and hitting them on your keyboard.
It barely works and the code is a messy pile of 💩. But hey, it was vibe-coded in about 30 minutes, so don't complain.
I probably should have found out how to read MIDI signals in a browser and made this web based, like a sane person.

## Features 💯
- Displays random notes between C4 and H4 on a staff.
- Lets you respond by pressing the correct key on an attached MIDI keyboard.
- Gives immediate feedback: ✅ Correct or ❌ Wrong.
- Lets you skip with the spacebar if you’re stuck...

## What it doesn’t do
- Any kind of polished UI.
- Support for accidentals (flats/sharps).
- Notice when you attach a MIDI device after starting the game.
- Keep score.
- Show more than one note at a time.
- Graceful error handling.

## Installation

You’ll need Rust installed. Then clone this repo and run:

```bash
cargo run
```

Make sure you have a MIDI keyboard connected — the program will auto-detect the first available MIDI input.

## Disclaimer

This was hacked together for personal use. It’s buggy. It’s ugly. But it gets the job done (sometimes).
No guarantees of correctness, stability, or sanity.

## License
No formal license. Use it, modify it, share it. Just don’t blame me when it breaks.

Enjoy!

# Battle Cats Gacha Seeker

A tool for quickly finding gacha seeds in The Battle Cats.

You can search for seeds that contain a specific pattern of cats or seeds that
contain a specific pattern of rarities.

I am very new to rust, so this code probably isn't great.

The seed tracking code is based on godfat's work here:
<https://gitlab.com/godfat/battle-cats-rolls> and you can view the cats for any
seed here: <http://bc.godfat.org/>

## Prerequisites

- [Rust](https://www.rust-lang.org/)
- [Cargo](https://crates.io/) (If you installed rust using rustup, you should
  already have cargo)
- [Git](https://git-scm.com/)

## Usage

I don't have a release yet, so you will have to build from source.

```bash
git clone https://github.com/fieryhenry/BattleCatsGachaSeeker.git
cd BattleCatsGachaSeeker
cargo run --release
```

## Bevy_ggf

[![Crates.io](https://img.shields.io/crates/v/bevy_ggf.svg)](https://crates.io/crates/bevy_ggf)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/NoahShomette/bevy_ggf/blob/main/LICENSE-MIT)
[![Crates.io](https://img.shields.io/crates/d/bevy_ggf)](https://crates.io/crates/bevy_ggf)

# What is this?

Bevy Grid Game Framework (Bevy_ggf), is a framework for creating grid based tactics and strategy games in the Bevy game
engine. This framework is intended to provide easy to use yet extensible systems for quickly putting together basic
grid based tactics games and allow them to be customized and extended as deeply as possible. If you run into a problem
with something that goes against this goal, open an [issue](https://github.com/NoahShomette/bevy_ggf/issues/new/choose)
and we will work to address it!

Currently the focus is on building the framework for tactics games in the style of Advance Wars or Final
Fantasy. Long term goals are to extend the framework to support larger and more complex strategy games like
Civilization.

# Crate Status

This crate is actively being developed however it is still very early in development. Any advice, opinions, pull
requests, issues, etc would be greatly appreciated!

Check out the [help wanted](https://noahshomette.github.io/bevy_ggf/development/help_wanted.html) section of the mdbook
if you want to see where your efforts might be msot helpful, however we appreciate any help even if its not from this
list!

**Version Compatibility Table:**

| Bevy Version | Crate Version |
|--------------|---------------|
| `0.9`        | `main`        |

---

# Current Features

* Massively inprogresss

# In Progress Features

- Mapping
- Movement
- Combat
- Units

# MVP Requirements

In no particular order, here are the features required for a true 1.0 release. These features would provide all the
fundamentals needed to create a game in the style of Advance Wars.

- [ ] Mapping
- [ ] Movement
- [ ] Combat
- [ ] Units
- [ ] Buildings
- [ ] Camera
- [ ] Selection
- [ ] Win/Lose Conditions
- [ ] Game management
- [ ] Saving/Loading
- [ ] Built in scene/map editor

---

# Helpful Links

To learn how to use Bevy_ggf see the wiki:

[Wiki Tutorial](https://github.com/NoahShomette/bevy_ggf/wiki#getting-started)

If you are interested in helping develop the project, check out the design mdbook

[bevy_ggf mdbook](https://noahshomette.github.io/bevy_ggf/)


---

# Dependencies

Bevy_ggf depends on the outstanding community of developers and their excellent crates that have helped to make Bevy_ggf
what it is. Some of the bigger crates that this project depends on are listed below. Thank you to everyone who has
contributed to these crates!

* [Bevy](https://github.com/bevyengine/bevy)
* [Bevy_Ecs_Tilemap](https://github.com/StarArawn/bevy_ecs_tilemap)
* [Leafwing-Input-Manager](https://github.com/Leafwing-Studios/leafwing-input-manager)
* [iyes_loopless](https://github.com/IyesGames/iyes_loopless)

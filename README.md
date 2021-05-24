[![Build status](https://github.com/jorgenpt/uasset-rs/workflows/Rust/badge.svg)](https://github.com/jorgenpt/uasset-rs/actions?query=workflow%3ARust)
[![Crate](https://img.shields.io/crates/v/uasset.svg)](https://crates.io/crates/uasset)
[![API](https://docs.rs/uasset/badge.svg)][docs-rs]
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE-APACHE)

# `uasset`

`uasset` supports parsing `.uasset` files from Unreal Engine in pure Rust, to aid in building tools that reason about Unreal assets without needing to
boot up an entire editor. Most of the format has been gleaned from Unreal Engine's own parsing code, which you can find in [the official UnrealEngine repo][unrealengine]
if you have permission. (Specifically, a lot of it comes from [PackageFileSummary.h][packagefilesummary-h] and [PackageFileSummary.cpp][packagefilesummary-cpp]).

It's designed to work with Unreal Engine assets as old as 4.10 (but might work farther back -- let me know!), and it's intended to be updated to work with the latest engine
version (at time of writing, that is 4.26).

For details on how to use `uasset`, please refer to [the documentation on docs.rs][docs-rs]

## License

`uasset-rs` is licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[unrealengine]: https://github.com/EpicGames/UnrealEngine/
[packagefilesummary-h]: https://github.com/EpicGames/UnrealEngine/blob/master/Engine/Source/Runtime/CoreUObject/Public/UObject/PackageFileSummary.h
[packagefilesummary-cpp]: https://github.com/EpicGames/UnrealEngine/blob/master/Engine/Source/Runtime/CoreUObject/Private/UObject/PackageFileSummary.cpp
[docs-rs]: https://docs.rs/uasset

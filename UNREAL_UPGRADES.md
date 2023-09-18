Here are the steps I take when upgrading uasset-rs to a new version:

1. Install the new engine version from Epic Games Store (in this case, 5.1.1)
1. Copy the assets from the previous version into a new project in the new engine version (i.e. [assets/UE50](https://github.com/jorgenpt/uasset-rs/tree/main/assets/UE50))
1. Open the project in the new engine version
1. Re-save all the assets with the new engine
1. Copy the re-saved assets into the assets directory (i.e. [assets/UE51](https://github.com/jorgenpt/uasset-rs/tree/main/assets/UE51))
1. Run the commandline tool against the test assets (`cargo run --features=commandline-tool validate .\assets\UE51\`)
1. See if anything goes wrong and if so, address it

That last bullet point of course is doing a lot of work, and it's got to be considered on a case-by-case basis. I'll go through what I did for 5.1.

The immediate failure is this:

```
.\assets\UE51\DirectCycle\DirectCycleA.uasset: Could not parse asset: failed to parse .\assets\UE51\DirectCycle\DirectCycleA.uasset: UnsupportedUE5Version(1008)
```

This means that `EUnrealEngineObjectUE5Version` from `Engine/Source/Runtime/Core/Public/UObject/ObjectVersion.h` has a new value, 1008! If you open up [ObjectVersion.h](https://github.com/EpicGames/UnrealEngine/blob/5.1/Engine/Source/Runtime/Core/Public/UObject/ObjectVersion.h#L39) from the 5.1 branch, you'll see that there are two new entries that I then add to [ObjectVersionUE5](https://github.com/jorgenpt/uasset-rs/blob/e7f401753992c403e4af76e6ef1fcd70f12c7562/src/enums.rs#L330).

The next step is to add tests that automatically try to load the newly added assets. I go to [the test_utilities](https://github.com/jorgenpt/uasset-rs/blob/main/test_utilities/src/lib.rs) and [add a new test case for 5.1](https://github.com/jorgenpt/uasset-rs/commit/beda5a1294133a82a86fc8eca36265315d888eef#diff-db822ca26e41e0762e8bdc9eeb054a90a40abaf8937a3d3d98f577f5b83c4956R148) with the expected `ObjectVersionUE5::ADD_SOFTOBJECTPATH_LIST` version. A quick `cargo test` will reveal these tests as failing, because the new object version requires appropriate migration code.

The `FSOFTOBJECTPATH_REMOVE_ASSET_PATH_FNAMES` migration you can find in `FSoftObjectPath::SerializePathWithoutFixup` -- it looks like a field that used to be serialized as an `FName` is now serialized as an `FString`. We don't currently do any serialization of `FSoftObjectPath`, so we don't need to make any changes for this.

Reviewing `ADD_SOFTOBJECTPATH_LIST`, we can find a reference to it in [`FPackageFileSummary`'s serialization code](https://github.com/EpicGames/UnrealEngine/blob/5.1/Engine/Source/Runtime/CoreUObject/Private/UObject/PackageFileSummary.cpp#L217). This is relevant to us, as this is a structure we serialize. The format of the serialization is what is called "indirect array" serialization in uasset-rs. It means that it's serializing a list of entries by encoding a count & offest in the header of the asset, and you can seek to that offset to deserialize the relevant entries (`FSoftObjectPath` in this case). We don't have any immediate use for this header data, and we haven't added `FSoftObjectPath` deserialization yet, so we leave it as-is and [just load the `int32` values into `AssetHeader`](https://github.com/jorgenpt/uasset-rs/commit/c79a3721fecb630aa39c66f189c8b1e541520b88#diff-b1a35a68f14e696205874893c07fd24fdb88882b47c23cc0e0c80a30c7d53759R348). Run the tests again, and tada! It successfully loads all the old _and_ the new 5.1 assets correctly.

Next up I need to do the same for 5.2 and 5.3, but not today.

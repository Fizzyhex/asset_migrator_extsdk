# Asset Migrator (Extended SDK)

This is a modified version of the asset migrator that reads files from the package cache. The syntax is as follows:

```sh
./prefab_converter.exe [src assets path] [dst assets path]
```

Here's an example of the tool being used to migrate assets from one Unity project to another, feeding the package name of the [Marrow Extended SDK](https://github.com/notnotnotswipez/Marrow-ExtendedSDK-MAINTAINED) into the `[package name]` argument:

```
./prefab_converter.exe "C:\Bonelab\Patch3\Avatars\AvatarProject\Assets" "E:\Documents\Bonelab\Patch4\Avatar Project\Avatar Project\Assets" "C\UnityProject\Assets\Prefabs\MyAvatar.prefab" "C\UnityProject\Assets\Prefabs\MyAvatar (1.5x) Variant.prefab"
```

## Asset Migrator (For Unity modding)

Ever had files from one Unity project you wanted to migrate to another?

Usually you'd export a package, but this isn't ideal when exporting giant amounts of assets!

Or ever wanted to port a modding project from a prior game to a new one? Given the script's names remain the same, asset migrator should be able to port your favorite scenes / prefabs!

Asset migrator helps solve that problem by resolving meta files for you and copying data properly!

### Special Thanks

* [notnotnotswipez](https://github.com/notnotnotswipez) - Thank you for writing the original!
* Fersy - Thank you for testing!
* DayTrip - Thank you for testing!

### DISCLAIMER!
This is a rewrite + extension of https://github.com/notnotnotswipez/PrefabConverter to Rust!
All credit goes to notnotnotswipez for his original implementation.

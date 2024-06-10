// ===================================================================================
//  BSD 3-Clause License
//
//  Copyright (c) 2023-2024, Liam R. (zCubed3)
//
//  Redistribution and use in source and binary forms, with or without
//  modification, are permitted provided that the following conditions are met:
//
//  1. Redistributions of source code must retain the above copyright notice, this
//     list of conditions and the following disclaimer.
//
//  2. Redistributions in binary form must reproduce the above copyright notice,
//     this list of conditions and the following disclaimer in the documentation
//     and/or other materials provided with the distribution.
//
//  3. Neither the name of the copyright holder nor the names of its
//     contributors may be used to endorse or promote products derived from
//     this software without specific prior written permission.
//
//  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
//  AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
//  IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
//  DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
//  FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
//  DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
//  SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
//  CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
//  OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
//  OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
// ===================================================================================

mod dropwatch;
mod meta_file;

use std::collections::HashMap;
use std::env;
use std::fs::*;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

use crate::meta_file::*;

#[derive(Default, Debug)]
struct AssetConversion {
    path: String,
    output_path: String,
}

impl PartialEq<AssetConversion> for AssetConversion {
    fn eq(&self, other: &AssetConversion) -> bool {
        self.path == other.path
    }
}

fn print_help() {
    println!("Proper usage of prefab_converter.exe is as follows\n");
    println!("./prefab_converter.exe [src assets path] [dst assets path]");
    println!("\n... = Any number of valid prefab paths (in the source assets path)!");
    println!("\nExample:");
    println!("\n./prefab_converter.exe \"C:/CustomItemsSDK/Assets\" \"C:/MarrowSDK/Assets\"");
}

fn main() {
    sleep(Duration::from_millis(10000u64));

    // Handle arguments
    let args: Vec<String> = env::args().collect();

    let (src_assets, dst_assets) = if args.len() > 1 {
        // Minimum is 4
        if args.len() < 3 {
            print_help();
            return;
        }

        (args[1].clone(), args[2].clone())
    } else {
        print_help();
        return;
    };

    // Before we export, create the temp folder
    let _ = create_dir("ConversionOutput");
    let export_path = "./ConversionOutput".to_string();

    let convert_extensions = {
        let mut vec = Vec::<String>::new();

        let default_extensions = vec![".prefab", ".mat", ".asset", ".unity", ".controller"];

        for ext in default_extensions {
            vec.push(String::from(ext));
        }

        if let Ok(file) = read_to_string("./extensions.txt") {
            // For each line, add it to the extension list
            for line in file.lines() {
                // Is this a comment?
                if line.starts_with("#") {
                    continue;
                }

                // Empty line?
                if line.is_empty() {
                    continue;
                }

                let str = String::from(line);

                if !vec.contains(&str) {
                    vec.push(str)
                }
            }
        }

        vec
    };

    println!("-- [Run Info] --");

    println!("Target Extensions:");
    for ext in &convert_extensions {
        println!("\t{}", ext);
    }

    println!("--============--");

    //
    // Collection stage
    //

    // We read two projects worth of hash files
    // Any overlap between the two is eliminated (we assume the asset already exists properly)
    let mut missing_metas = Vec::<MetaFile>::new();
    let mut remapped_metas = HashMap::<String, MetaFile>::new();

    println!("-- [Collection Stage] --");
    print!("If this is the first time you've done this since rebooting");
    println!(" you might have to wait a second or two for the OS to cache files and directories!");
    println!("Subsequent runs should be much faster though!");
    println!("--====================--");

    {
        println!("Collecting source meta files...");
        let src_metas = collect_meta_files(&src_assets);

        println!("Collecting destination meta files...");
        let mut dst_metas = collect_meta_files(&dst_assets);

        // The source package will be a folder up from the dst assets, in 'Library/PackageCache'
        println!("Collecting meta files from package cache...");

        let src_package_cache = {
            let mut package_cache = PathBuf::from(&dst_assets);
            package_cache.pop();
            package_cache.push("Library");
            package_cache.push("PackageCache");
    
            package_cache
        };
    
        let src_package_str = src_package_cache.as_path().to_str().unwrap();
        println!("Package cache path: {src_package_str}");

        let mut package_metas = collect_meta_files(&src_package_str.to_owned());

        // Remove duplicate meta files that have the same name in dst_metas from package_metas
        package_metas.retain(|package_meta| {
            !dst_metas.iter().any(|dst_meta| dst_meta.base_name == package_meta.base_name)
        });

        dst_metas.append(&mut package_metas);

        //let drop = Dropwatch::new_begin("OVERLAPPING");

        println!("Determining missing meta files...");
        for src_meta in &src_metas {
            //println!("{:?}", src_meta);

            let mut same_found = false;

            for dst_meta in &dst_metas {
                if src_meta.guid_hash == dst_meta.guid_hash {
                    same_found = true;
                    break;
                }

                // Is this the same asset but with a different GUID?
                if src_meta.base_hash == dst_meta.base_hash {
                    same_found = true;
                    remapped_metas.insert(src_meta.guid.clone(), dst_meta.clone());
                    break;
                }
            }

            if !same_found {
                missing_metas.push(src_meta.clone());
            }
        }
    }

    //
    // Conversion stage
    //
    println!("-- [Conversion Stage] --");
    println!("Please be patient, conversion may take a while!");
    println!("--====================--");

    let mut convert_queue = Vec::<AssetConversion>::new();

    for prefab in args.iter().skip(3) {
        let prefab_dir = PathBuf::from(prefab);
        let mut relative_export_path = PathBuf::from(&export_path);

        let sanitized = {
            if let Ok(prefab) = prefab_dir.strip_prefix(&src_assets) {
                prefab
            } else {
                prefab_dir.as_path()
            }
        };

        relative_export_path.push(sanitized);
        relative_export_path.pop();

        let mut import = PathBuf::from(prefab);

        if !import.starts_with(&src_assets) {
            import = PathBuf::from(&src_assets);
            import.push(prefab);
        }

        convert_queue.push(AssetConversion {
            path: import.display().to_string(),
            output_path: relative_export_path.display().to_string(),
        });
    }

    while let Some(convert) = convert_queue.pop() {
        let prefab_path = Path::new(&convert.path);

        if prefab_path.is_dir() {
            continue;
        }

        let mut prefab_file = File::open(prefab_path).unwrap();

        // Copy over the meta file first (if it doesn't exist)
        let mut meta_path = prefab_path.display().to_string();
        meta_path.push_str(".meta");

        let mut contents = String::new();
        let _size = prefab_file.read_to_string(&mut contents);

        let mut converted_contents = contents.clone();

        // Find all occurrences of "guid"
        for indice in contents.match_indices("guid: ") {
            let guid: String = contents.chars().skip(indice.0 + 6).take(32).collect();

            // Check if this has been remapped
            if let Some(meta_file) = remapped_metas.get(&guid) {
                converted_contents.replace_range(indice.0 + 6..indice.0 + 6 + 32, &meta_file.guid);
                continue;
            }

            // Check if this is in our list of missing ones
            // If so copy it
            let mut need_delete = false;
            let mut delete = 0usize;

            for missing_meta in &missing_metas {
                if missing_meta.guid == guid {
                    // Copy the asset (with and without the meta over)
                    // If the file doesn't exist already, copy it
                    let prefab_dir = PathBuf::from(&missing_meta.directory);
                    let mut relative_export_path = PathBuf::from(&export_path);
                    relative_export_path.push(prefab_dir.strip_prefix(&src_assets).unwrap());

                    let export_path = relative_export_path.display().to_string();
                    let _ = create_dir_all(&export_path);

                    let (asset_src_path, meta_src_path) = missing_meta.get_paths();
                    let (asset_dst_path, meta_dst_path) = missing_meta.get_paths_stem(&export_path);

                    if Path::new(&asset_src_path).exists() && !Path::new(&asset_dst_path).exists() {
                        copy(&asset_src_path, &asset_dst_path).unwrap();
                    }

                    if Path::new(&meta_src_path).exists() && !Path::new(&meta_dst_path).exists() {
                        copy(&meta_src_path, &meta_dst_path).unwrap();
                    }

                    // If this is a prefab, push it to the list of queued conversions
                    // If it hasn't been pushed already!
                    for ext in &convert_extensions {
                        if missing_meta.base_name.ends_with(ext.as_str())
                            && !convert_queue.iter().any(|e| e.path == asset_src_path)
                        {
                            println!(
                                "[Conversion]: Enqueuing referenced asset {:?}",
                                asset_src_path
                            );

                            convert_queue.push(AssetConversion {
                                path: asset_src_path.clone(),
                                output_path: relative_export_path.display().to_string(),
                            });

                            break;
                        }
                    }

                    // After being successfully copied, this is removed from the missing list
                    // This prevents prefab duplication / overwriting
                    need_delete = true;
                    break;
                }

                delete += 1;
            }

            if need_delete {
                missing_metas.remove(delete);
            }
        }

        let _ = create_dir_all(&convert.output_path);

        let mut file_path = PathBuf::from(convert.output_path);
        file_path.push(prefab_path.file_name().unwrap());

        write(&file_path, converted_contents).unwrap();

        let mut extension = file_path.extension().unwrap().to_str().unwrap().to_string();
        extension.push_str(".meta");

        file_path.set_extension(extension);
        let _ = copy(meta_path, &file_path);
    }
}

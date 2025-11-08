use proc_macro::TokenStream;
use quote::quote;
use std::collections::BTreeMap;
use std::path::Path;
use syn::{parse_macro_input, LitStr};
use walkdir::WalkDir;

#[derive(Debug)]
struct FolderNode {
    textures: Vec<(String, String)>,
    texture_arrays: Vec<(String, Vec<String>)>,
    j11_tilesets: Vec<(String, String)>,
    subfolders: BTreeMap<String, FolderNode>,
}

impl FolderNode {
    fn new() -> Self {
        FolderNode {
            textures: Vec::new(),
            texture_arrays: Vec::new(),
            j11_tilesets: Vec::new(),
            subfolders: BTreeMap::new(),
        }
    }
}

fn sanitize_ident(name: &str) -> String {
    name.replace('-', "_").replace('.', "_").replace(' ', "_")
}

fn is_j11_tileset(file_stem: &str) -> bool {
    file_stem.ends_with(".j11")
}

fn strip_j11_suffix(file_stem: &str) -> String {
    if file_stem.ends_with(".j11") {
        file_stem[..file_stem.len() - 4].to_string()
    } else {
        file_stem.to_string()
    }
}

fn parse_frame_or_variant(file_stem: &str) -> Option<(String, usize)> {
    // Don't parse j11 tilesets as frame/variant sequences
    if is_j11_tileset(file_stem) {
        return None;
    }

    if let Some(pos) = file_stem.rfind("_f") {
        let base = &file_stem[..pos];
        let num_str = &file_stem[pos + 2..];
        if let Ok(num) = num_str.parse::<usize>() {
            return Some((base.to_string(), num));
        }
    }

    if let Some(pos) = file_stem.rfind("_v") {
        let base = &file_stem[..pos];
        let num_str = &file_stem[pos + 2..];
        if let Ok(num) = num_str.parse::<usize>() {
            return Some((base.to_string(), num));
        }
    }

    None
}

fn build_folder_tree(assets_path: &Path, assets_folder_str: &str) -> FolderNode {
    let mut root = FolderNode::new();

    if !assets_path.exists() {
        return root;
    }

    let mut files_by_dir: BTreeMap<String, Vec<(String, String, String)>> = BTreeMap::new();

    for entry in WalkDir::new(assets_path).min_depth(1) {
        if let Ok(entry) = entry {
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    if ext == "png" || ext == "jpg" || ext == "jpeg" || ext == "bmp" || ext == "gif"
                    {
                        if let Some(file_stem) = path.file_stem() {
                            let file_stem_str = file_stem.to_string_lossy().to_string();

                            let relative_path = path
                                .strip_prefix(assets_path)
                                .unwrap()
                                .to_string_lossy()
                                .to_string();

                            let full_path = format!("{}/{}", assets_folder_str, relative_path);

                            let parent_relative = path
                                .parent()
                                .unwrap()
                                .strip_prefix(assets_path)
                                .unwrap()
                                .to_string_lossy()
                                .to_string();

                            files_by_dir
                                .entry(parent_relative.clone())
                                .or_insert_with(Vec::new)
                                .push((file_stem_str, full_path, parent_relative));
                        }
                    }
                }
            }
        }
    }

    for (_dir_path, files) in files_by_dir {
        let mut grouped: BTreeMap<String, BTreeMap<usize, (String, String)>> = BTreeMap::new();
        let mut singles: Vec<(String, String, String)> = Vec::new();

        for (file_stem, full_path, parent_relative) in files {
            if let Some((basename, index)) = parse_frame_or_variant(&file_stem) {
                grouped
                    .entry(basename)
                    .or_insert_with(BTreeMap::new)
                    .insert(index, (full_path.clone(), parent_relative.clone()));
            } else {
                singles.push((file_stem, full_path, parent_relative));
            }
        }

        for (basename, indexed_files) in grouped {
            let mut sorted_files: Vec<(usize, String, String)> = indexed_files
                .into_iter()
                .map(|(idx, (path, parent))| (idx, path, parent))
                .collect();
            sorted_files.sort_by_key(|(idx, _, _)| *idx);

            if !sorted_files.is_empty() {
                let parent_relative = sorted_files[0].2.clone();
                let field_name = sanitize_ident(&basename);
                let paths: Vec<String> =
                    sorted_files.into_iter().map(|(_, path, _)| path).collect();

                let mut current_node = &mut root;
                if !parent_relative.is_empty() {
                    for component in Path::new(&parent_relative).components() {
                        let folder_name = component.as_os_str().to_string_lossy().to_string();
                        current_node = current_node
                            .subfolders
                            .entry(folder_name)
                            .or_insert_with(FolderNode::new);
                    }
                }

                current_node.texture_arrays.push((field_name, paths));
            }
        }

        for (file_stem, full_path, parent_relative) in singles {
            let is_j11 = is_j11_tileset(&file_stem);
            let field_name = if is_j11 {
                sanitize_ident(&strip_j11_suffix(&file_stem))
            } else {
                sanitize_ident(&file_stem)
            };

            let mut current_node = &mut root;
            if !parent_relative.is_empty() {
                for component in Path::new(&parent_relative).components() {
                    let folder_name = component.as_os_str().to_string_lossy().to_string();
                    current_node = current_node
                        .subfolders
                        .entry(folder_name)
                        .or_insert_with(FolderNode::new);
                }
            }

            if is_j11 {
                current_node.j11_tilesets.push((field_name, full_path));
            } else {
                current_node.textures.push((field_name, full_path));
            }
        }
    }

    root
}

fn generate_struct_for_folder(
    folder_path: &str,
    node: &FolderNode,
    nested_structs: &mut Vec<proc_macro2::TokenStream>,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let struct_name_str = if folder_path.is_empty() {
        "TextureManager".to_string()
    } else {
        format!("{}Manager", sanitize_ident(folder_path))
    };
    let struct_name = syn::Ident::new(&struct_name_str, proc_macro2::Span::call_site());

    let mut field_declarations = Vec::new();
    let mut load_statements = Vec::new();

    let mut sorted_textures = node.textures.clone();
    sorted_textures.sort_by(|a, b| a.0.cmp(&b.0));

    for (field_name, file_path) in &sorted_textures {
        let field_ident = syn::Ident::new(field_name, proc_macro2::Span::call_site());

        field_declarations.push(quote! {
            pub #field_ident: Texture2D
        });

        load_statements.push(quote! {
            #field_ident: rl.load_texture(thread, #file_path)
                .unwrap_or_else(|e| {
                    eprintln!("Warning: Failed to load texture: {} ({}), using fallback texture", #file_path, e);
                    rl.load_texture_from_image(thread, &fallback_image)
                        .expect("Failed to create texture from fallback image")
                })
        });
    }

    let mut sorted_arrays = node.texture_arrays.clone();
    sorted_arrays.sort_by(|a, b| a.0.cmp(&b.0));

    for (field_name, file_paths) in &sorted_arrays {
        let field_ident = syn::Ident::new(field_name, proc_macro2::Span::call_site());

        field_declarations.push(quote! {
            pub #field_ident: Vec<Texture2D>
        });

        let load_statements_for_array: Vec<_> = file_paths.iter().map(|path| {
            quote! {
                rl.load_texture(thread, #path)
                    .unwrap_or_else(|e| {
                        eprintln!("Warning: Failed to load texture: {} ({}), using fallback texture", #path, e);
                        rl.load_texture_from_image(thread, &fallback_image)
                            .expect("Failed to create texture from fallback image")
                    })
            }
        }).collect();

        load_statements.push(quote! {
            #field_ident: vec![#(#load_statements_for_array),*]
        });
    }

    let mut sorted_j11 = node.j11_tilesets.clone();
    sorted_j11.sort_by(|a, b| a.0.cmp(&b.0));

    for (field_name, file_path) in &sorted_j11 {
        let field_ident = syn::Ident::new(field_name, proc_macro2::Span::call_site());

        field_declarations.push(quote! {
            pub #field_ident: J11TileSet
        });

        load_statements.push(quote! {
            #field_ident: J11TileSet::new(
                rl.load_texture(thread, #file_path)
                    .unwrap_or_else(|e| {
                        eprintln!("Warning: Failed to load J11 tileset: {} ({}), using fallback texture", #file_path, e);
                        rl.load_texture_from_image(thread, &fallback_image)
                            .expect("Failed to create texture from fallback image")
                    })
            )
        });
    }

    for (subfolder_name, subfolder_node) in &node.subfolders {
        let new_path = if folder_path.is_empty() {
            subfolder_name.clone()
        } else {
            format!("{}_{}", folder_path, subfolder_name)
        };

        let (subfolder_struct, subfolder_load) =
            generate_struct_for_folder(&new_path, subfolder_node, nested_structs);

        let field_name = syn::Ident::new(
            &sanitize_ident(subfolder_name),
            proc_macro2::Span::call_site(),
        );
        let subfolder_type = syn::Ident::new(
            &format!("{}Manager", sanitize_ident(&new_path)),
            proc_macro2::Span::call_site(),
        );

        field_declarations.push(quote! {
            pub #field_name: #subfolder_type
        });

        load_statements.push(quote! {
            #field_name: #subfolder_load
        });

        nested_structs.push(subfolder_struct);
    }

    let struct_def = quote! {
        pub struct #struct_name {
            #(#field_declarations),*
        }
    };

    let instantiation = quote! {
        #struct_name {
            #(#load_statements),*
        }
    };

    (struct_def, instantiation)
}

#[proc_macro]
pub fn generate_texture_manager(input: TokenStream) -> TokenStream {
    let assets_folder = parse_macro_input!(input as LitStr);
    let assets_folder_str = assets_folder.value();

    let assets_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(&assets_folder_str);

    let root_node = build_folder_tree(&assets_path, &assets_folder_str);

    let fallback_path = format!("{}/fallback.png", assets_folder_str);

    let mut nested_structs = Vec::new();
    let (main_struct, main_load) = generate_struct_for_folder("", &root_node, &mut nested_structs);

    let expanded = quote! {
        use texture_helpers::{J11TileSet, Neighbors};

        #(#nested_structs)*

        #main_struct

        impl TextureManager {
            pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread) -> Self {
                let fallback_image = Image::load_image(#fallback_path)
                    .expect(&format!("Failed to load fallback texture at {} - this is required!", #fallback_path));

                #main_load
            }
        }
    };

    TokenStream::from(expanded)
}

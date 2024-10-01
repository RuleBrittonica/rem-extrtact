//! Utility functions for the rem-extract crate.
//! At some point these will be merged into rem-utils.

use crate::{
    extraction::Cursor,
    error::ExtractionError,
};

use std::{fs, path::{Path, PathBuf}};

use ra_ap_project_model::{
    CargoConfig,
    ProjectWorkspace,
    ProjectManifest,
};

use ra_ap_ide::{
    Analysis,
    AnalysisHost,
    DiagnosticsConfig,
    FilePosition,
    FileRange,
    RootDatabase,
    SingleResolve,
    TextRange,
    TextSize,
    TextEdit,
    SnippetEdit,
    SourceChange,
};

use ra_ap_ide_db::{
    imports::insert_use::{
        ImportGranularity,
        InsertUseConfig,
        PrefixKind,
    }, rename, SnippetCap
};

use ra_ap_ide_assists::{
    Assist,
    AssistConfig,
    AssistKind,
    AssistResolveStrategy,
};

use ra_ap_vfs::{
    AbsPathBuf,
    VfsPath,
    Vfs,
    FileId,
};

use ra_ap_load_cargo::{
    LoadCargoConfig,
    ProcMacroServerChoice,
    load_workspace,
};



// TODO
pub fn cursor_to_offset( cursor: &Cursor ) -> u32 {
   0 as u32
}

/// TODO
/// Returns the path to the manifest directory of the given file
/// The manifest directory is the directory containing the Cargo.toml file
/// for the project.
///
/// ### Example
/// Given a directory structure like:
/// ```plaintext
/// /path/to/project
/// ├── Cargo.toml
/// └── src
///    └── main.rs
/// ```
/// The manifest directory of `main.rs` is `/path/to/project`
pub fn get_manifest_dir( path: &PathBuf ) -> Result<PathBuf, ExtractionError> {
    // Start from the directory of the file
    let mut current_dir = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path.as_path()
    };

    // Check if the current directory is the root and contains a Cargo.toml file
    if fs::metadata(current_dir.join("Cargo.toml")).is_ok() {
        return Ok(current_dir.to_path_buf());
    }

    // Traverse up the directory tree until a Cargo.toml file is found
    while let Some(parent) = current_dir.parent() {
        if fs::metadata(current_dir.join("Cargo.toml")).is_ok() {
            return Ok(current_dir.to_path_buf());
        }
        current_dir = parent;
    }

    // Return an InvalidManifest error if no Cargo.toml file is found
    Err(ExtractionError::InvalidManifest)
}


/// Given a `PathBuf` to a folder, returns the `AbsPathBuf` to the `Cargo.toml`
/// file in that folder.
pub fn get_cargo_toml( manifest_dir: &PathBuf ) -> AbsPathBuf {
    AbsPathBuf::try_from(
        manifest_dir
            .join( "Cargo.toml" )
            .to_str()
            .unwrap()
    ).unwrap()
}

/// Loads as `ProjectManifest` from the given `AbsPathBuf` to a `Cargo.toml` file.
pub fn load_project_manifest( cargo_toml: &AbsPathBuf ) -> ProjectManifest {
    ProjectManifest::from_manifest_file(
        cargo_toml.clone()
    ).unwrap()
}

pub fn progress( _message: String ) -> () {
    // println!( "{}", _message );
}

/// Loads a project workspace from a `ProjectManifest` and `CargoConfig`
pub fn load_project_workspace(
    project_manifest: &ProjectManifest,
    cargo_config: &CargoConfig,
) -> ProjectWorkspace {
    ProjectWorkspace::load(
        project_manifest.clone(),
        cargo_config,
        &progress
    ).unwrap()
}

/// Loads a `RootDatabase` containing from a `ProjectWorkspace` and `CargoConfig`
pub fn load_workspace_data(
    workspace: ProjectWorkspace,
    cargo_config: &CargoConfig,
) -> (
    RootDatabase,
    Vfs
) {
    let load_cargo_config: LoadCargoConfig = LoadCargoConfig {
        load_out_dirs_from_check: true,
        with_proc_macro_server: ProcMacroServerChoice::None,
        prefill_caches: false,
    };

    let (db,
        vfs,
        _proc_macro
    ) = load_workspace(
        workspace,
        &cargo_config.extra_env,
        &load_cargo_config
    ).unwrap();

    (db, vfs)
}

/// Runs the analysis on an AnalysisHost. A wrapper around `AnalysisHost::analysis`
fn run_analysis( host: AnalysisHost ) -> Analysis {

    let analysis: Analysis = host.analysis();

    analysis
}

/// Gets a list of available assists for a given file and range
fn get_assists (
    analysis: &Analysis,
    manifest_dir: &PathBuf,
    vfs: &Vfs,
    path_components: &Vec<&str>, // Vec of path components, e.g. [ "src", "main.rs" ]
    range: (u32, u32), // Tuple of start and end offsets
) -> Vec<Assist> {

    // Build out the AssistConfig
    let snippet_cap_: Option<SnippetCap> = None;
    let allowed_assists: Vec<AssistKind> = vec![
        AssistKind::RefactorExtract,
    ];

    let insert_use_: InsertUseConfig = InsertUseConfig {
        granularity: ImportGranularity::Preserve,
        enforce_granularity: false,
        prefix_kind: PrefixKind::ByCrate,
        group: false,
        skip_glob_imports: false,
    };

    let assist_config: AssistConfig = AssistConfig {
        snippet_cap: snippet_cap_,
        allowed: Some(allowed_assists),
        insert_use: insert_use_,
        prefer_no_std: false,
        prefer_prelude: false,
        prefer_absolute: false,
        assist_emit_must_use: false,
        term_search_fuel: 2048, // * NFI what this is
        term_search_borrowck: false,
    };

    // Build out the DiagnosticsConfig
    let diagnostics_config: DiagnosticsConfig = DiagnosticsConfig::test_sample(); // TODO This may need to be specific to the program

    // Build out the ResolveStrategy
    // FIXME: This is currently bugged it seems - Both extract_variable and extract_function are being returned
    let resolve: AssistResolveStrategy = AssistResolveStrategy::Single(
        SingleResolve {
            assist_id: "extract_function".to_string(),
            assist_kind: AssistKind::RefactorExtract,
        }
    );

    // Build out the FileRange
    let mut file_path: PathBuf = manifest_dir.clone();
    for component in path_components {
        file_path = file_path.join( component );
    }
    let vfs_path: VfsPath = VfsPath::new_real_path(
        file_path
            .to_string_lossy()
            .to_string(),
    );

    let file_id_: FileId = vfs.file_id( &vfs_path ).unwrap();
    let range_: TextRange = TextRange::new(
        TextSize::try_from( range.0 ).unwrap(),
        TextSize::try_from( range.1 ).unwrap(),
    );

    let frange: FileRange = FileRange {
        file_id: file_id_,
        range: range_,
    };

    // Call the assists_with_fixes method
    let assists: Vec<Assist> = analysis.assists_with_fixes(
        &assist_config,
        &diagnostics_config,
        resolve,
        frange
    ).unwrap();

    assists
}

/// Filter the list of assists to only be the extract_function assist
/// FIXME This is a hack to get around the fact that the resolve strategy is bugged
/// and is returning both extract_variable and extract_function
pub fn filter_extract_function_assist( assists: Vec<Assist> ) -> Assist {
    let extract_assist = assists
        .iter()
        .find( |assist| assist.label == "Extract into function" )
        .unwrap()
        .clone();

    extract_assist
}

pub fn apply_extract_function(
    assist: &Assist,
    manifest_dir: &PathBuf,
    vfs: &Vfs,
    path_components: &Vec<&str>, // Vec of path components, e.g. [ "src", "main.rs" ]
    output_path: &AbsPathBuf,
    callee_name: &str,
) -> PathBuf {
    // Copy the source file to the output directory
    let mut in_file_path: PathBuf = manifest_dir.clone();
    for component in path_components {
        in_file_path = in_file_path.join( component );
    }
    let vfs_in_path: VfsPath = VfsPath::new_real_path(
        in_file_path
            .to_string_lossy()
            .to_string(),
    );

    let vfs_out_path: VfsPath = VfsPath::new_real_path(
        output_path
            .as_str()
            .to_string(),
    );

    copy_file_vfs( &vfs_in_path, &vfs_out_path );

    // From here, extract the source change, but apply it to the copied file
    let src_change: SourceChange = assist.source_change
        .as_ref()
        .unwrap()
        .clone();
    let in_file_id: FileId = vfs.file_id( &vfs_in_path ).unwrap();
    let (text_edit, maybe_snippet_edit) = src_change.get_source_and_snippet_edit(
        in_file_id
    ).unwrap();
    let text_edit: TextEdit = text_edit.clone();
    let maybe_snippet_edit: Option<SnippetEdit> = maybe_snippet_edit.clone();

    apply_edits(
        &vfs_out_path,
        text_edit,
        maybe_snippet_edit,
    );

    // Rename the function from fun_name to NEW_FUNCTION_NAME using a search and
    // replace on the output file
    rename_function(
        &vfs_out_path,
        "fun_name",
        callee_name,
    );

    // Return the output file path
    PathBuf::from( vfs_out_path.to_string() )
}

// Apply a text edit.
// Then apply the snippet edit if it is present
fn apply_edits(
    vfs_path: &VfsPath,
    text_edit: TextEdit,
    maybe_snippet_edit: Option<SnippetEdit>,
) -> () {
    let path: PathBuf = vfs_to_pathbuf( vfs_path );
    let mut text: String = std::fs::read_to_string( &path ).unwrap();
    text_edit.apply( &mut text );
    match maybe_snippet_edit {
        Some( snippet_edit ) => {
            snippet_edit.apply( &mut text );
        },
        None => (),
    }
    std::fs::write( &path, text ).unwrap();
}

// Rename a function in a file using a search and replace
fn rename_function(
    vfs_path: &VfsPath,
    old_name: &str,
    new_name: &str,
) -> () {
    let path: PathBuf = vfs_to_pathbuf( vfs_path );
    let mut text: String = std::fs::read_to_string( &path ).unwrap();
    let old_name: String = old_name.to_string();
    let new_name: String = new_name.to_string();
    text = text.replace( &old_name, &new_name );
    std::fs::write( &path, text ).unwrap();
}

/// Converts a `VfsPath` to a `PathBuf`
fn vfs_to_pathbuf( vfs_path: &VfsPath ) -> PathBuf {
    let path_str = vfs_path.to_string();
    // println!("{}", path_str);
    PathBuf::from( path_str )
}

/// Copies a file from one `VfsPath` to another `VfsPath`
fn copy_file_vfs(
    source: &VfsPath,
    destination: &VfsPath,
) -> () {
    // Copy the file
    let from: PathBuf = vfs_to_pathbuf( source );
    let to: PathBuf = vfs_to_pathbuf( destination );
    let _ = std::fs::copy(from, to).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::env;

    // Helper function to create a temporary directory with a Cargo.toml
    fn setup_temp_project() -> PathBuf {
        let temp_dir = env::temp_dir().join("test_project");
        let _ = fs::create_dir_all(&temp_dir);
        let cargo_toml = temp_dir.join("Cargo.toml");

        let mut file = File::create(cargo_toml).unwrap();
        writeln!(file, "[package]\nname = \"test_project\"\nversion = \"0.1.0\"").unwrap();

        temp_dir
    }

    // Test case when Cargo.toml exists
    #[test]
    fn test_get_manifest_dir_valid() {
        let temp_dir = setup_temp_project();
        let src_dir = temp_dir.join("src");
        let _ = fs::create_dir_all(&src_dir);
        let main_file = src_dir.join("main.rs");
        File::create(&main_file).unwrap();

        let result = get_manifest_dir(&main_file);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir);
    }

    // Test case when Cargo.toml does not exist
    #[test]
    fn test_get_manifest_dir_invalid_manifest() {
        let temp_dir = env::temp_dir().join("test_invalid_project");
        let _ = fs::create_dir_all(&temp_dir);
        let src_dir = temp_dir.join("src");
        let _ = fs::create_dir_all(&src_dir);
        let main_file = src_dir.join("main.rs");
        File::create(&main_file).unwrap();

        let result = get_manifest_dir(&main_file);
        assert!(result.is_err());

        // Check that the error is an InvalidManifest
        if let ExtractionError::InvalidManifest = result.unwrap_err() {
            // Correct error type
        } else {
            panic!("Expected InvalidManifest error");
        }
    }

    // Test case when the path is to a directory, not a file
    #[test]
    fn test_get_manifest_dir_directory() {
        let temp_dir = setup_temp_project();
        let src_dir = temp_dir.join("src");
        let _ = fs::create_dir_all(&src_dir);

        let result = get_manifest_dir(&src_dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir);
    }

    // Test case when the path points to a non-existent file
    #[test]
    fn test_get_manifest_dir_non_existent_file() {
        let temp_dir = env::temp_dir().join("test_non_existent_project");
        let src_dir = temp_dir.join("src");

        let non_existent_file = src_dir.join("does_not_exist.rs");
        let result = get_manifest_dir(&non_existent_file);
        assert!(result.is_err());

        // Check that the error is an InvalidManifest
        if let ExtractionError::InvalidManifest = result.unwrap_err() {
            // Correct error type
        } else {
            panic!("Expected InvalidManifest error");
        }
    }
}

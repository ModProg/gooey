use devx_cmd::run;
use fs_extra::dir::CopyOptions;
use khonsu_tools::{anyhow, code_coverage::CodeCoverage};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Args {
    BuildBrowserExample {
        name: Option<String>,
    },
    GenerateCodeCoverageReport {
        #[structopt(long = "install-dependencies")]
        install_dependencies: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::from_args();
    match args {
        Args::BuildBrowserExample { name } => {
            build_browser_example(name.unwrap_or_else(|| String::from("basic")))?
        }
        Args::GenerateCodeCoverageReport {
            install_dependencies,
        } => CodeCoverage::<CodeCoverageConfig>::execute(install_dependencies)?,
    };
    Ok(())
}

fn build_browser_example(name: String) -> Result<(), anyhow::Error> {
    let (index_path, browser_path) = match name.as_str() {
        "bonsaidb-counter-client" => {
            run!(
                "cargo",
                "build",
                "--package",
                "bonsaidb-counter-client",
                "--target",
                "wasm32-unknown-unknown",
                "--target-dir",
                "target/wasm",
            )?;
            execute_wasm_bindgen(
                "target/wasm/wasm32-unknown-unknown/debug/bonsaidb-counter-client.wasm",
                "integrated-examples/bonsaidb/counter/browser/pkg/",
            )?;

            (
                String::from("index.html"),
                String::from("integrated-examples/bonsaidb/counter/browser"),
            )
        }
        regular_example => {
            build_regular_browser_example(regular_example)?;
            execute_wasm_bindgen(
                &format!(
                    "target/wasm/wasm32-unknown-unknown/debug/examples/{}.wasm",
                    regular_example
                ),
                "gooey/examples/browser/pkg/",
            )?;

            (
                format!("index.html?{}", regular_example),
                "gooey/examples/browser/".to_owned(),
            )
        }
    };

    println!(
        "Build succeeded. .{}/{} can be loaded through any http server that supports wasm.",
        browser_path, index_path,
    );
    println!();
    println!("For example, using `miniserve` (`cargo install miniserve`):");
    println!();
    println!("miniserve {}", browser_path);
    println!();
    println!("Then, navigate to: http://localhost:8080/{}", index_path);

    Ok(())
}

fn build_regular_browser_example(name: &str) -> Result<(), anyhow::Error> {
    println!("Executing cargo build");
    run!(
        "cargo",
        "build",
        "--example",
        name,
        "--no-default-features",
        "--features",
        "frontend-browser",
        "--target",
        "wasm32-unknown-unknown",
        "--target-dir",
        "target/wasm",
    )?;

    fs_extra::copy_items(
        &["gooey/assets"],
        &"gooey/examples/browser/assets",
        &CopyOptions {
            skip_exist: true,
            copy_inside: true,
            ..CopyOptions::default()
        },
    )?;

    Ok(())
}

fn execute_wasm_bindgen(wasm_path: &str, out_path: &str) -> Result<(), devx_cmd::Error> {
    println!("Executing wasm-bindgen (cargo install wasm-bindgen if you don't have this)");
    run!(
        "wasm-bindgen",
        wasm_path,
        "--target",
        "web",
        "--out-dir",
        out_path,
        "--remove-producers-section"
    )
}

struct CodeCoverageConfig;

impl khonsu_tools::code_coverage::Config for CodeCoverageConfig {
    fn ignore_paths() -> Vec<String> {
        vec![String::from("gooey/examples/*")]
    }
}

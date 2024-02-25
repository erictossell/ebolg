use pulldown_cmark::{html, Options, Parser};
use serde::Deserialize;
use serde_yaml::{self};
use std::env;
use std::path::{Path, PathBuf};
use std::{error::Error, fs};

#[derive(Debug, Deserialize, Clone)]
struct Metadata {
    title: String,
}

fn read_post_metadata(file_path: &Path) -> Result<(Metadata, String), Box<dyn Error>> {
    let content = fs::read_to_string(file_path)?;

    let (yaml_str, content_str) = extract_yaml_and_content(&content)?;

    let metadata: Metadata = serde_yaml::from_str(&yaml_str)?;
    println!("\n-------------------");
    println!("Metadata: {:?}", metadata);
    println!("Content snippet: {}", &content[..content.len().min(100)]);

    Ok((metadata, content_str))
}

fn extract_yaml_and_content(content: &str) -> Result<(String, String), Box<dyn Error>> {
    // Splitting the content based on the starting and ending triple-dashed lines of YAML front matter
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return Err("Failed to split content".into());
    }
    let yaml_str = parts[1].trim(); // The YAML content should be the second part
    let content_str = parts[2].trim(); // The remaining content starts after the second triple-dashed line

    Ok((yaml_str.to_string(), content_str.to_string()))
}

fn add_tailwind_classes(html_content: &str) -> String {
    html_content
        .replace("<h1>", r#"<h1 class="text-3xl font-bold">"#)
        .replace("<h2>", r#"<h2 class="text-2xl font-bold mb-2">"#)
        .replace("<p>", r#"<p class="text-gray-400 mb-4">"#)
        .replace(
            "<pre>",
            r#"<pre class="bg-gray-700 text-green-300 p-4 rounded mb-4 overflow-x-auto">"#,
        )
        .replace("<code>", r#"<code class="inline-block">"#)
        .replace(
            "<code class=\"inline-block\">",
            r#"<code class="inline-block bg-gray-700 text-green-300 p-1 rounded">"#,
        )
}

fn generate_html_header(
    title: &str,
    prev_post: Option<&Metadata>,
    next_post: Option<&Metadata>,
) -> String {
    let prev_button = prev_post.map_or(String::new(), |_| String::from(r#"<button class="bg-green-500 hover:bg-green-600 text-white font-bold py-2 px-4 rounded"><span>&larr; Back</span></button>"#));
    let next_button = next_post.map_or(String::new(), |_| String::from(r#"<button class="bg-green-500 hover:bg-green-600 text-white font-bold py-2 px-4 rounded"><span>Next &rarr;</span></button>"#));

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <link rel="stylesheet" href="/style/tailwind.css">   
</head>
<body class="bg-gray-800 text-white">
    <div class="container mx-auto px-4 py-8">
        <div class="flex justify-between items-center mb-6">
            {prev_button}
            <h1 class="text-3xl font-bold">{title}</h1>
            {next_button}
        </div>
        <article>
"#,
        title = title,
        prev_button = prev_button,
        next_button = next_button
    )
}

fn generate_html_footer() -> &'static str {
    r#"</article>
    </div>

</body>

</html>"#
}

fn convert_markdown_to_html(
    html_path: &Path,
    metadata: &Metadata,
    markdown_content: &str,
    prev_post: Option<&Metadata>,
    next_post: Option<&Metadata>,
) -> Result<(), Box<dyn Error>> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(markdown_content, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    let styled_html_content = add_tailwind_classes(&html_output);

    let header = generate_html_header(&metadata.title, prev_post, next_post);
    let footer = generate_html_footer();
    let complete_html = format!("{}{}{}", header, styled_html_content, footer);
    println!("HTML content length: {}", complete_html.len());
    println!("HTML file generated: {:?}", html_path);

    if let Err(e) = fs::write(html_path, complete_html) {
        eprintln!("Failed to write HTML to file: {}", e);
    }

    Ok(())
}

fn process_directory(dir_path: &Path, output_dir: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(output_dir)?;

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let sub_output_dir = output_dir.join(entry.file_name());
            process_directory(&path, &sub_output_dir)?;
        } else {
            match path.extension().and_then(std::ffi::OsStr::to_str) {
                Some("md") => {
                    let (metadata, content) = read_post_metadata(&path)?;
                    let file_stem = path.file_stem().unwrap().to_str().unwrap();
                    let html_file_name = PathBuf::from(format!("{}.html", file_stem));
                    let html_path = output_dir.join(html_file_name);

                    convert_markdown_to_html(&html_path, &metadata, &content, None, None)?;
                }
                Some("css") => {
                    let target_path = output_dir.join(path.file_name().unwrap());
                    fs::copy(&path, &target_path)?;
                }
                _ => {} // Ignore other file types
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <SOURCE> <OUTPUT DIRECTORY>", args[0]);
        return Ok(());
    }

    let source_path = Path::new(&args[1]);
    let output_dir = Path::new(&args[2]);

    fs::create_dir_all(&output_dir)?;

    if source_path.is_dir() {
        process_directory(source_path, &output_dir)?;
    } else if source_path.is_file() {
        if source_path.extension().and_then(std::ffi::OsStr::to_str) == Some("md") {
            let file_name = source_path.file_name().unwrap().to_str().unwrap();
            let target_file = output_dir.join(file_name).with_extension("html");
            let (metadata, content) = read_post_metadata(&source_path)?;
            convert_markdown_to_html(&target_file, &metadata, &content, None, None)?;
        }
    } else {
        eprintln!("The path specified does not exist or is not a file/directory.");
        return Ok(());
    }

    Ok(())
}

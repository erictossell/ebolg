use chrono::NaiveDate;
use pulldown_cmark::{html, Options, Parser};
use serde::Deserialize;
use serde_yaml::{self};
use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};
use std::{error::Error, fs};

#[derive(Debug, Deserialize, Clone)]
struct Metadata {
    title: String,
    date: NaiveDate,
}

fn read_post_metadata(file_path: &Path) -> Result<(Metadata, String), Box<dyn Error>> {
    // Simulate extracting YAML as a string and the remainder of the content
    let content = fs::read_to_string(file_path)?;

    // Assume you have a function to extract YAML string and content correctly
    let (yaml_str, content_str) = extract_yaml_and_content(&content)?;

    let metadata: Metadata = serde_yaml::from_str(&yaml_str)?;
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
    // Ensure to close the tags with the same classes if necessary
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
    <script src="https://cdn.tailwindcss.com"></script>
    <style>
        /* Additional styles can be added here if needed */
    </style>
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
    html_path: &Path, // Changed from file_path to html_path for clarity
    metadata: &Metadata,
    markdown_content: &str,
    prev_post: Option<&Metadata>,
    next_post: Option<&Metadata>,
) -> Result<(), Box<dyn Error>> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(markdown_content, options);

    // Convert Markdown to HTML
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    // Apply Tailwind CSS classes to the converted HTML
    let styled_html_content = add_tailwind_classes(&html_output);

    // Generate the complete HTML with header, styled content, and footer
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
    fs::create_dir_all(output_dir)?; // Ensure the output directory exists

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // If it's a directory, recursively process it
            let sub_output_dir = output_dir.join(entry.file_name());
            process_directory(&path, &sub_output_dir)?;
        } else if path.is_file() && path.extension().and_then(std::ffi::OsStr::to_str) == Some("md")
        {
            // Process Markdown files
            let (metadata, content) = read_post_metadata(&path)?;
            let file_stem = path.file_stem().unwrap().to_str().unwrap();
            let html_file_name = PathBuf::from(format!("{}.html", file_stem));
            let html_path = output_dir.join(html_file_name);

            convert_markdown_to_html(&html_path, &metadata, &content, None, None)?;
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <FILE or DIRECTORY>", args[0]);
        return Ok(());
    }

    let path = Path::new(&args[1]);

    // Check if the path is a file or directory and process accordingly
    if path.is_dir() {
        let output_dir = path.parent().unwrap_or_else(|| Path::new("")).join("dist"); // Output directory for processed HTML files
        process_directory(path, &output_dir)?;
    } else if path.is_file() {
        // Process a single file
        // Determine the output directory based on the file's parent (if available)
        let output_dir = path.parent().unwrap_or_else(|| Path::new("")).join("dist");
        fs::create_dir_all(&output_dir)?;

        // Process the individual file
        let (metadata, content) = read_post_metadata(path)?;
        let file_stem = path.file_stem().unwrap().to_str().unwrap();
        let html_file_name = PathBuf::from(format!("{}.html", file_stem));
        let html_path = output_dir.join(html_file_name);

        convert_markdown_to_html(&html_path, &metadata, &content, None, None)?;
    } else {
        eprintln!("The path specified does not exist or is not a file/directory.");
    }

    Ok(())
}

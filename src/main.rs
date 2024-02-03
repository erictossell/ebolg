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
    file_path: &Path,
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

    // Save the combined HTML to a new file
    let new_file_name = file_path.with_extension("html");
    fs::write(new_file_name, complete_html)?;
    Ok(())
}

fn process_directory(dir_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut posts: BTreeMap<NaiveDate, (PathBuf, Metadata, String)> = BTreeMap::new();

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(std::ffi::OsStr::to_str) == Some("md") {
            let (metadata, content) = match read_post_metadata(&path) {
                Ok(data) => data,
                Err(_) => continue, // Skip files without valid metadata
            };
            posts.insert(metadata.date, (path, metadata, content));
        }
    }

    let dates: Vec<_> = posts.keys().cloned().collect();
    for (i, date) in dates.iter().enumerate() {
        let (path, metadata, content) = &posts[date];
        let prev_post = dates
            .get(i.wrapping_sub(1))
            .and_then(|date| posts.get(date).map(|(_, meta, _)| meta));
        let next_post = dates
            .get(i + 1)
            .and_then(|date| posts.get(date).map(|(_, meta, _)| meta));
        convert_markdown_to_html(path, metadata, content, prev_post, next_post)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <FILE or DIRECTORY>", args[0]);
        std::process::exit(1);
    }
    let path = Path::new(&args[1]);
    println!("Attempting to access path: {:?}", path); // Debugging line

    if path.exists() {
        if path.is_dir() {
            process_directory(path)?;
        } else if path.is_file() {
            // Assuming you want to process a single file if it's not a directory
            let (metadata, content) = read_post_metadata(&path)?;
            convert_markdown_to_html(&path, &metadata, &content, None, None)?;
        } else {
            eprintln!("The path specified is neither a file nor a directory.");
            std::process::exit(1);
        }
    } else {
        eprintln!("The path specified does not exist or is not accessible.");
        std::process::exit(1);
    }

    Ok(())
}

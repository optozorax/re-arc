use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
struct Puzzle {
    input: Vec<Vec<i64>>,
    output: Vec<Vec<i64>>,
}

// Reuse the same CSS template
const CELL_SIZE: u32 = 10;
const CSS_TEMPLATE: &str = r#"
@import url("https://fonts.googleapis.com/css2?family=Anonymous+Pro:ital,wght@0,400;0,700;1,400;1,700");

@font-face {
    font-family: 'AtariClassicChunky';
    src: url('https://arcprize.org/media/fonts/AtariClassicChunky.eot');
    src: url('https://arcprize.org/media/fonts/AtariClassicChunky.eot?#iefix') format('embedded-opentype'),
        url('https://arcprize.org/media/fonts/AtariClassicChunky.woff2') format('woff2'),
        url('https://arcprize.org/media/fonts/AtariClassicChunky.woff') format('woff'),
        url('https://arcprize.org/media/fonts/AtariClassicChunky.svg#AtariClassicChunky') format('svg');
    font-weight: normal;
    font-style: normal;
    font-display: swap;
}
:root {
    --white: #EEEEEE;
    --offwhite: #C0C0C0;
    --black: #000000;
    --magenta: #E53AA3;
    --magenta-light: #ff7bcc;
    --red: #F93C31;
    --blue: #1E93FF;
    --blue-light: #87D8F1;
    --yellow: #FFDC00;
    --orange: #FF851B;
    --maroon: #921231;
    --green: #4FCC30;
    --gray: #555555;
    --gray-light: #999999;
}

body {
    background-color: var(--black);
    color: var(--white);
    font-family: 'Anonymous Pro', monospace;
    display: flex;
    flex-direction: column;
    align-items: center;
    margin: 0;
    padding: 20px;
}

h1 {
    font-family: 'AtariClassicChunky', monospace;
    color: var(--magenta);
    margin-bottom: 30px;
}

h3 {
    word-break: break-all;
    word-wrap: anywhere;
    white-space: normal;
    height: 35pt;
    margin: 0px;
}

.task-container {
    display: flex;
    flex-wrap: wrap;
    gap: 20px;
    width: 100%;
    max-width: 2200px;
    justify-content: center;
}

.task {
    flex: 0 1 auto;
    min-width: 200px;

    background-color: var(--black);
    padding: 10px;

    border: 0.5px solid var(--gray);
}

.subtask {
    flex: 0 1 auto;
    min-width: 200px;
    // max-width: 400px;

    background-color: var(--black);
    padding: 10px;
}

.task-title {
    color: var(--offwhite);
    margin-bottom: 3px;
    font-size: 14px;
}

.grid-container {
    display: flex;
    flex-direction: column;
    gap: 3px;
}

.grid {
    display: grid;
}

.cell {
    width: 10px;
    height: 10px;
    border: 0.2px solid var(--gray);
}

.color-0 { background-color: var(--black); }
.color-1 { background-color: var(--blue); }
.color-2 { background-color: var(--red); }
.color-3 { background-color: var(--green); }
.color-4 { background-color: var(--yellow); }
.color-5 { background-color: var(--gray-light); }
.color-6 { background-color: var(--magenta); }
.color-7 { background-color: var(--orange); }
.color-8 { background-color: var(--blue-light); }
.color-9 { background-color: var(--maroon); }

a {
    color: var(--blue);
    text-decoration: none;
}

a:hover {
    color: var(--blue-light);
}

p {
    margin: 0px;
}

.invariants-container {
    margin-top: 10px;
    padding: 10px;
    border: 1px solid #ddd;
    border-radius: 4px;
}

.invariant {
    display: inline-block;
    margin: 2px 5px;
    padding: 2px 6px;
    background-color: #262626;
    border-radius: 3px;
    font-size: 0.9em;
}

.palette-container {
    margin-top: 10px;
}

.palette {
    display: flex;
    gap: 2px;
    margin: 5px 0;
}
"#;

fn create_2d_grid_html(data: &[Vec<i64>]) -> String {
    let columns = data[0].len();

    let mut grid_html = format!(
        r#"<div class="grid" style="grid-template-columns: repeat({}, {}px);">"#,
        columns, CELL_SIZE
    );

    for row in data {
        for &cell in row {
            grid_html.push_str(&format!(r#"<div class="cell color-{}"></div>"#, cell));
        }
    }
    grid_html.push_str("</div>");
    grid_html
}

fn get_size_string(data: &[Vec<i64>]) -> String {
    format!("{}×{}", data.len(), data[0].len())
}

fn create_task_html(puzzle: &Puzzle, task_name: &str) -> String {
    format!(
        r#"
        <div class="subtask">
            <div class="task-title">{} ({} → {})</div>
            <div class="grid-container" style="flex-direction: column; gap: 20px;">
                {}
                {}
            </div>
        </div>
        "#,
        task_name,
        get_size_string(&puzzle.input),
        get_size_string(&puzzle.output),
        create_2d_grid_html(&puzzle.input),
        create_2d_grid_html(&puzzle.output)
    )
}

fn generate_single_task_page(task_path: &Path, output_dir: &Path) -> std::io::Result<PathBuf> {
    let task_name = task_path.file_name().unwrap().to_string_lossy();
    let content = fs::read_to_string(&task_path)?;
    let puzzle_sets: Vec<Puzzle> = serde_json::from_str(&content)?;
    let task_name = task_name.strip_suffix(".json").unwrap();

    let mut task_html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>RE-ARC {}</title>
            <style>{}</style>
        </head>
        <body>
            <a href="index.html">go back to all tasks</a>
            <h1>{} (<a href="https://arcprize.org/play?task={}">original</a>)</h1>
            <h3>({} examples)</h3>
            <div class="task-container">
        "#,
        task_name,
        CSS_TEMPLATE,
        task_name,
        task_name,
        puzzle_sets.len()
    );

    // Create task HTML for each puzzle in the set
    for (j, puzzle) in puzzle_sets.iter().enumerate() {
        task_html.push_str(&create_task_html(puzzle, &format!("{}", j)));
    }

    task_html.push_str(
        r#"
            </div>
        </body>
        </html>
        "#,
    );

    let output_path = output_dir.join(format!("{}.html", task_name));
    let mut file = File::create(&output_path)?;
    file.write_all(task_html.as_bytes())?;

    Ok(output_path)
}

fn generate_index_page(tasks_dir: &Path, output_dir: &Path) -> std::io::Result<()> {
    let mut index_html = format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>RE-ARC dataset visualization</title>
            <style>{}</style>
        </head>
        <body>
            <h1><a href="https://github.com/michaelhodel/re-arc/">RE-ARC</a> dataset visualization</h1>
            <div class="task-container">
        "#,
        CSS_TEMPLATE
    );

    for entry in fs::read_dir(tasks_dir)? {
        let entry = entry?;
        let path = entry.path();
        let task_name = path.file_name().unwrap().to_string_lossy();
        let content = fs::read_to_string(&path)?;
        let puzzle_sets: Vec<Puzzle> = serde_json::from_str(&content)?;

        if !puzzle_sets.is_empty() {
            index_html.push_str(&format!(
                r#"<div class="task"><h3><a href="{task_name}.html">{task_name}</a> (<a href="https://arcprize.org/play?task={task_name}">original</a>)</h3>"#,
                task_name=task_name.strip_suffix(".json").unwrap()
            ));

            index_html.push_str(&format!(
                r#"<center><p>({} examples)</p></center>"#,
                puzzle_sets.len()
            ));

            // Show only first puzzle from first puzzle set
            if let Some(first_puzzle) = puzzle_sets.first() {
                index_html.push_str(&create_task_html(first_puzzle, "example"));
            }

            index_html.push_str("</div>");
        }
    }

    index_html.push_str(
        r#"
            </div>
        </body>
        </html>
        "#,
    );

    let mut file = File::create(output_dir.join("index.html"))?;
    file.write_all(index_html.as_bytes())?;
    Ok(())
}

fn draw() -> std::io::Result<()> {
    let tasks_dir = Path::new("tasks");
    let output_dir = Path::new("visualization");

    fs::create_dir_all(output_dir)?;

    for entry in fs::read_dir(tasks_dir)? {
        let entry = entry?;
        let path = entry.path();
        generate_single_task_page(&path, output_dir)?;
    }

    generate_index_page(tasks_dir, output_dir)?;
    Ok(())
}

fn main() {
    color_backtrace::install();

    draw().unwrap();
}

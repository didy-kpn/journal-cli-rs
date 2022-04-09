use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{create_dir_all, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "journal-cli")]
struct Opt {
    #[structopt(subcommand)]
    sub: Sub,
}

#[derive(StructOpt, Debug)]
enum Sub {
    #[structopt(name = "new", about = "Create a new journal directory")]
    New(DirectoryName),

    #[structopt(name = "add", about = "Add a new page")]
    Add(PageKind),
}

#[derive(StructOpt, Debug)]
struct DirectoryName {
    #[structopt(parse(from_os_str), about = "directory name")]
    name: PathBuf
}

#[derive(StructOpt, Debug)]
enum PageKind {
    #[structopt(name = "entry", about = "Add today's entry")]
    Entry,

    #[structopt(name = "article", about = "Add new article")]
    Article,
}

#[derive(Debug, Serialize, Deserialize)]
struct JournalSetting {
    entry_path: String,
    article_path: String,
    journal_template: String,
}

fn main() {
    let opt = Opt::from_args();
    let today = Utc::today();
    let setting_file_name = "journal-cli.yaml";

    match &opt.sub {
        // ページを追加
        Sub::Add(add) => {
            // 設定ファイルを読み込む
            let file = OpenOptions::new().read(true).open(setting_file_name);
            if let Err(err) = file {
                eprintln!("journal-cli.yaml: {}", err);
                return;
            }
            let file = file.unwrap();
            let mut f = BufReader::new(file);
            let mut content = String::new();
            f.read_to_string(&mut content).unwrap();
            let setting: JournalSetting = serde_yaml::from_str(&content).unwrap();

            match add {
                // 本日の日記
                PageKind::Entry => {
                    let dir_path =
                        &format!("{}/{}/{}", setting.entry_path, today.year(), today.month());
                    if !Path::new(dir_path).exists() {
                        if let Err(error) = create_dir_all(dir_path) {
                            eprintln!("create_dir_all: {}", error);
                            return;
                        }
                    }

                    let file_name =
                        format!("{}_{}_{}.md", today.year(), today.month(), today.day());
                    let content = setting
                        .journal_template
                        .replace("{}", &today.format("%Y/%m/%d").to_string());
                    write_file(dir_path, &file_name, &content);
                }
                // まとめなどの記事
                PageKind::Article => {}
            }
        }
        // 日記を作成する
        Sub::New(directory_name) => {
            let directory_path = &format!("{}/{}", env::current_dir().unwrap().display(), directory_name.name.as_path().display());

            // 日記ディレクトリまたは設定ファイルが存在してる場合はディレクトリを作成しない
            if Path::new(directory_path).exists() || Path::new(setting_file_name).exists() {
                eprintln!("not ok: {} or journal-cli.yaml exists", directory_name.name.as_path().display());
                return;
            }

            if let Err(error) = create_dir_all(directory_path.clone()) {
                eprintln!("create_dir_all: {}", error);
                return;
            } else {
                println!("ok: journal directory");
            }

            // 各記事へのリンク(README)
            write_file(directory_path, "README.md", "# README \n");

            // 今後のやるべきことリスト(TODO)
            write_file(directory_path, "TODO.md", "# TODO (やるべきこと)\n");

            // 実績(CHANGELOG)
            write_file(directory_path, "CHANGELOG.md", "# CHANGELOG (実績)\n");

            // ガイドライン(CONTRIBUTING)
            write_file(directory_path, "CONTRIBUTING.md", "# CONTRIBUTING (ガイドライン)\n");

            // 設定ファイル
            let journal_template = r#"# {}

## Concrete Experience (具体的経験)

## Reflective Observation (省察)

## Abstract Conceptualization (概念化):

## Active Experimentation (試行):

"#;
            let setting = JournalSetting {
                entry_path: format!(
                    "{}/entries",
                    directory_path
                ),
                article_path: format!(
                    "{}/articles",
                    directory_path
                ),
                journal_template: journal_template.to_string(),
            };
            write_file(
                directory_path,
                setting_file_name,
                &serde_yaml::to_string(&setting).unwrap(),
            );
        }
    };
}

fn write_file(directory_path: &str, file_name: &str, content: &str) {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(format!("{}/{}", directory_path, file_name))
        .unwrap();

    let mut f = BufWriter::new(file);
    if let Err(error) = f.write(content.as_bytes()) {
        eprintln!("f.write(content): {}", error);
    } else {
        println!("ok: {}/{} file", directory_path, file_name);
    }
}

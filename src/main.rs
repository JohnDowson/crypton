#![allow(unused_braces)]
//this is because #[clap(subcommand)] produces this warning
mod tags;
mod utils;
use clap::{AppSettings, Args, Clap};
use sqlx::sqlite::SqlitePool;
use std::env;
use std::path::PathBuf;
use tags::{tags, TagsSubcommand};
use utils::*;

#[derive(Clap, Debug)]
#[clap(version = "0.1.0", author = "Ivan C. <ichinenov@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
#[clap(setting = AppSettings::InferSubcommands)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand {
    List {
        tag: Option<String>,
    },
    Tags {
        #[clap(subcommand)]
        subcommand: Option<TagsSubcommand>,
    },
    New(Note),
    Remove(Hash),
    Edit(Hash),
}

#[derive(Args, Debug)]
struct Note {
    note: Vec<String>,
    #[clap(short, long)]
    tag: Option<String>,
}

impl From<Note> for String {
    fn from(val: Note) -> Self {
        val.note
            .into_iter()
            .map(|mut s| {
                s.push(' ');
                s
            })
            .collect::<String>()
    }
}

#[derive(Args, Debug)]
struct Hash {
    arg: String,
}

#[tokio::main]
async fn main() -> CResult<()> {
    use SubCommand::*;
    let opts = Opts::parse();
    let _ = dotenv::dotenv();

    let db_path =
        env::var("CRYPTON_DB").unwrap_or_else(|_| "$HOME/.config/crypton/crypton.db".to_string());
    let db_path: PathBuf = shellexpand::full(&db_path)?.into_owned().into();
    if !db_path.is_file() {
        let prefix = db_path.parent().unwrap();
        std::fs::create_dir_all(prefix)?;
        std::fs::File::create(&db_path)?;
    }

    let pool = SqlitePool::connect(db_path.to_str().unwrap()).await?;
    sqlx::migrate!().run(&pool).await?;

    match opts.subcmd {
        List { tag: None } => list(&pool).await?,
        List { tag: Some(tag) } => list_tagged(&pool, &tag).await?,
        New(mut n) if n.note.is_empty() => {
            let tag = n.tag.take();
            new(&pool, tag).await?
        }
        New(mut n) => {
            let tag = n.tag.take();
            let note: String = n.into();
            let note = save_note(&pool, &note).await?;
            if let Some(tag) = tag {
                tag_note(&pool, &tag, note).await?
            }
        }
        Remove(h) => remove(&pool, h.arg).await?,
        Edit(h) => edit(&pool, h.arg).await?,
        Tags { subcommand } => tags(&pool, subcommand).await?,
    }
    Ok(())
}

async fn remove(pool: &SqlitePool, mut hash: String) -> CResult<()> {
    let mut conn = pool.acquire().await?;
    hash.push('%');
    sqlx::query!(
        r#"
DELETE FROM crypton_notes
WHERE (id, created_at) =
(SELECT id, created_at FROM crypton_notes
WHERE hash LIKE ?1
ORDER BY created_at DESC
LIMIT 1)
"#,
        hash
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

async fn list(pool: &SqlitePool) -> CResult<()> {
    let mut conn = pool.acquire().await?;
    let res = sqlx::query!(
        r#"
SELECT created_at, contents, hash FROM crypton_notes
        "#
    )
    .fetch_all(&mut conn)
    .await?;
    for record in res {
        println!(
            "{} | {} | {}",
            record.hash.chars().take(8).collect::<String>(),
            record.created_at,
            record.contents
        );
    }
    Ok(())
}

async fn list_tagged(pool: &SqlitePool, tag: &str) -> CResult<()> {
    let mut conn = pool.acquire().await?;
    let res = sqlx::query!(
        r#"
SELECT created_at, contents, hash FROM crypton_tag_mapping
JOIN crypton_notes ON note_id = crypton_notes.id
JOIN crypton_tags ON tag_id = crypton_tags.id
WHERE crypton_tags.name = ?1
        "#,
        tag
    )
    .fetch_all(&mut conn)
    .await?;
    for record in res {
        println!(
            "{} | {} | {}",
            record.hash.chars().take(8).collect::<String>(),
            record.created_at,
            record.contents
        );
    }
    Ok(())
}

async fn new(pool: &SqlitePool, tag: Option<String>) -> CResult<()> {
    let note = open_in_editor("")?;
    let note = save_note(pool, &note).await?;
    if let Some(tag) = tag {
        tag_note(pool, &tag, note).await?
    }
    Ok(())
}

async fn tag_note(pool: &SqlitePool, tag: &str, note: i64) -> CResult<()> {
    let mut conn = pool.acquire().await?;
    let tag = sqlx::query!(
        r#"
INSERT OR IGNORE INTO crypton_tags (name)
VALUES ( ?1 );
SELECT id FROM crypton_tags
ORDER BY id DESC
LIMIT 1;
        "#,
        tag
    )
    .fetch_one(&mut conn)
    .await?
    .id;
    sqlx::query!(
        r#"
INSERT INTO crypton_tag_mapping ( tag_id, note_id)
VALUES ( ?1, ?2 )
        "#,
        tag,
        note
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

async fn save_note(pool: &SqlitePool, note: &str) -> CResult<i64> {
    let trimmed = note.trim();
    if trimmed.is_empty() {
        return Err(AppError::EmptyNote.into());
    }
    let hash = note_hash(trimmed)?;

    let mut conn = pool.acquire().await?;
    let note = sqlx::query!(
        r#"
INSERT INTO crypton_notes (hash, contents )
VALUES ( ?1, ?2 )
        "#,
        hash,
        trimmed
    )
    .execute(&mut conn)
    .await?
    .last_insert_rowid();
    Ok(note)
}

async fn edit(pool: &SqlitePool, mut hash: String) -> CResult<()> {
    hash.push('%');
    let mut conn = pool.acquire().await?;
    let res = sqlx::query!(
        r#"
SELECT id, contents FROM crypton_notes
WHERE hash LIKE ?1
        "#,
        hash
    )
    .fetch_one(&mut conn)
    .await
    .map_err(|_| AppError::NotFound)?;
    let (note, id) = { (res.contents, res.id) };
    let note = open_in_editor(&note)?;
    if note.is_empty() {
        return Err(AppError::EmptyNote.into());
    }
    let hash = note_hash(&note)?;
    sqlx::query!(
        r#"
UPDATE crypton_notes
SET contents = ?1,
    hash = ?2
WHERE id = ?3
        "#,
        note,
        hash,
        id
    )
    .execute(&mut conn)
    .await?;
    Ok(())
}

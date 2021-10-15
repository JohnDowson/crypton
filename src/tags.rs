use crate::utils::CResult;
use clap::Subcommand;
use itertools::Itertools;
use sqlx::SqlitePool;

#[derive(Subcommand, Debug)]
pub enum TagsSubcommand {
    Remove,
    Rename,
    List,
}
pub async fn tags(pool: &SqlitePool, subcommand: Option<TagsSubcommand>) -> CResult<()> {
    use TagsSubcommand::*;
    match subcommand {
        Some(Remove) => todo!(),
        Some(Rename) => todo!(),
        Some(List) | None => list_tags(pool).await,
    }
}

async fn list_tags(pool: &SqlitePool) -> CResult<()> {
    let mut conn = pool.acquire().await?;

    #[allow(unstable_name_collisions)]
    let res = sqlx::query!(
        r#"
SELECT name FROM crypton_tags
        "#
    )
    .fetch_all(&mut conn)
    .await?
    .into_iter()
    .map(|r| r.name)
    .intersperse(", ".into())
    .collect::<String>();
    println!("{}", res);
    Ok(())
}

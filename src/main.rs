use anyhow::{anyhow, Ok, Result};
use clap::Parser;
use reqwest::{Client, Response, Url};
use std::{collections::HashMap, str::FromStr};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Opts {
  #[clap(subcommand)]
  subcmd: SubCommand,
}

#[derive(Parser, Debug)]
enum SubCommand {
  Get(Get),
  Post(Post),
}

#[derive(Parser, Debug)]
struct Get {
  url: String,
}

#[derive(Parser, Debug)]
struct Post {
  #[clap(parse(try_from_str = parse_url))]
  url: String,
  #[clap(parse(try_from_str = parse_kv_pair))]
  body: Vec<KvPair>,
}

#[derive(Debug)]
struct KvPair {
  k: String,
  v: String,
}

impl FromStr for KvPair {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut split = s.split("=");
    let err = || anyhow!(format!("Failed to parse {}", s));
    Ok(Self {
      k: (split.next().ok_or_else(err)?).into(),
      v: (split.next().ok_or_else(err)?).into(),
    })
  }
}
// 1. parse command
fn parse_kv_pair(s: &str) -> Result<KvPair> {
  Ok(s.parse()?)
}

fn parse_url(s: &str) -> Result<String> {
  let _url: Url = s.parse()?;
  Ok(s.into())
}

// 2. do some action
async fn get(client: Client, args: &Get) -> Result<()> {
  let resp = client.get(&args.url).send().await?;
  println!("{:?}", resp.text().await?);
  Ok(())
}

async fn post(client: Client, args: &Post) -> Result<()> {
  let mut body = HashMap::new();
  for pair in args.body.iter() {
    body.insert(&pair.k, &pair.v);
  }
  let resp = client.post(&args.url).json(&body).send().await?;
  Ok(print_resp(&resp))
}

fn print_resp(resp: &Response) {
  println!("{}", resp.status())
}
#[tokio::main]
async fn main() -> Result<()> {
  let opts: Opts = Opts::parse();
  let client = Client::new();
  let result = match opts.subcmd {
    SubCommand::Get(args) => get(client, &args).await?,
    SubCommand::Post(args) => post(client, &args).await?,
  };
  Ok(result)
}

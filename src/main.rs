use std::io;
use std::process::{self, Command};

fn get_anime_title() -> String {
    println!("Inserisci il titolo dell'anime da guardare: ");

    let mut anime_name = String::new();
    io::stdin().read_line(&mut anime_name).unwrap();
    let anime_name = anime_name.trim();

    let anime_name_url = anime_name.replace(" ", "+");
    let search_url = format!(
        "https://www.animeworld.ac/search?keyword={}",
        anime_name_url
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .build()
        .unwrap();

    let response = client.get(&search_url).send().unwrap();
    let html_content = response.text().unwrap();

    let document = scraper::Html::parse_document(&html_content);

    let anime_selector = scraper::Selector::parse("div.item a.name").unwrap();

    struct AnimeList {
        url: String,
        name: String,
    }

    let anime: Vec<AnimeList> = document
        .select(&anime_selector)
        .map(|a| {
            let url = a
                .value()
                .attr("href")
                .map(|href| format!("https://www.animeworld.ac{}", href))
                .unwrap_or_default();

            let name = a
                .value()
                .attr("data-jtitle")
                .unwrap_or("")
                .to_string();

            AnimeList { url, name }
        })
        .collect();

    for (i, a) in anime.iter().enumerate() {
        println!("[{}] Nome: {} | URL: {}", i + 1, a.name, a.url);
    }

    println!("\nInserisci il numero dell'anime: ");
    let mut scelta = String::new();
    io::stdin().read_line(&mut scelta).unwrap();
    let scelta: usize = scelta.trim().parse().unwrap();

    if let Some(a) = anime.get(scelta - 1) {
        return a.url.clone();
    } else {
        println!("Numero non valido!");
        process::exit(1);
    }
}

fn get_episode_url(anime_url: &str) -> String {
    let response = reqwest::blocking::get(anime_url).unwrap();
    let html_content = response.text().unwrap();

    let main_website = "https://www.animeworld.ac";

    let document = scraper::Html::parse_document(&html_content);

    let episode_selector = scraper::Selector::parse("li.episode").unwrap();
    let a_selector = scraper::Selector::parse("a").unwrap();

    let mut episodes: Vec<String> = Vec::new();

    for episode in document.select(&episode_selector) {
        if let Some(a) = episode.select(&a_selector).next() {
            if let Some(href) = a.value().attr("href") {
                episodes.push(href.to_string());
            }
        }
    }

    if episodes.is_empty() {
        println!("Nessun episodio trovato");
        process::exit(1);
    }

    let max_episode = episodes.len();

    println!("Inserisci episodio (1-{})", max_episode);

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    let episode: usize = match input.trim().parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Input non valido");
            process::exit(1);
        }
    };

    if episode == 0 || episode > max_episode {
        println!("Episodio non valido");
        process::exit(1);
    }

    format!("{}{}", main_website, episodes[episode - 1])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let anime_url = get_anime_title();
    let episode_site = get_episode_url(&anime_url);

    println!("Pagina episodio: {}", episode_site);

    let response = reqwest::blocking::get(&episode_site)?;
    let html_content = response.text()?;

    let document = scraper::Html::parse_document(&html_content);
    let selector = scraper::Selector::parse("a#alternativeDownloadLink")?;

    let link_download = document
        .select(&selector)
        .next()
        .and_then(|a| a.value().attr("href"))
        .ok_or("link non trovato")?
        .to_string();

    println!("Stream URL: {}", link_download);

    Command::new("mpv")
        .arg(&link_download)
        .arg("--cache=yes")
        .arg("--cache-secs=500")
        .spawn()?;

    println!("Video avviato in streaming");

    Ok(())
}
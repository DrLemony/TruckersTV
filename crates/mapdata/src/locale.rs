//! The language the game is set to.

/// The game's language, from its `config.cfg` (in the player's documents, `Euro Truck Simulator 2/`):
/// `uset g_lang_init "tr_tr"`.
pub fn game_language(config: &str) -> Option<String> {
    config.lines().find_map(|line| {
        let mut words = line.split_whitespace();
        (words.next() == Some("uset") && words.next() == Some("g_lang_init"))
            .then(|| words.next())
            .flatten()
            .map(|value| value.trim_matches('"').to_owned())
            .filter(|language| !language.is_empty())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_language_from_the_config() {
        let config = "uset g_fps_limit \"60\"\nuset g_lang_init \"tr_tr\"\nuset g_other \"1\"\n";
        assert_eq!(game_language(config), Some("tr_tr".to_owned()));
        assert_eq!(game_language("uset g_lang_init \"\"\n"), None);
        assert_eq!(game_language(""), None);
    }
}

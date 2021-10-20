// Code for searching battle logs.

use crate::directory::LogParser;

// Directly taken from https://github.com/AnnikaCodes/battlesearch/blob/main/src/search.rs
use std::{path::PathBuf, fs};
use crate::{BattleToolsError, id::to_id};

pub struct BattleSearcher {
    pub user_id: String,
    pub wins_only: bool,
    pub forfeits_only: bool,
}

fn bytes_to_id(bytes: &Option<&[u8]>) -> Option<String> {
    match bytes {
        Some(b) => Some(to_id(&String::from_utf8_lossy(b))),
        None => None,
    }
}


impl BattleSearcher {
    pub fn new(
        username: &str,
        wins_only: bool,
        forfeits_only: bool,
    ) -> Self {
        Self {
            user_id: to_id(username),
            wins_only,
            forfeits_only,
        }
    }
}

impl LogParser<()> for BattleSearcher {
    /// json is in the form [p1name, p2name, winner, endType]
    /// TODO: handle `date`
    fn handle_log_file(&self, raw_json: String) -> Result<(), BattleToolsError> {
        let date = "TODO";
        let file_name = "TODO";
        let mut json_parser = pikkr_annika::Pikkr::new(
            &vec![
                "$.p1".as_bytes(),      // p1 name - idx 0
                "$.p2".as_bytes(),      // p2 name - idx 1
                "$.winner".as_bytes(),  // winner - idx 2
                "$.endType".as_bytes(), // end type - idx 3
            ],
            2,
        )
        .unwrap();
        let json = json_parser.parse(&raw_json).unwrap();

        if json.len() != 4 {
            // should never happen
            return Err(BattleToolsError::InvalidLog(format!(
                "BattleSearcher::check_log(): found {} elements in parsed JSON (expected 4)",
                json.len()
            )));
        }

        // parse players
        let p1id = match bytes_to_id(json.get(0).unwrap()) {
            Some(a) => a,
            None => return Err(BattleToolsError::InvalidLog(format!("No p1 value"))),
        };
        let p2id = match bytes_to_id(json.get(1).unwrap()) {
            Some(a) => a,
            None => return Err(BattleToolsError::InvalidLog(format!("No p2 value"))),
        };
        let p1_is_searched_user = p1id == self.user_id;
        let p2_is_searched_user = p2id == self.user_id;
        if !p1_is_searched_user && !p2_is_searched_user {
            // Searched user is not a player in the battle.
            return Ok(());
        }

        // parse winner
        let winner_id = bytes_to_id(json.get(2).unwrap());
        let searched_user_won = match winner_id {
            Some(ref winner) => winner == &self.user_id,
            None => false,
        };
        if !searched_user_won && self.wins_only {
            return Ok(());
        }

        // parse endType
        let is_forfeit = match json.get(3).unwrap() {
            Some(bytes) => String::from_utf8_lossy(bytes) == "\"forfeit\"",
            None => false,
        };
        if !is_forfeit && self.forfeits_only {
            return Ok(());
        }

        // formatting
        let win_type_str = if is_forfeit { "by forfeit" } else { "normally" };
        let win_str = match winner_id {
            Some(ref winner) => format!("{} won {}", winner, win_type_str),
            None => String::from("there was no winner"),
        };

        let room = file_name.replace(".log.json", "");

        println!(
            "({}) <<{}>> {} vs. {} ({})",
            date, room, p1id, p2id, win_str
        );

        Ok(())
    }

    fn handle_results(&mut self, _results: Vec<()>) -> Result<(), BattleToolsError> {
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;
    use lazy_static::lazy_static;
    use test::Bencher;
    use crate::{testing::*, directory::ParallelDirectoryParser};

    lazy_static! {
        static ref SAMPLE_JSON: String = String::from(
            r#"{"winner":"Annika","seed":[1,1,1,1],"turns":2,"p1":"Annika","p2":"Rust Haters","p1team":[{"name":"Rotom","species":"Rotom-Fan","gender":"N","shiny":false,"gigantamax":false,"level":84,"moves":["airslash","voltswitch","willowisp","thunderbolt"],"ability":"Levitate","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":31},"item":"Heavy-Duty Boots"},{"name":"Regirock","species":"Regirock","gender":"N","shiny":false,"gigantamax":false,"level":85,"moves":["curse","rockslide","rest","bodypress"],"ability":"Sturdy","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Chesto Berry"},{"name":"Conkeldurr","species":"Conkeldurr","gender":"","shiny":false,"gigantamax":false,"level":80,"moves":["facade","knockoff","machpunch","drainpunch"],"ability":"Guts","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Flame Orb"},{"name":"Reuniclus","species":"Reuniclus","gender":"","shiny":false,"gigantamax":false,"level":84,"moves":["trickroom","focusblast","psychic","shadowball"],"ability":"Magic Guard","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":0},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":0},"item":"Life Orb"},{"name":"Incineroar","species":"Incineroar","gender":"","shiny":false,"gigantamax":false,"level":80,"moves":["knockoff","uturn","earthquake","flareblitz"],"ability":"Intimidate","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Scarf"},{"name":"Miltank","species":"Miltank","gender":"F","shiny":false,"gigantamax":false,"level":84,"moves":["healbell","bodyslam","earthquake","milkdrink"],"ability":"Sap Sipper","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Leftovers"}],"p2team":[{"name":"Drednaw","species":"Drednaw","gender":"","shiny":false,"gigantamax":false,"level":84,"moves":["stoneedge","swordsdance","superpower","liquidation"],"ability":"Swift Swim","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Life Orb"},{"name":"Pinsir","species":"Pinsir","gender":"","shiny":false,"gigantamax":false,"level":84,"moves":["closecombat","stoneedge","xscissor","knockoff"],"ability":"Moxie","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Scarf"},{"name":"Pikachu","species":"Pikachu-Sinnoh","gender":"","shiny":false,"gigantamax":false,"level":92,"moves":["knockoff","volttackle","voltswitch","irontail"],"ability":"Lightning Rod","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Light Ball"},{"name":"Latios","species":"Latios","gender":"M","shiny":false,"gigantamax":false,"level":78,"moves":["dracometeor","calmmind","psyshock","roost"],"ability":"Levitate","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":31},"item":"Soul Dew"},{"name":"Entei","species":"Entei","gender":"N","shiny":false,"gigantamax":false,"level":78,"moves":["flareblitz","stoneedge","extremespeed","sacredfire"],"ability":"Inner Focus","evs":{"hp":85,"atk":85,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":31,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Band"},{"name":"Exeggutor","species":"Exeggutor-Alola","gender":"","shiny":false,"gigantamax":false,"level":86,"moves":["gigadrain","flamethrower","dracometeor","leafstorm"],"ability":"Frisk","evs":{"hp":85,"atk":0,"def":85,"spa":85,"spd":85,"spe":85},"ivs":{"hp":31,"atk":0,"def":31,"spa":31,"spd":31,"spe":31},"item":"Choice Specs"}],"score":[0,2],"inputLog":[">lol you thought i'd leak someone's real input log"],"log":["|j|☆Annika","|j|☆Rust Hater","|player|p1|Annika|cynthia|1400","|player|p2|Rust Hater|cynthia|1100","|teamsize|p1|6","|teamsize|p2|6","|gametype|singles","|gen|8","|tier|[Gen 8] Random Battle","|rated|"],"p1rating":{"entryid":"75790599","userid":"annika","w":"4","l":4,"t":"0","gxe":46.8,"r":1516.9377700433,"rd":121.36211247153,"rptime":1632906000,"rpr":1474.7452159936,"rprd":115.09180605287,"elo":1400.4859871929,"col1":8,"oldelo":"1057.7590112468"},"p2rating":{"entryid":"75790599","userid":"rusthater","w":"4","l":5,"t":"0","gxe":41.8,"r":"1516.9377700433","rd":"121.36211247153","rptime":"1632906000","rpr":1434.9434039083,"rprd":109.84367373045,"elo":1130.7522733629,"col1":9,"oldelo":"1040.4859871929"},"endType":"normal","timestamp":"Wed Nov 1 1970 00:00:01 GMT-0400 (Eastern Daylight Time)","roomid":"battle-gen8randombattle-1","format":"gen8randombattle", "comment": "if you're curious - this is my own rating info & teams from my battles - no violation of privacy here!"}"#
        );
    }

    #[bench]
    pub fn bench_parse_wins_only(b: &mut Bencher) {
        let searcher = BattleSearcher::new("Rusthaters", true, false);
        b.iter(|| searcher.handle_log_file(SAMPLE_JSON.clone()));
    }

    #[bench]
    pub fn bench_parse_forfeits_only(b: &mut Bencher) {
        let searcher = BattleSearcher::new("Rusthaters", false, true);
        b.iter(|| searcher.handle_log_file(SAMPLE_JSON.clone()));
    }

    #[bench]
    pub fn bench_parse_forfeit_wins_only(b: &mut Bencher) {
        let searcher = BattleSearcher::new("Rusthaters", true, true);
        b.iter(|| searcher.handle_log_file(SAMPLE_JSON.clone()));
    }

    #[bench]
    pub fn bench_parse(b: &mut Bencher) {
        let searcher = BattleSearcher::new("Rusthaters", false, false);
        b.iter(|| searcher.handle_log_file(SAMPLE_JSON.clone()));
    }

    #[bench]
    fn bench_handle_directory_1k(b: &mut Bencher) {
        build_test_dir(1_000).unwrap();

        let mut searcher = BattleSearcher::new("Rusthaters", false, false);
        b.iter(|| {
            searcher
                .handle_directories(vec![TEST_ROOT_DIR.clone()])
                .unwrap()
        });
    }}
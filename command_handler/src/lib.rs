use std::{
    process::{Command,Output},
    str,
};

struct CommandData {
    command: String,
    closure: Box<dyn Fn(String, String) -> String>,
}
impl CommandData{
    fn new(command: &String) -> Option<CommandData>
    {
        let mut parts = command.split(",");
        let default_closure = Box::new(|stdout, _stderr| {stdout});
        match parts.next()?.trim()
        {
            "action" => {
                match parts.next()?.trim()
                {
                    "getVolume" => {
                        let command = String::from("pamixer --get-volume");
                        return Some(CommandData{command, closure: default_closure});
                    },
                    "setVolume" => {
                        let new_val = parts.next()?;
                        if !is_string_in_range(new_val.to_string(), 0, 100) {return None}
                        let command = format!("pactl set-sink-volume @DEFAULT_SINK@ {new_val}%");
                        return Some(CommandData{command, closure: default_closure});
                    },
                    "toggleScreen" => {
                        let command = String::from("./command_handler/scripts/ToggleLaptopScreen.sh");
                        let closure =  Box::new(|_, _| {String::from("success")}); 
                        return Some(CommandData{command, closure});
                    },
                    "setBrightness" => {
                        let brightness = parts.next()?;
                        if !is_string_in_range(brightness.to_string(), 0, 100){return None}
                        let command = format!("brightnessctl -q s {brightness}%");
                        let closure = Box::new(|_,_| {String::from("success")});
                        return Some(CommandData{command,closure});
                    },
                    "getMediaData" => {
                        let command = get_media_data();
                        return Some(CommandData{command, closure: default_closure}); 
                    },
                    "playPauseMediaPlayer" => {
                        let player = parts.next()?;
                        if !valid_player(player.to_string()) {return None} 
                        let closure = Box::new(|_,_| {String::from("success")});
                        let command = format!("playerctl play-pause -p {player}");
                        return Some(CommandData{command,closure});
                    },
                    "nextMediaPlayer" => {
                        let player = parts.next()?;
                        if !valid_player(player.to_string()) {return None} 
                        let command = format!("playerctl next -p {player}");
                        let closure = Box::new(|_,_| {String::from("success")});
                        return Some(CommandData{command,closure});
                    },
                    "previousMediaPlayer" => {
                        let player = parts.next()?;
                        if !valid_player(player.to_string()) {return None} 
                        let command = format!("playerctl previous -p {player}");
                        let closure = Box::new(|_,_| {String::from("success")});
                        return Some(CommandData{command,closure});
                    },
                    "getPlayers" => {
                        let command = String::from("playerctl -l");
                        return Some(CommandData{command,closure:default_closure});
                    },
                    "getPlayingMedia" => {
                        let player = parts.next()?;
                        if !valid_player(player.to_string()) {return None}
                        let command = format!("playerctl metadata -p {player}");
                        return Some(CommandData{command, closure:default_closure});
                    }
                    "toggleShuffle" => {
                        let player = parts.next()?;
                        if !valid_player(player.to_string()) {return None}
                        let command = format!("playerctl shuffle toggle -p {player}");
                        return Some(CommandData{command, closure:default_closure});
                    }
                    _ => return None,

                }
            },
            "key" => {todo!("implement key")},
            _ => return None,
        }
    }
}
fn is_string_in_range(str:String, num1:i32, num2:i32) -> bool
{
    let num: Result<i32,_> = str.parse();
    match num
    {
        Ok(num) => {
            return num>=num1 && num<=num2;
        }
        Err(_) => return false
    };

}
fn run_command(command: String) -> Output 
{
    let output = if cfg!(target_os = "linux") {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process")
    } else{
        todo!("not on linux shell");
    };
    return output;
}
fn convert_utf_to_string(vec: &Vec<u8>) -> String
{
    match str::from_utf8(&vec) {
        Ok(v) => v.to_string(),
        Err(e) =>panic!("Invalid UTF-8 sequence: {}",e)
    }
}
fn handle_command(command: String) -> String 
{
    let Some(parsed_command) = parse_command(&command) else {return format!("invalid request: {command}")};

    let command_output = run_command(parsed_command.command);

    let s1 = convert_utf_to_string(&command_output.stdout);
    let s2 = convert_utf_to_string(&command_output.stderr);

    println!("stdout result: {}", s1);
    if s2.len() != 0 {
        println!("stderr result {s2}");
    }
    return (parsed_command.closure)(s1.to_string(), s2.to_string());
}

fn parse_command(command: &String) -> Option<CommandData>
{
    let command = CommandData::new(command);
    return command;
}


pub fn handle_request(command: String) -> String
{
    handle_command(command)
}

fn get_media_players() -> String
{
    let players = run_command(String::from("playerctl -l"));
    return convert_utf_to_string(&players.stdout);
}

fn get_media_data() -> String
{
    let players = get_media_players();
    let mut output = String::new();
    for player in players.lines() {
        output =output+ "playerctl metadata -p " + player + ";";
    }
    return output;
}

fn valid_player(player:String) ->bool {
    let players = get_media_players();
    for media_player in players.lines() {
        if media_player == player {return true}
    }
    return false;
}

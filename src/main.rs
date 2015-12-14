extern crate time;
extern crate hueclient;
use hueclient::bridge::Bridge;
use hueclient::errors::HueError;
use hueclient::bridge::Light;
use hueclient::bridge::CommandLight;
use std::process::Command;
use std::time::Duration;
use std::thread::sleep;

fn do_register(bridge: &Bridge) {
    println!("Got hue bridge: {:?}", bridge);
    loop {
        let r = bridge.register_user("homelights", "homelights");
        match r {
            Ok(r) => {
                println!("Done: {:?}", r);
                break;
            },
            Err(HueError::BridgeError(ref error)) if error.code == 101 => {
                println!("Push bridge button!");
                std::thread::sleep_ms(1000);
            },
            Err(e) => panic!(e)
        }
    }
}

fn current_state() -> State {
    match Command::new("ping")
        .arg("android-e5116e9ab0465563.oob.hackerbots.net.")
        .arg("-c 1")
        .output().ok().unwrap()
        .status.success() {
        true => { 
            let now = time::now();
            if now.tm_hour > 8 && now.tm_hour < 9 && now.tm_wday >= 1 && now.tm_wday <= 5 {
                State::Leaving
            } else {
                State::Home
            }
        },
        false => State::Away
    }
}

fn enter_state(light: &Light, state: State) -> Option<CommandLight> {
    match light.name.trim() {
        "Entrance" =>
            match state {
                State::Home =>
                    Some(CommandLight::on().with_hue(0).with_bri(5)),
                State::Away =>
                    Some(CommandLight::off()),
                State::Leaving =>
                    Some(CommandLight::on().with_ct(7500).with_bri(255)),
                _ => None
            },
        _ => None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Unknown,
    Home,
    Leaving,
    Away
}

fn activate_state(state: State, bridge: &Bridge) {
    println!("Entering {:?} mode", state);
    match bridge.get_all_lights() {
        Ok(lights) => {
            for ref light in lights.iter() {
                let cmd = enter_state(&light.light, state);
                match cmd {
                    Some(cmd) => {
                        bridge.set_light_state(light.id, cmd);
                    },
                    None => ()
                }
            }
        },
        Err(err) => panic!(err)
    }
}

fn main() {
    let bridge = Bridge::discover_required().with_user("homelights".to_string());
    let mut lastState = State::Unknown;
    activate_state(lastState, &bridge);
    loop {
        let state = current_state();
        if lastState != state {
            activate_state(state, &bridge);
            lastState = state;
        }
        sleep(Duration::new(5, 0));
    }
}

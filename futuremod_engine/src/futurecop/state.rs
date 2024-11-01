use super::{
    global::{Global, SelectedGameMode, VolatileGlobal},
    EVENTS, FRAME_NUMBER, FUTURE_COP_MODULE, GAME_MODE, HEAP, IN_GAME_LOOP, IS_PLAYING,
    IS_TWO_PLAYER, MAIN_WINDOW, SCENE,
};

#[derive(Debug)]
pub struct Mission {
    pub name: Global<String>,
    pub filepath: Global<String>,
    pub loaded: Global<bool>,
    pub loading_progress: VolatileGlobal<u8>,
}

#[derive(Debug)]
pub struct WindowHandles {
    pub main_window: VolatileGlobal<u32>,
    pub heap: VolatileGlobal<u32>,
    pub future_cop_module: VolatileGlobal<u32>,
    pub events: VolatileGlobal<u32>,
}

#[derive(Debug)]
pub struct GameState {
    pub in_game_loop: VolatileGlobal<bool>,
    pub is_two_player: VolatileGlobal<bool>,
    pub is_playing: VolatileGlobal<bool>,
    pub game_mode: SelectedGameMode,
    pub scene: VolatileGlobal<u8>,
}

/// Information about FutureCop
#[derive(Debug)]
pub struct FutureCopState {
    pub state: GameState,
    pub current_mission: Option<Mission>,
    pub frame_number: VolatileGlobal<u32>,
    pub handles: WindowHandles,
}

pub static mut FUTURE_COP: FutureCopState = FutureCopState {
    state: GameState {
        in_game_loop: IN_GAME_LOOP,
        is_two_player: IS_TWO_PLAYER,
        is_playing: IS_PLAYING,
        game_mode: GAME_MODE,
        scene: SCENE,
    },
    current_mission: None,
    frame_number: FRAME_NUMBER,
    handles: WindowHandles {
        main_window: MAIN_WINDOW,
        heap: HEAP,
        future_cop_module: FUTURE_COP_MODULE,
        events: EVENTS,
    },
};

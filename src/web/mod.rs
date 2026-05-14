pub mod ui;

use crate::game::{Game, GameState};
use crate::snake::Direction;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let container = document
        .get_element_by_id("game-container")
        .expect("should have #game-container on the page")
        .dyn_into::<web_sys::HtmlElement>()?;

    // Create canvas
    let canvas = document.create_element("canvas")?.dyn_into::<web_sys::HtmlCanvasElement>()?;

    // Set canvas size (cell size = 15 pixels)
    canvas.set_width(40 * 15);
    canvas.set_height(20 * 15);

    container.set_inner_html(""); // Clear loading text
    container.append_child(&canvas)?;

    let ctx = canvas.get_context("2d")?.unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    let game =
        Game::new(40, 20, false, 'O', crate::game::Theme::Classic, crate::game::Difficulty::Normal);

    let game = Rc::new(RefCell::new(game));
    let ctx = Rc::new(RefCell::new(ctx));
    let doc = Rc::new(RefCell::new(document));

    // Setup keyboard listener
    {
        let game = game.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::KeyboardEvent| {
            let mut game = game.borrow_mut();
            match event.key().as_str() {
                "w" | "W" => {
                    game.handle_input(Direction::Up, 1);
                },
                "s" | "S" => {
                    game.handle_input(Direction::Down, 1);
                },
                "a" | "A" => {
                    game.handle_input(Direction::Left, 1);
                },
                "d" | "D" => {
                    game.handle_input(Direction::Right, 1);
                },
                "ArrowUp" => {
                    game.handle_input(Direction::Up, 2);
                },
                "ArrowDown" => {
                    game.handle_input(Direction::Down, 2);
                },
                "ArrowLeft" => {
                    game.handle_input(Direction::Left, 2);
                },
                "ArrowRight" => {
                    game.handle_input(Direction::Right, 2);
                },
                "Enter" => {
                    game.shoot_laser(2);
                },
                " " => {
                    if game.state == GameState::Menu {
                        game.state = GameState::Playing;
                    } else if game.state == GameState::Playing {
                        game.shoot_laser(1);
                    }
                },
                "t" | "T" => {
                    game.auto_pilot = !game.auto_pilot;
                    if game.auto_pilot {
                        game.used_bot_this_session = true;
                    }
                },
                "r" | "R" => {
                    if game.state == GameState::GameOver {
                        game.reset();
                    }
                },
                "p" | "P" => {
                    if game.state == GameState::Playing {
                        game.state = GameState::Paused;
                    } else if game.state == GameState::Paused {
                        game.state = GameState::Playing;
                    }
                },
                _ => {},
            }
        });
        doc.borrow()
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Game loop
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut last_time = window.performance().unwrap().now();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |time: f64| {
        let mut game_ref = game.borrow_mut();
        let ctx_ref = ctx.borrow();

        let delta = time - last_time;
        if delta > 100.0 {
            // 10 FPS
            if game_ref.state == GameState::Playing {
                game_ref.update();
            }
            ui::draw(&game_ref, &ctx_ref);
            last_time = time;
        }

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut(f64)>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

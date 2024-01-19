use js_sys::*;
use leptos::*;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::JsFuture;
use web_sys::*;

#[component]
pub fn HomePage() -> impl IntoView {
    let game_action = create_action(|_: &()| async move { run(()).await });
    let pending = game_action.pending();
    let r = game_action.dispatch(());

    view! {
        <h1>"↓↓↓↓ game ↓↓↓↓"</h1>
        <div id="game-container">
            <canvas id="bevy"></canvas>
            <button
                id="run-button"
                on:click=move |_| {
                    game_action.dispatch(());
                }
            >

                "Run Game"
            </button>
        </div>
    }
}

async fn run(_: ()) -> Result<(), JsValue> {
    let w = web_sys::window().unwrap_throw();
    let document = w.document().unwrap_throw();

    // Load the JavaScript file that initializes the WASM module
    let script = document
        .create_element("script")?
        .dyn_into::<HtmlScriptElement>()?;
    script.set_type("module");
    script.set_inner_html(
        r#"
        import init from 'https://untitled-game.b-cdn.net/mygame.js';

        async function runWasm() {
            try {
                const wasm = await init('https://untitled-game.b-cdn.net/mygame_bg.wasm');
                if (wasm && wasm.main) {
                    wasm.main();
                }
            } catch (error) {
                console.error('Error loading WASM:', error);
            }
        }

        runWasm();
    "#,
    );
    document.body().unwrap_throw().append_child(&script)?;

    Ok(())
}


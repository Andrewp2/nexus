use leptos::{component, create_action, create_signal, view, IntoView};
use web_sys::{
    wasm_bindgen::JsCast, wasm_bindgen::JsValue, wasm_bindgen::UnwrapThrowExt, HtmlScriptElement,
};

#[component]
pub fn Home() -> impl IntoView {
    let game_action = create_action(|_: &()| async move { run(()).await });
    let (invisible, set_invisible) = create_signal(false);

    view! {
        <h1>"↓↓↓↓ game ↓↓↓↓"</h1>
        <div id="game-container">
            <canvas id="bevy"></canvas>
            <button
                id="run-button"
                class:invisible=move || invisible()
                on:click=move |_| {
                    game_action.dispatch(());
                    set_invisible(true);
                }

                class="transition-colors duration-300 ease-in-out text-[#color] py-1.5 px-1.5 bg-[#primary-color] rounded-md"
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

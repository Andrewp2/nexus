use std::error::Error;

use crate::{
    error_template::{AppError, ErrorTemplate},
    server::public::Login,
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    view! {
        <Stylesheet id="leptos" href="/pkg/nexus.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        <link rel="preconnect" href="https://fonts.googleapis.com"/>
        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin/>
        <link
            href="https://fonts.googleapis.com/css2?family=Roboto+Slab&display=swap"
            rel="stylesheet"
        />

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <body>
                <Header/>
                <main>
                    <Routes>
                        <Route path="" view=HomePage/>
                        <Route path="about" view=AboutPage/>
                        <Route path="community" view=CommunityPage/>
                        <Route path="terms_and_conditions" view=TermsAndConditionsPage/>
                        <Route path="credits" view=CreditsPage/>
                        <Route path="support" view=SupportFAQPage/>
                        <Route path="log_in" view=LoginPage/>
                        <Route path="email_verification" view=EmailVerification/>
                        <Route path="email_verification/:email_uuid" view=EmailVerificationAttempt/>
                    </Routes>
                </main>
                <Footer/>
            </body>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click class:red=move || count() % 2 == 1>
            "Click Me: "
            {count}
        </button>
    }
}

#[component]
fn AboutPage() -> impl IntoView {
    view! {
        <h1>"$GAME_NAME is an exploration into the cave world of $WORLD_NAME."</h1>
        <p>Build, craft, explore, and conquer the ancient ruined cities beneath the earth.</p>
    }
}

#[component]
fn CommunityPage() -> impl IntoView {
    view! { <A href="https://discord.gg/EuqSvxDPRY">Join the official Discord!</A> }
}

#[component]
fn TermsAndConditionsPage() -> impl IntoView {
    view! {
        <p>This is where the TOS and EULA will go once they are created.</p>
        <p>
            In the meantime, understand I do not take responsibility for any harms that may come to the player while playing.
        </p>
        <p>You are allowed to stream this game</p>
    }
}

#[component]
fn CreditsPage() -> impl IntoView {
    view! {}
}

#[component]
fn SupportFAQPage() -> impl IntoView {
    view! {
        <h1>FAQ</h1>
        <ul>
            <li>What</li>
        </ul>
        <p>Contact andrewpetersongamedev@gmail.com for any issues with support</p>
    }
}

#[component]
fn EmailVerification() -> impl IntoView {
    view! {
        <h1>
            You should be recieving an email to the email address you specified when logging in.
        </h1>
        <h2>Click on that link, and you can log in as you wish.</h2>
    }
}

#[derive(Params, PartialEq, Clone)]
pub struct EmailVerificationParams {
    uuid: String,
}

#[component]
fn EmailVerificationAttempt() -> impl IntoView {
    let params = use_params::<EmailVerificationParams>();
    let uuid = move || {
        params.with(|params| {
            params
                .as_ref()
                .map(|params| params.uuid.clone())
                .unwrap_or_default()
        })
    };

    let x = create_resource(
        || (),
        move |_| async move { crate::server::public::verify_email(uuid()).await },
    );

    let f = move || match x.get() {
        None => view! { <div>hi there</div> },
        Some(s) => match s {
            Ok(_) => view! { <div>Verification was successful</div> },
            Err(e) => view! { <div>error oh no</div> },
        },
    };

    view! { <div>{f}</div> }
}

#[component]
fn LoginPage() -> impl IntoView {
    let login = create_server_action::<Login>();
    let error: RwSignal<Option<Box<dyn Error>>> = create_rw_signal(Default::default());
    view! {
        <ActionForm action=login error=error>
            <h1>"Log in"</h1>
            <label>
                "Email:" <br/>
                <input
                    type="email"
                    placeholder="Email"
                    maxlength="32"
                    name="email"
                    class="auth-input"
                    required
                />
            </label>
            <br/>
            <label>
                "Password:" <br/>
                <input
                    type="password"
                    placeholder="Password"
                    name="password"
                    class="auth-input"
                    required
                    minlength="10"
                />
            </label>
            <br/>
            <label>
                <input type="checkbox" name="remember" class="auth-input"/>
                "Remember me?"
            </label>
            <br/>
            <button type="submit" class="button">
                "Log In"
            </button>
        </ActionForm>
    }
}

#[component]
fn Header() -> impl IntoView {
    view! {
        <header class="header">
            <div class="headerlinks">
                <img/>
                <nav class="nav">
                    <A href="">Home</A>
                    <A href="about">About</A>
                    <A href="community">Community</A>
                    <A href="support">Help</A>
                </nav>
                <div class="authgroup">
                    <A href="log_in">Log in</A>
                    |
                    <A href="log_in">Register</A>
                </div>
            </div>
        </header>
    }
}

#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class:footer=true>
            <div class="footeritems">
                Copyright 2023-2023 Andrew Peterson.
                <A href="terms_and_conditions">Terms and conditions</A>
                <A href="credits">Credits</A> <A href="support">Support / FAQ</A>
            </div>
        </footer>
    }
}


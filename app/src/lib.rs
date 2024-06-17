pub mod common;
pub mod error_template;
pub mod errors;
pub mod pages;
pub mod public;
pub mod site;

#[cfg(feature = "ssr")]
pub mod server;

use crate::{
    common::{footer::Footer, header::Header},
    error_template::{AppError, ErrorTemplate},
    pages::{
        about::About, checkout::Checkout, checkout_cancel::CheckoutCancel,
        checkout_success::CheckoutSuccess, credits::Credits, download::Download,
        email_verification::EmailVerification,
        email_verification_attempt::EmailVerificationAttempt,
        end_user_license_agreement::EndUserLicenseAgreement, home::Home,
        login_and_signup::LoginAndSignup, support_faq::SupportFAQ,
    },
};
use leptos::{
    component, create_server_action, create_signal, provide_context, view, Errors, IntoView, Memo,
    SignalGet,
};
use leptos_meta::{provide_meta_context, Stylesheet, Title};
use leptos_router::{ProtectedRoute, Route, Router, Routes};
use public::{Login, Logout};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum AccountState {
    LoggedIn,
    LoggedOut,
}

impl Default for AccountState {
    fn default() -> Self {
        AccountState::LoggedOut
    }
}

#[derive(Debug, Clone)]
struct CSRFToken {
    value: String,
}

#[component]
pub fn NexusApp() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    let (bought_game, set_bought_game) = create_signal(false);
    let csrf_token: Option<CSRFToken> = Option::None;
    provide_context(csrf_token);
    let login = create_server_action::<Login>();
    let logout = create_server_action::<Logout>();

    let state = Memo::new(move |prev: Option<&AccountState>| {
        let logged_in_value = login.value().get();
        let logged_out_value = logout.value().get();

        // TODO: maintain login when closing tabs
        match (logged_in_value, logged_out_value, prev) {
            (Some(Ok(_)), None, None) => AccountState::LoggedIn,
            (_, Some(Ok(_)), None) => AccountState::LoggedOut,
            (Some(Ok(_)), _, Some(AccountState::LoggedOut)) => AccountState::LoggedIn,
            (_, Some(Ok(_)), Some(AccountState::LoggedIn)) => AccountState::LoggedOut,
            (None, None, None) => AccountState::LoggedOut,
            errors => {
                log::error!("{errors:#?}");
                AccountState::default()
            }
        }
    });

    view! {
        <Stylesheet id="leptos" href="/pkg/nexus.css"/>

        // sets the document title
        <Title text="Project Glint"/>

        <link rel="preconnect" href="https://fonts.googleapis.com"/>
        <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin/>
        <link
            href="https://fonts.googleapis.com/css2?family=Roboto+Slab&display=swap"
            rel="stylesheet"
        />

        <script src="https://js.stripe.com/v3/"></script>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <body class="min-h-screen font-['Roboto_Slab'] flex flex-col bg-background-color text-t-color m-0 p-0">
                <Header logout_action=logout account_state=state/>
                <main class="flex-1 p-10 bgbackground-color backdrop-blur-md">
                    <Routes>
                        <Route path="" view=Home/>
                        <Route path="about" view=About/>
                        <Route path="terms_of_service" view=EndUserLicenseAgreement/>
                        <Route path="credits" view=Credits/>
                        <Route path="support" view=SupportFAQ/>
                        <Route
                            path="log_in"
                            view=move || {
                                view! { <LoginAndSignup login_action=login/> }
                            }
                        />

                        <ProtectedRoute
                            path="download"
                            view=move || {
                                view! { <Download/> }
                            }

                            condition=|| { true }

                            redirect_path=""
                        />
                        <Route path="email_verification" view=EmailVerification/>
                        <Route path="email_verification/:uuid" view=EmailVerificationAttempt/>
                        <Route path="checkout" view=Checkout/>
                        <Route path="checkout/cancel" view=CheckoutCancel/>
                        <Route path="checkout/success" view=CheckoutSuccess/>
                    </Routes>
                </main>
                <Footer/>
            </body>
        </Router>
    }
}

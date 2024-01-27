use std::error::Error;

use crate::{
    common::{footer::Footer, header::Header},
    error_template::{AppError, ErrorTemplate},
    pages::{
        about::AboutPage, community::CommunityPage, credits::CreditsPage,
        email_verification::EmailVerification,
        email_verification_attempt::EmailVerificationAttempt, home::HomePage,
        login_and_signup::LoginAndSignupPage, support_faq::SupportFAQPage,
        terms_and_conditions::TermsAndConditionsPage,
    },
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
                        <Route path="log_in" view=LoginAndSignupPage/>
                        <Route path="email_verification" view=EmailVerification/>
                        <Route path="email_verification/:email_uuid" view=EmailVerificationAttempt/>
                    </Routes>
                </main>
                <Footer/>
            </body>
        </Router>
    }
}


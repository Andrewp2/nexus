use std::error::Error;

use crate::{
    common::{footer::Footer, header::Header},
    error_template::{AppError, ErrorTemplate},
    pages::{
        about::About, checkout::Checkout, checkout_cancel::CheckoutCancel,
        checkout_success::CheckoutSuccess, community::Community, credits::Credits,
        email_verification::EmailVerification,
        email_verification_attempt::EmailVerificationAttempt, home::Home,
        login_and_signup::LoginAndSignup, support_faq::SupportFAQ,
        terms_and_conditions::TermsAndConditions,
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

        <script src="https://js.stripe.com/v3/"></script>

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
                        <Route path="" view=Home/>
                        <Route path="about" view=About/>
                        <Route path="community" view=Community/>
                        <Route path="terms_and_conditions" view=TermsAndConditions/>
                        <Route path="credits" view=Credits/>
                        <Route path="support" view=SupportFAQ/>
                        <Route path="log_in" view=LoginAndSignup/>
                        <Route path="email_verification" view=EmailVerification/>
                        <Route path="email_verification/:email_uuid" view=EmailVerificationAttempt/>
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


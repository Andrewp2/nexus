use crate::{
    common::{footer::Footer, header::Header},
    error_template::{AppError, ErrorTemplate},
    pages::{
        about::About, checkout::Checkout, checkout_cancel::CheckoutCancel,
        checkout_success::CheckoutSuccess, community::Community, credits::Credits,
        download::Download, email_verification::EmailVerification,
        email_verification_attempt::EmailVerificationAttempt,
        end_user_license_agreement::EndUserLicenseAgreement, home::Home,
        login_and_signup::LoginAndSignup, support_faq::SupportFAQ,
    },
};
use leptos::{component, create_signal, view, Errors, IntoView};
use leptos_meta::{provide_meta_context, Stylesheet, Title};
use leptos_router::{ProtectedRoute, Route, Router, Routes};

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let (logged_in, set_logged_in) = create_signal(false);
    let (bought_game, set_bought_game) = create_signal(false);

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
            <body class="min-h-screen font-['Roboto_Slab'] flex flex-col bg-primary-color text-white m-0 p-0">
                <Header logged_in=logged_in set_logged_in=set_logged_in/>
                <main class="flex-1 p-10 bg-[#background-color] backdrop-blur-md">
                    <Routes>
                        <Route path="" view=Home/>
                        <Route path="about" view=About/>
                        <Route path="community" view=Community/>
                        <Route path="terms_of_service" view=EndUserLicenseAgreement/>
                        <Route path="credits" view=Credits/>
                        <Route path="support" view=SupportFAQ/>
                        <Route
                            path="log_in"
                            view=move || {
                                view! {
                                    <LoginAndSignup
                                        logged_in=logged_in
                                        set_logged_in=set_logged_in
                                    />
                                }
                            }
                        />

                        <ProtectedRoute
                            path="download"
                            view=move || {
                                view! {
                                    <Download logged_in=logged_in set_logged_in=set_logged_in/>
                                }
                            }

                            condition=|| { true }

                            redirect_path=""
                        />

                        // <Route
                        // path="download"
                        // view=move || {
                        // view! {
                        // <Download logged_in=logged_in set_logged_in=set_logged_in/>
                        // }
                        // }
                        // />

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


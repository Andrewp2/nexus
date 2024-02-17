use leptos::{component, view, IntoView};

#[component]
pub fn EndUserLicenseAgreement() -> impl IntoView {
    view! {
        <h2>"End User License Agreement"</h2>
        <h2>"$GAME Terms of Service"</h2>
        <h3>"General"</h3>
        <ul>
            <li>
                "By downloading, installing, opening or otherwise using $GAME or any other connected services or products you agree to be bound by these Terms of Service which constitute an agreement with $COMPANY_NAME"
            </li>
            <li>
                "If there is anything you are wondering about in connection with the use of $GAME or with these Terms of Service, do not hesitate to contact me at support@mysite.com"
            </li>
        </ul>
        <p>This is where the TOS and EULA will go once they are created.</p>
        <p>
            In the meantime, understand I do not take responsibility for any harms that may come to the player while playing.
        </p>
        <p>You are allowed to stream this game</p>
    }
}


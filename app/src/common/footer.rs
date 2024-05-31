use leptos::{component, view, IntoView};
use leptos_router::A;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="h-12 mt-auto text-sm flex items-center justify-between p-5 bg-white/5 border-white/10 border-t-2">
            <div class="flex items-center justify-center mx-auto space-x-4">
                "Copyright 2023-2024 Project Glint LLC." <br/>
                <A
                    href="terms_of_service"
                    class="text-t-color p-1.5 rounded-md hover:bg-hover-accent-color glow-hover"
                >
                    Terms of Service
                </A>
                <A
                    href="credits"
                    class="text-t-color p-1.5 rounded-md hover:bg-hover-accent-color glow-hover"
                >
                    Credits
                </A>
                <A
                    href="support"
                    class=" text-t-color p-1.5 rounded-md hover:bg-hover-accent-color glow-hover"
                >
                    Support / FAQ
                </A>
            </div>
        </footer>
    }
}

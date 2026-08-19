use std::ops::Not;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_oidc::LogoutLink;
use opendut_auth::public::Authentication;
use opendut_auth::types::ROLE_OPENDUT_USER;
use crate::components::LoadingSpinner;
use crate::app::{use_app_globals, AppGlobals, AppGlobalsError};
use crate::user::{UserAuthenticationSignal};

#[must_use]
#[component(transparent)]
pub fn Initialized(
    app_globals: AppGlobalsResource,
    children: ChildrenFn,
    #[prop(optional)] _groups: Vec<String>,
    #[prop(optional)] _roles: Vec<String>,
    #[prop(optional, default = true)] authentication_required: bool,
) -> impl IntoView {

    let children = StoredValue::new(children);

    view! {
        <Suspense
            fallback=LoadingSpinner
        >
            {move || Suspend::new(async move {
                let app_globals_result = app_globals.await;

                match app_globals_result {
                    Ok(app_globals) => {
                        provide_context(app_globals);

                        Either::Right(view! {
                            <InitializedAndAuthenticated
                                authentication_required=authentication_required
                            >
                                { children.read_value()() }
                            </InitializedAndAuthenticated>
                        })
                    }
                    Err(error) => {
                        tracing::error!("Error while constructing app globals: {}", error);
                        Either::Left(
                            view! { <FallbackMessage message=""/> }
                        )
                    }
                }
            })}
        </Suspense>
    }
}

#[component]
fn InitializedAndAuthenticated(
    children: ChildrenFn,
    #[prop(optional)] _groups: Vec<String>,
    #[prop(optional)] _roles: Vec<String>,
    #[prop(optional, default = true)] authentication_required: bool,
) -> impl IntoView {

    let children = StoredValue::new(children);

    match use_app_globals().auth {
        Authentication::Disabled => {
            Either::Left(children.read_value()().into_any())
        }
        Authentication::Enabled(auth) => {
            let user = use_context::<UserAuthenticationSignal>().expect("UserAuthenticationSignal should be provided in the context.");

            let is_authenticated = move || { auth.get().is_authenticated() };

            // Show component if context is initialized and either the user is authenticated or no authentication is needed.
            let show_component = move || {
                is_authenticated() || authentication_required.not()
            };

            let has_required_role = move || {
                user.get().has_role(ROLE_OPENDUT_USER).unwrap_or(false)
            };

            Either::Right(view! {
                <Show
                    when=show_component
                    fallback=|| view! { <FallbackMessage message="You are currently not logged in."/> }
                >
                    <Show
                        when=has_required_role
                        fallback=|| view! { <MissingPermissions/> }
                    >
                        {children.read_value()()}
                    </Show>
                </Show>
            })
        }
    }
}

#[component]
fn FallbackMessage(message: &'static str) -> impl IntoView {
    view! {
        <p>
            <div class="columns is-full">
                <div class="column">
                    <h1 class="title is-5 has-text-centered">{ message }</h1>
                </div>
            </div>
        </p>
    }
}

#[component]
fn MissingPermissions() -> impl IntoView {
    let user = use_context::<UserAuthenticationSignal>().expect("UserAuthenticationSignal should be provided in the context.");
    let username = move || user.get().username();

    view! {
        <div class="columns is-centered">
            <div class="column is-half">
                <article class="message is-warning">
                    <div class="message-header">
                        <p>"Missing Permissions"</p>
                    </div>
                    <div class="message-body">
                        <p class="mb-4">
                            "You are logged in as " <strong>{ username }</strong>
                            " but your account does not have the required permissions to access openDuT."
                        </p>
                        <p class="mb-4">
                            "Please contact your administrator to request the necessary role assignment."
                        </p>
                        <LogoutLink class="button is-warning">
                            <span class="is-size-6">"Sign out"</span>
                        </LogoutLink>
                    </div>
                </article>
            </div>
        </div>
    }
}

pub type AppGlobalsResource = LocalResource<Result<AppGlobals, AppGlobalsError>>;

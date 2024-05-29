use leptos::{component, create_memo, create_rw_signal, create_slice, IntoView, RwSignal, SignalUpdate, SignalWith, SignalWithUntracked, view};
use opendut_types::peer::executor::{Engine};

use crate::components::UserInputValue;
use crate::peers::configurator::tabs::executor::executor_panel::ExecutorPanel;
use crate::peers::configurator::types::{EMPTY_CONTAINER_IMAGE_ERROR_MESSAGE, UserPeerConfiguration, UserPeerExecutor};

mod executor_panel;

#[component]
pub fn ExecutorTab(peer_configuration: RwSignal<UserPeerConfiguration>) -> impl IntoView {
    view! {
        <div>
            <ExecutorTable peer_configuration />
        </div>
    }
}

#[component]
fn ExecutorTable(peer_configuration: RwSignal<UserPeerConfiguration>) -> impl IntoView {

    let (executors, executors_setter) = create_slice(peer_configuration,
        |peer_configuration| {
            Clone::clone(&peer_configuration.executors)
        },
        |peer_configuration, value| {
            peer_configuration.executors = value
        }
    );

    let user_input_string = move |user_input| {
        match user_input {
            UserInputValue::Left(_) => String::new(),
            UserInputValue::Right(value) => value.to_owned(),
            UserInputValue::Both(_, value) => value.to_owned(),
        }
    };

    let on_executor_delete = move |container_image: String| {
        let remaining_executors = executors.with_untracked(|executors| {
            executors.iter()
                .filter(|executor| executor.with_untracked(|executor| 
                    match executor {
                        UserPeerExecutor::Container { image, .. } => {
                            let image = user_input_string(Clone::clone(image));
                            image != container_image
                        }
                    }
                ))
                .cloned()
                .collect::<Vec<_>>()
        });
        executors_setter.set(remaining_executors)
    };

    

    let panels = create_memo(move |_| {
        executors.with(|executors| {
            executors.iter()
                .cloned()
                .map(|executor| {
                    view! {
                        <ExecutorPanel executor on_delete=on_executor_delete  />
                    }
                })
                .collect::<Vec<_>>()
        })
    });

    view! {
        <div>
            <div>
                { panels }
            </div>
             <div class="mt-5">
                <div
                    class="dut-panel-ghost has-text-success px-4 py-3 is-clickable is-flex is-justify-content-center"
                    on:click=move |_| {
                        peer_configuration.update(|peer_configuration| {
                            let user_peer_executor = create_rw_signal(
                                UserPeerExecutor::Container {
                                    engine: Engine::Podman,
                                    name: UserInputValue::Right(String::from("")),
                                    image: UserInputValue::Left(String::from(EMPTY_CONTAINER_IMAGE_ERROR_MESSAGE)),
                                    volumes: vec![],
                                    devices: vec![],
                                    envs: vec![],
                                    ports: vec![],
                                    command: UserInputValue::Right(String::from("")),
                                    args: vec![],
                                    results_url: UserInputValue::Right(String::from("")),
                                    is_collapsed: false
                                }
                            );
                            peer_configuration.executors.push(user_peer_executor);
                        });
                    }
                >
                    <span><i class="fa-solid fa-circle-plus"></i></span>
                </div>
            </div>
        </div>
    }
}

use leptos::prelude::*;
use opendut_types::peer::executor::container::Engine;
use opendut_types::peer::executor::ExecutorId;
use crate::components::UserInputValue;
use crate::peers::configurator::tabs::executor::executor_panel::ExecutorPanel;
use crate::peers::configurator::types::{EMPTY_CONTAINER_IMAGE_ERROR_MESSAGE, UserPeerConfiguration, UserPeerExecutor, UserPeerExecutorKind};

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

    let on_executor_delete = move |executor_id_to_delete: ExecutorId| {
        let remaining_executors = executors.with_untracked(|executors| {
            executors.iter()
                .filter(|executor| executor.with_untracked(|executor|
                    executor.id != executor_id_to_delete
                ))
                .cloned()
                .collect::<Vec<_>>()
        });
        executors_setter.set(remaining_executors)
    };


    let panels = move || {
        executors.with(|executors| {
            executors.iter()
                .cloned()
                .map(|executor| {
                    view! {
                        <ExecutorPanel executor on_delete=on_executor_delete />
                    }
                })
                .collect::<Vec<_>>()
        })
    };

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
                            let user_peer_executor = RwSignal::new(
                                UserPeerExecutor {
                                    id: ExecutorId::random(),
                                    kind: UserPeerExecutorKind::Container {
                                        engine: Engine::Podman,
                                        name: UserInputValue::Right(String::from("")),
                                        image: UserInputValue::Left(String::from(EMPTY_CONTAINER_IMAGE_ERROR_MESSAGE)),
                                        volumes: vec![],
                                        devices: vec![],
                                        envs: vec![],
                                        ports: vec![],
                                        command: UserInputValue::Right(String::from("")),
                                        args: vec![],
                                    },
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

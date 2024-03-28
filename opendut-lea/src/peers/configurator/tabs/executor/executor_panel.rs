use leptos::{component, create_read_slice, create_rw_signal, create_slice, event_target_value, IntoView, RwSignal,  SignalGet, SignalGetUntracked, SignalUpdate, SignalWith, SignalWithUntracked, view};
use opendut_types::peer::executor::{ContainerCommand, ContainerCommandArgument, ContainerDevice, ContainerImage, ContainerName, ContainerPortSpec, ContainerVolume, Engine, IllegalContainerImage};
use strum::IntoEnumIterator;

use crate::components::{ButtonColor, ButtonSize, ButtonState, ConfirmationButton, FontAwesomeIcon, IconButton, Toggled, UserInput, UserInputValue, VectorUserInput};
use crate::peers::configurator::types::{EMPTY_CONTAINER_IMAGE_ERROR_MESSAGE, UserContainerEnv, UserPeerExecutor};
use crate::util::NON_BREAKING_SPACE;

#[component]
pub fn ExecutorPanel<OnDeleteFn>(
    executor: RwSignal<UserPeerExecutor>,
    on_delete: OnDeleteFn
) -> impl IntoView
where
    OnDeleteFn: Fn(String) + 'static
{
    let is_collapsed = move || { 
        match executor.get() {
            UserPeerExecutor::Container { is_collapsed, .. } => { is_collapsed }
        }
    };
    
    view! {
        <div class="panel is-light">
            <ExecutorPanelHeading executor on_delete/>
            <div class="panel-block" class=("is-hidden", is_collapsed)>
                <div class="container">
                    <ExecutorEngineInput executor />
                    <ExecutorContainerNameInput executor />
                    <ExecutorContainerImageInput executor />
                    <ExecutorContainerVolumesInput executor />
                    <ExecutorContainerDevicesInput executor />
                    <ExecutorContainerEnvsInput executor />
                    <ExecutorContainerPortsInput executor />
                    <ExecutorContainerCommandInput executor />
                    <ExecutorContainerArgsInput executor />
                </div>
            </div>
        </div>
    }
}

#[component]
fn ExecutorPanelHeading<OnDeleteFn>(
    executor: RwSignal<UserPeerExecutor>,
    on_delete: OnDeleteFn
) -> impl IntoView
where
    OnDeleteFn: Fn(String) + 'static
{
    let (is_collapsed, set_is_collapsed) = create_slice(executor,
        move |executor| {
            match executor {
                UserPeerExecutor::Container { is_collapsed, .. } => { Clone::clone(is_collapsed) }
            }
        },
        move |executor, value| {
            match executor {
                UserPeerExecutor::Container { is_collapsed, .. } => { *is_collapsed = value; }
            }
        }
    );

    let collapse_button_icon = is_collapsed.derive_toggled(FontAwesomeIcon::ChevronDown, FontAwesomeIcon::ChevronUp);


    let executor_heading = create_read_slice(executor,
       |executor| {
           match executor {
               UserPeerExecutor::Container { image, name, .. } => {
                   let name = match name {
                       UserInputValue::Left(_) => String::from(""),
                       UserInputValue::Right(ref value) => value.to_owned(),
                       UserInputValue::Both(_, ref value) => value.to_owned()
                   };
                   if name.is_empty() {
                       match image {
                           UserInputValue::Left(_) => String::from(""),
                           UserInputValue::Right(ref value) => value.to_owned(),
                           UserInputValue::Both(_, ref value) => value.to_owned()
                       }
                   } else {
                      name
                   }
               }
           }
       }
    );

    view! {
        <div class="panel-heading px-2 py-3">
            <div class="is-flex is-justify-content-space-between is-align-items-center">
                <div>
                    <span class="pr-1">
                        <IconButton
                            icon=collapse_button_icon
                            color=ButtonColor::Light
                            size=ButtonSize::Small
                            state=ButtonState::Enabled
                            label="Show Device Details"
                            on_action=move || set_is_collapsed.set(!is_collapsed.get_untracked())
                        />
                    </span>
                    <span class="is-size-5 has-text-weight-bold pr-3">"Container:"</span>
                    <span class="is-size-5 has-text-weight-bold">{ executor_heading }</span>
                </div>
                <div>
                    <ConfirmationButton
                        icon=FontAwesomeIcon::TrashCan
                        color=ButtonColor::Light
                        size=ButtonSize::Small
                        state=ButtonState::Enabled
                        label="Delete Executor?"
                        on_conform=move || {
                            match executor.get_untracked() {
                                UserPeerExecutor::Container{ image, .. } => {
                                     let image = match image {
                                        UserInputValue::Left(_) => String::new(),
                                        UserInputValue::Right(value) => value.to_owned(),
                                        UserInputValue::Both(_, value) => value.to_owned(),
                                    };
                                    on_delete(image)
                                }
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}

#[component]
fn ExecutorEngineInput<>(
    executor: RwSignal<UserPeerExecutor>
) -> impl IntoView {

    let (getter, setter) = create_slice(executor,
        move |executor| {
            match executor {
                UserPeerExecutor::Container { engine, .. } => { Clone::clone(engine) }
            }
        },
        move |executor, value| {
            match executor {
                UserPeerExecutor::Container { engine, .. } => { *engine = value; }
            }
        }
    );
    
    let value = getter.with(|engine| match engine {
        Engine::Docker => { "Docker" }
        Engine::Podman => { "Podman" }
    });

    let dropdown_options = move || {
            Engine::iter()
                .map(|engine| {
                    let engine_value = engine.to_string();
                    if engine_value == value {
                        view! {
                            <option selected>{engine_value}</option>
                        }
                    } else {
                        view! {
                            <option>{engine_value}</option>
                        }
                    }
                })
                .collect::<Vec<_>>()
    };

    view! {
        <div class="field pb-3">
            <label class="label">Engine</label>
            <div class="control">
                <div class="select"
                    on:change=move |ev| {
                        let target_value = event_target_value(&ev);
                        match target_value.as_str() {
                            "Docker" => { setter.set(Engine::Docker); }
                            "Podman" => { setter.set(Engine::Podman); }
                            _ => {}
                        };
                    }>
                    <select>
                        { dropdown_options }
                    </select>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ExecutorContainerNameInput(
    executor: RwSignal<UserPeerExecutor>,
) -> impl IntoView {

    let (getter, setter) = create_slice(executor,
        move |executor| {
            match executor {
                UserPeerExecutor::Container { name, .. } => { Clone::clone(name) }
            }
        },
        move |executor, value| {
            match executor {
                UserPeerExecutor::Container { name, .. } => { *name = value; }
            }
        }
    );

    let validator = |input: String| {
        match ContainerName::try_from(input.clone()) {
            Ok(name) => {
                UserInputValue::Right(String::from(name))
            }
            Err(cause) => {
                UserInputValue::Both(cause.to_string(), input)
            }
        }
    };

    view! {
        <UserInput
            getter
            setter
            label="Container Name"
            placeholder=""
            validator
        />
    }
}

#[component]
fn ExecutorContainerImageInput(
    executor: RwSignal<UserPeerExecutor>,
) -> impl IntoView {

    let (getter, setter) = create_slice(executor,
        move |executor| {
            match executor {
                UserPeerExecutor::Container { image, .. } => { Clone::clone(image) }
            }
        },
        move |executor, value| {
            match executor {
                UserPeerExecutor::Container { image, .. } => { *image = value; }
            }
        }
    );

    let validator = |input: String| {
        match ContainerImage::try_from(input) {
            Ok(name) => {
                UserInputValue::Right(String::from(name.value()))
            }
            Err(cause) => {
                match cause {
                    IllegalContainerImage::TooShort { value, .. } => {
                        UserInputValue::Both(String::from(EMPTY_CONTAINER_IMAGE_ERROR_MESSAGE), value)
                    }
                }
            }
        }
    };

    view! {
        <UserInput
            getter
            setter
            label="Image"
            placeholder=""
            validator
        />
    }
}

#[component]
fn ExecutorContainerVolumesInput(
    executor: RwSignal<UserPeerExecutor>,
) -> impl IntoView {

    let (getter, setter) = create_slice(executor,
        move |executor| {
            match executor {
                UserPeerExecutor::Container { volumes, .. } => { Clone::clone(volumes) }
            }
        },
        move |executor, value| {
            match executor {
                UserPeerExecutor::Container { volumes, .. } => { *volumes = value; }
            }
        }
    );

    let validator = |input: String| {
        match ContainerVolume::try_from(input.clone()) {
            Ok(volume) => {
                UserInputValue::Right(String::from(volume.value()))
            }
            Err(cause) => {
                UserInputValue::Both(cause.to_string(), input)
            }
        }
    };
    
    let on_add_volume = move || {
        executor.update(|executor| {
            let volume = create_rw_signal(
                UserInputValue::Left(String::from("Container volume must not be empty."))
            );
            match executor {
                UserPeerExecutor::Container{ volumes, .. } => {
                    volumes.push(volume);
                }
            }
        })
    };
    
    view! {
        <VectorUserInput
            getter
            setter
            label="Volumes"
            placeholder=""
            validator
            on_add=on_add_volume
            delete_label="Delete Volume?"
        />
    }
}

#[component]
fn ExecutorContainerDevicesInput(
    executor: RwSignal<UserPeerExecutor>,
) -> impl IntoView {

    let (getter, setter) = create_slice(executor,
        move |executor| {
            match executor {
                UserPeerExecutor::Container { devices, .. } => { Clone::clone(devices) }
            }
        },
        move |executor, value| {
            match executor {
                UserPeerExecutor::Container { devices, .. } => { *devices = value; }
            }
        }
    );

    let validator = |input: String| {
        match ContainerDevice::try_from(input.clone()) {
            Ok(device) => {
                UserInputValue::Right(String::from(device.value()))
            }
            Err(cause) => {
                UserInputValue::Both(cause.to_string(), input)
            }
        }
    };

    let on_add_device = move || {
        executor.update(|executor| {
            let device = create_rw_signal(
                UserInputValue::Left(String::from("Container device must not be empty."))
            );
            match executor {
                UserPeerExecutor::Container{ devices, .. } => {
                    devices.push(device);
                }
            }
        })
    };

    view! {
        <VectorUserInput
            getter
            setter
            label="Devices"
            placeholder=""
            validator
            on_add=on_add_device
            delete_label="Delete Device?"
        />
    }
}

#[component]
fn ExecutorContainerPortsInput(
    executor: RwSignal<UserPeerExecutor>,
) -> impl IntoView {

    let (getter, setter) = create_slice(executor,
        move |executor| {
            match executor {
                UserPeerExecutor::Container { ports, .. } => { Clone::clone(ports) }
            }
        },
        move |executor, value| {
            match executor {
                UserPeerExecutor::Container { ports, .. } => { *ports = value; }
            }
        }
    );

    let validator = |input: String| {
        match ContainerPortSpec::try_from(input.clone()) {
            Ok(port) => {
                UserInputValue::Right(String::from(port.value()))
            }
            Err(cause) => {
                UserInputValue::Both(cause.to_string(), input)
            }
        }
    };

    let on_add_port = move || {
        executor.update(|executor| {
            let port = create_rw_signal(
                UserInputValue::Left(String::from("Container port specification must not be empty."))
            );
            match executor {
                UserPeerExecutor::Container{ ports, .. } => {
                    ports.push(port);
                }
            }
        })
    };

    view! {
        <VectorUserInput
            getter
            setter
            label="Ports"
            placeholder=""
            validator
            on_add=on_add_port
            delete_label="Delete Port?"
        />
    }
}

#[component]
fn ExecutorContainerCommandInput(
    executor: RwSignal<UserPeerExecutor>,
) -> impl IntoView {

    let (getter, setter) = create_slice(executor,
        move |executor| {
            match executor {
                UserPeerExecutor::Container { command, .. } => { Clone::clone(command) }
            }
        },
        move |executor, value| {
            match executor {
                UserPeerExecutor::Container { command, .. } => { *command = value; }
            }
        }
    );

    let validator = |input: String| {
        match ContainerCommand::try_from(input.clone()) {
            Ok(command) => {
                UserInputValue::Right(String::from(command))
            }
            Err(cause) => {
                UserInputValue::Both(cause.to_string(), input)
            }
        }
    };

    view! {
        <UserInput
            getter
            setter
            label="Command"
            placeholder=""
            validator
        />
    }
}

#[component]
fn ExecutorContainerArgsInput(
    executor: RwSignal<UserPeerExecutor>,
) -> impl IntoView {

    let (getter, setter) = create_slice(executor,
        move |executor| {
            match executor {
                UserPeerExecutor::Container { args, .. } => { Clone::clone(args) }
            }
        },
        move |executor, value| {
            match executor {
                UserPeerExecutor::Container { args, .. } => { *args = value; }
            }
        }
    );

    let validator = |input: String| {
        match ContainerCommandArgument::try_from(input.clone()) {
            Ok(arg) => {
                UserInputValue::Right(String::from(arg.value()))
            }
            Err(cause) => {
                UserInputValue::Both(cause.to_string(), input)
            }
        }
    };

    let on_add_arg = move || {
        executor.update(|executor| {
            let arg = create_rw_signal(
                UserInputValue::Left(String::from("Container command argument must not be empty."))
            );
            match executor {
                UserPeerExecutor::Container{ args, .. } => {
                    args.push(arg);
                }
            }
        })
    };

    view! {
        <VectorUserInput
            getter
            setter
            label="Arguments"
            placeholder=""
            validator
            on_add=on_add_arg
            delete_label="Delete Arg?"
        />
    }
}

#[component]
fn ExecutorContainerEnvsInput(
    executor: RwSignal<UserPeerExecutor>,
) -> impl IntoView {

    let (getter, setter) = create_slice(executor,
        move |executor| {
            match executor {
                UserPeerExecutor::Container { envs, .. } => { Clone::clone(envs) }
            }
        },
        move |executor, value| {
            match executor {
                UserPeerExecutor::Container { envs, .. } => { *envs = value; }
            }
        }
    );

    let on_add_env = move || {
        executor.update(|executor| {
            let env = create_rw_signal(
                UserContainerEnv {
                    name: UserInputValue::Left(String::from("Container environment variable name must not be empty.")),
                    value: UserInputValue::Right(String::from(""))
                }
            );
            match executor {
                UserPeerExecutor::Container{ envs, .. } => {
                    envs.push(env);
                }
            }
        })
    };

    let on_env_delete = move |env_name: String| {
        let remaining_envs= getter.with_untracked(|envs| {
            envs.iter()
                .filter(|envs| envs.with_untracked(|env| {
                    let value = match env.clone().name {
                        UserInputValue::Left(_) => String::new(),
                        UserInputValue::Right(value) => value.to_owned(),
                        UserInputValue::Both(_, value) => value.to_owned(),
                    };
                    value != env_name
                }))
                .cloned()
                .collect::<Vec<_>>()
        });
        setter.set(remaining_envs)
    };

    let name_validator = |input: String| {
        match input.is_empty() {
            true => {
                UserInputValue::Both(String::from("Container env name must not be empty."), input)
            }
            false => {
                UserInputValue::Right(input)
            }
        }
    };
    

    let panels = move || {
        getter.with(|envs| {
            envs.iter()
                .cloned()
                .map(|env| {
                    let (name_getter, name_setter) = create_slice(env,
                        |env| {
                            Clone::clone(&env.name)
                        },
                        |env, value| {
                            env.name = value;
                        }
                    );

                    let (value_getter, value_setter) = create_slice(env,
                          |env| {
                              Clone::clone(&env.value)
                          },
                          |env, value| {
                              env.value = value;
                          }
                    );

                    let name_text = move || {
                        name_getter.with(|input| match input {
                            UserInputValue::Left(_) => String::new(),
                            UserInputValue::Right(value) => value.to_owned(),
                            UserInputValue::Both(_, value) => value.to_owned(),
                        })
                    };
                    
                    let value_text = move || {
                        value_getter.with(|input| match input {
                            UserInputValue::Left(_) => String::new(),
                            UserInputValue::Right(value) => value.to_owned(),
                            UserInputValue::Both(_, value) => value.to_owned(),
                        })
                    };

                    let help_name_text = move || {
                        name_getter.with(|input| match input {
                            UserInputValue::Right(_) => String::from(NON_BREAKING_SPACE),
                            UserInputValue::Left(error) => error.to_owned(),
                            UserInputValue::Both(error, _) => error.to_owned(),
                        })
                    };

                    let help_value_text = move || {
                        value_getter.with(|input| match input {
                            UserInputValue::Right(_) => String::from(NON_BREAKING_SPACE),
                            UserInputValue::Left(error) => error.to_owned(),
                            UserInputValue::Both(error, _) => error.to_owned(),
                        })
                    };

                    view! {
                        <div>
                            <div class="control is-flex is-align-items-center">
                                <label class="label has-text-weight-normal mb-0">name</label>
                                <input
                                    class="input mr-2"
                                    type="text"
                                    aria-label="EnvName"
                                    placeholder=""
                                    prop:value={ name_text }
                                    on:input=move |ev| {
                                        let validated_value = name_validator(event_target_value(&ev));
                                        name_setter.set(validated_value);
                                    }
                                />
                                <label class="label has-text-weight-normal mb-0">value</label>
                                <input
                                    class="input"
                                    type="text"
                                    aria-label="EnvValue"
                                    placeholder=""
                                    prop:value={ value_text }
                                    on:input=move |ev| {
                                        let target_value = event_target_value(&ev);
                                        value_setter.set(UserInputValue::Right(target_value));
                                    }
                                />
                                <ConfirmationButton
                                    icon=FontAwesomeIcon::TrashCan
                                    color=ButtonColor::Light
                                    size=ButtonSize::Normal
                                    state=ButtonState::Enabled
                                    label="Delete Env?"
                                    on_conform=move || {
                                        let name = match name_getter.get_untracked() {
                                            UserInputValue::Left(_) => String::new(),
                                            UserInputValue::Right(value) => value.to_owned(),
                                            UserInputValue::Both(_,value) => value.to_owned(),
                                        };
                                        on_env_delete(name)
                                    }
                                />
                            </div>
                            <div class="help has-text-danger">
                                { help_name_text }
                                { move || { String::from(NON_BREAKING_SPACE)} }
                                { help_value_text }
                            </div>
                        </div>
                    }
                })
                .collect::<Vec<_>>()
        })
    };

    view! {
        <div>
             <div class="field">
                <label class="label">Environment Variables</label>
                { panels }
            </div>
            <div>
                <div
                    class="dut-panel-ghost has-text-success px-4 py-3 is-clickable is-flex is-justify-content-center mb-5"
                    on:click=move |_| {
                       on_add_env()
                    }
                >
                    <span><i class="fa-solid fa-circle-plus"></i></span>
                </div>
            </div>
        </div>
    }
}

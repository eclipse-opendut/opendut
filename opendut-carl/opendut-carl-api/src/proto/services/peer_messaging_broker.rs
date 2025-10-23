use opendut_model::conversion;
use opendut_model::proto::ConversionResult;

tonic::include_proto!("opendut.carl.services.peer_messaging_broker");

conversion! {
    type Model = crate::carl::broker::UpstreamMessage;
    type Proto = Upstream;

    fn from(value: Model) -> Proto {
        let context: Option<TracingContext> = value.context.map(|c| c.into());
        let message = match value.payload {
            crate::carl::broker::UpstreamMessagePayload::Ping => {
                upstream::Message::Ping(Ping { })
            },
            crate::carl::broker::UpstreamMessagePayload::PeerConfigurationState(_) => todo!()
        };

        Upstream { context, message: Some(message) }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let context: Option<crate::carl::broker::TracingContext> = value.context
            .map(crate::carl::broker::TracingContext::try_from)
            .transpose()?;
        let proto_message = extract!(value.message)?;
        let payload = match proto_message {
            upstream::Message::Ping(_) => {
                crate::carl::broker::UpstreamMessagePayload::Ping
            },
            upstream::Message::PeerConfigurationState(state) => {
                crate::carl::broker::UpstreamMessagePayload::PeerConfigurationState(
                    state.try_into()?
                )
            },
        };

        Ok(Model { context, payload })
    }
}

conversion! {
    type Model = opendut_model::peer::configuration::api::EdgePeerConfigurationState;
    type Proto = Upstream;

    fn from(value: Model) -> Proto {
        let state = opendut_model::proto::peer::configuration::api::PeerConfigurationState::from(value);
        let message = upstream::Message::PeerConfigurationState(state);
        Upstream {
            context: None,
            message: Some(message),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let proto_message = extract!(value.message)?;
        match proto_message {
            upstream::Message::PeerConfigurationState(state) => Ok(state.try_into()?),
            _ => Err(ErrorBuilder::message("This is not a peer configuration state message.")),
        }
    }
}

conversion! {
    type Model = crate::carl::broker::DownstreamMessage;
    type Proto = Downstream;

    fn from(value: Model) -> Proto {
        let context: Option<TracingContext> = value.context.map(|c| c.into());
        let message = match value.payload {
            crate::carl::broker::DownstreamMessagePayload::Pong => {
                downstream::Message::Pong(Pong { })
            }
            crate::carl::broker::DownstreamMessagePayload::ApplyPeerConfiguration(apply) => {
                let apply = ApplyPeerConfiguration::from(*apply);
                downstream::Message::ApplyPeerConfiguration(apply)
            }
            crate::carl::broker::DownstreamMessagePayload::DisconnectNotice => {
                downstream::Message::DisconnectNotice(DisconnectNotice { })
            }
        };

        Downstream { context, message: Some(message) }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let context: Option<crate::carl::broker::TracingContext> = value.context
            .map(crate::carl::broker::TracingContext::try_from)
            .transpose()?;
        let proto_message = extract!(value.message)?;
        let payload = match proto_message {
            downstream::Message::Pong(_) => {
                crate::carl::broker::DownstreamMessagePayload::Pong
            }
            downstream::Message::ApplyPeerConfiguration(apply) => {
                let apply_peer_configuration: crate::carl::broker::ApplyPeerConfiguration = apply.try_into()?;
                crate::carl::broker::DownstreamMessagePayload::ApplyPeerConfiguration(Box::new(apply_peer_configuration))
            }
            downstream::Message::DisconnectNotice(_) => {
                crate::carl::broker::DownstreamMessagePayload::DisconnectNotice
            }
        };

        Ok(Model { context, payload })
    }
}

conversion! {
    type Model = crate::carl::broker::ApplyPeerConfiguration;
    type Proto = ApplyPeerConfiguration;
    
    fn from(value: Model) -> Proto {
        ApplyPeerConfiguration {
            old_configuration: Some(value.old_configuration.into()),
            configuration: Some(value.configuration.into()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let old_configuration = opendut_model::peer::configuration::OldPeerConfiguration::try_from(extract!(value.old_configuration)?)?;
        let configuration = opendut_model::peer::configuration::PeerConfiguration::try_from(extract!(value.configuration)?)?;
        Ok(Model {
            old_configuration,
            configuration,
        })
    }
}

conversion! {
    type Model = crate::carl::broker::TracingContext;
    type Proto = TracingContext;
    
    fn from(value: Model) -> Proto {
        TracingContext {
            values: value.values,
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Ok(Model {
            values: value.values,
        })
    }
}

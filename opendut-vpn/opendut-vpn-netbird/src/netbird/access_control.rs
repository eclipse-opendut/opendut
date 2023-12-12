use serde::Serialize;
use opendut_types::cluster::ClusterId;

#[derive(Serialize)]
#[serde(rename_all="kebab-case")]
pub(crate) enum RuleFlow {
    Bidirect,
}

pub fn rule_name(cluster_id: ClusterId) -> String {
    format!("opendut-cluster-rule-{cluster_id}")
}

pub fn rule_description(cluster_id: ClusterId) -> String {
    format!("Rule for the openDuT cluster <{cluster_id}>.")
}

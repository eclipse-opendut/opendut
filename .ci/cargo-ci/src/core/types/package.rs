use crate::workspace;

pub use cicero::workspace::Package;


pub trait PackageExt: Sized {
    fn dockerfile_path(&self) -> Option<&'static str>;
    fn applications() -> Vec<Self>;
}
impl PackageExt for cicero::workspace::Package {
    fn dockerfile_path(&self) -> Option<&'static str> {
        if self == &workspace::package::opendut_carl {
            Some(".ci/docker/carl/Dockerfile")
        }
        else if self == &workspace::package::opendut_edgar {
            Some(".ci/docker/edgar/Dockerfile")
        }
        else {
            None
        }
    }

    fn applications() -> Vec<Self> {
        use workspace::package::*;
        vec![opendut_carl, opendut_cleo, opendut_edgar, opendut_lea]
    }
}

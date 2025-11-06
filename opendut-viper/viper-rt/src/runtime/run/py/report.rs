use std::cell::RefCell;
use std::path::PathBuf;
use rustpython_vm::{PyPayload, PyRef, PyResult, VirtualMachine};
use std::rc::Rc;
use opendut_viper_py::report::report::PyReportProperties;
use opendut_viper_py::report::ReportPropertiesCollector;
use crate::run::{ReportProperty, ReportPropertyValue};

pub fn make_report_properties(collector: Rc<RefCell<Vec<ReportProperty>>>, vm: &VirtualMachine) -> PyRef<PyReportProperties> {
    PyReportProperties::new(Box::new(Collector(collector))).into_ref(&vm.ctx)
}

struct Collector(Rc<RefCell<Vec<ReportProperty>>>);

impl ReportPropertiesCollector for Collector {

    fn set_number_property(&self, name: String, value: i64) -> PyResult<()> {
        self.0.borrow_mut().push(ReportProperty {
            name,
            value: ReportPropertyValue::Number(value),
        });
        Ok(())
    }

    fn set_string_property(&self, name: String, value: String) -> PyResult<()> {
        self.0.borrow_mut().push(ReportProperty {
            name,
            value: ReportPropertyValue::String(value),
        });
        Ok(())
    }

    fn set_file_property(&self, value: String) -> PyResult<()> {
        self.0.borrow_mut().push(ReportProperty {
            name: Clone::clone(&value),
            value: ReportPropertyValue::File(PathBuf::from(value)),
        });
        Ok(())
    }
}

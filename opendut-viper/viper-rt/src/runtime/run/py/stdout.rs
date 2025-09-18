use rustpython_vm::builtins::PyStrRef;
use rustpython_vm::{extend_class, py_class, PyObjectRef, PyRef, PyResult, VirtualMachine};
use std::cell::RefCell;
use std::rc::Rc;

pub fn make_stdout_object(
    vm: &VirtualMachine,
    buffer: Rc<RefCell<Option<Vec<String>>>>,
) -> PyObjectRef {
    let ctx = &vm.ctx;
    // there's not really any point to storing this class so that there's a consistent type object,
    // we just want a half-decent repr() output
    let cls = PyRef::leak(py_class!(
        ctx,
        "Stdout",
        vm.ctx.types.object_type.to_owned(),
        {}
    ));
    let write_method = vm.new_method(
        "write",
        cls,
        move |_self: PyObjectRef, data: PyStrRef, _vm: &VirtualMachine| -> PyResult<()> {
            if let Ok(Some(buffer)) = buffer.try_borrow_mut().as_deref_mut() {
                buffer.push(data.as_str().to_owned())
            }
            Ok(())
        },
    );
    let flush_method = vm.new_method("flush", cls, |_self: PyObjectRef| {});
    extend_class!(ctx, cls, {
        "write" => write_method,
        "flush" => flush_method,
    });
    ctx.new_base_object(cls.to_owned(), None)
}

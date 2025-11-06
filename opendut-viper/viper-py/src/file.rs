#![allow(clippy::module_inception)]
use rustpython_vm::pymodule;

#[pymodule]
pub mod file {
    use std::cell::RefCell;
    use std::fs::{File, OpenOptions};
    use std::io::{BufReader, BufRead, Read, BufWriter, Write};
    use rustpython_vm::{pyclass, Py, PyObjectRef, PyPayload, PyRef, PyResult, VirtualMachine};
    use rustpython_vm::function::OptionalArg;
    use rustpython_vm::protocol::PyIterReturn;
    use rustpython_vm::types::{IterNext, Iterable, SelfIter};
    
    #[derive(Debug)]
    pub enum Mode {
        Read,
        Write,
        ReadWrite,
    }
    
    #[pyclass(no_attr, name )]
    #[derive(Debug, PyPayload)]
    pub struct FileHandler {
        buf_reader: RefCell<Option<BufReader<File>>>,
        buf_writer: RefCell<Option<BufWriter<File>>>,
        mode: Mode,
    }

    #[pyclass(with(Iterable, IterNext))]
    #[opendut_viper_pygen::pygen]
    impl FileHandler {

        #[viper(skip)]
        pub fn new(file: String, #[viper(default = "r")] mode: OptionalArg<String>, vm: &VirtualMachine) -> PyResult<Self> {
            
            let mode = match mode.unwrap_or_else(|| String::from("r")).as_str() {
                "w" => Mode::Write,
                "r+" | "w+" => Mode::ReadWrite,
                "r" => Mode::Read,
                value => return Err(vm.new_runtime_error(format!("Unknown Mode: {value}")))
            };

            let opened_file = match mode {
                Mode::Read => File::open(&file),
                Mode::Write | Mode::ReadWrite => {
                    OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(file)
                }
            }.map_err(|err| vm.new_value_error(format!("{err}")))?;

            let (buf_reader, buf_writer) = match mode {
                Mode::Read => (Some(BufReader::new(opened_file)), None),
                Mode::Write => (None, Some(BufWriter::new(opened_file))),
                Mode::ReadWrite => { 
                    (
                        Some(BufReader::new(opened_file.try_clone()
                            .map_err(|err| vm.new_value_error(format!("{err}")))?)),
                        Some(BufWriter::new(opened_file))
                    )
                }
            };

            Ok(
                Self {
                    buf_reader: RefCell::new(buf_reader),
                    buf_writer: RefCell::new(buf_writer),
                    mode
                }
            )
        }
        
        #[pymethod(magic)]
        #[viper(skip)]
        fn enter(this: PyRef<Self>) -> PyResult<PyRef<Self>> {
            Ok(this)
        }

        #[pymethod(magic)]
        #[viper(skip)]
        fn exit(&self, _ty: PyObjectRef, _value: PyObjectRef, _tb: PyObjectRef) -> PyResult<()> {
            Ok(())
        }
        
        #[pymethod]
        fn read(&self, #[viper(skip)] vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            if matches!(self.mode, Mode::Write) {
                return Err(vm.new_runtime_error("Cannot read in write mode!".to_string()));
            }

            let mut content = String::new();
            let mut reader = self.buf_reader.borrow_mut();
            let _ = reader.as_mut().unwrap().read_to_string(&mut content);
            
            Ok(vm.ctx.new_str(content).into())
        }

        #[pymethod]
        fn readline(&self, #[viper(skip)] vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            if matches!(self.mode, Mode::Write) {
                return Err(vm.new_runtime_error("Cannot read in write mode!".to_string()));
            }

            let mut line = String::new();
            let mut reader = self.buf_reader.borrow_mut();
            let bytes_read = reader.as_mut().unwrap().read_line(&mut line)
                .map_err(|err| vm.new_value_error(format!("{err}")))?;

            if bytes_read == 0 {
                Ok(vm.ctx.new_str("").into())
            } else {
                Ok(vm.ctx.new_str(line).into())
            }
        }

        #[pymethod]
        fn readlines(&self, #[viper(skip)] vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            if matches!(self.mode, Mode::Write) {
                return Err(vm.new_runtime_error("Cannot read in write mode!".to_string()));
            }

            let mut reader = self.buf_reader.borrow_mut();
            let reader = reader.as_mut().unwrap();
            let mut lines = Vec::new();
            let mut line = String::new();

            while reader.read_line(&mut line).map_err(|err| vm.new_value_error(format!("{err}")))? > 0 {
                lines.push(vm.ctx.new_str(line.trim_end()).into());
                line.clear();
            }

            Ok(vm.ctx.new_list(lines).into())
        }

        #[pymethod]
        fn write(&self, text: String, #[viper(skip)] vm: &VirtualMachine) -> PyResult<()> {
            if matches!(self.mode, Mode::Read) {
                return Err(vm.new_runtime_error("Cannot write in read mode!".to_string()));
            }

            let mut writer = self.buf_writer.borrow_mut();
            let writer = writer.as_mut().unwrap();

            writer.write_all(text.as_bytes())
                .map_err(|err| vm.new_value_error(format!("{err}")))?;

            writer.flush()
                .map_err(|err| vm.new_value_error(format!("{err}")))?;

            Ok(())
        }

        #[pymethod]
        fn writelines(&self, text: Vec<String>, #[viper(skip)] vm: &VirtualMachine) -> PyResult<()> {
            if matches!(self.mode, Mode::Read) {
                return Err(vm.new_runtime_error("Cannot write in read mode!".to_string()));
            }

            let mut writer = self.buf_writer.borrow_mut();
            let writer = writer.as_mut().unwrap();

            for line in text {
                writer.write_all(line.as_bytes())
                    .and_then(|_| writer.write_all(b"\n"))
                    .map_err(|err| vm.new_value_error(format!("{err}")))?;
            }

            writer.flush()
                .map_err(|err| vm.new_value_error(format!("{err}")))?;

            Ok(())
        }
    }

    impl SelfIter for FileHandler {}


    impl IterNext for FileHandler {
        fn next(this: &Py<Self>, vm: &VirtualMachine) -> PyResult<PyIterReturn<PyObjectRef>> {

            let mut reader_opt = this.buf_reader.borrow_mut();
            
            let reader = match reader_opt.as_mut() {
                Some(reader) => reader,
                None => return Ok(PyIterReturn::StopIteration(None)),
            };

            let mut line = String::new();
            
            match reader.read_line(&mut line) {
                Ok(0) => Ok(PyIterReturn::StopIteration(None)), 
                Ok(_) => Ok(PyIterReturn::Return(vm.ctx.new_str(line).into())),
                Err(err) => Err(vm.new_value_error(format!("Read error: {err}"))),
            }
        }
    }
}

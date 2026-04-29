use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("io.remailable.Remailable").qml_file("qml/main.qml"))
        .files(["src/cxxqt.rs"])
        .build();
}
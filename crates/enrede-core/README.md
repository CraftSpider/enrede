
Implementation detail of the enrede library - to support the proc macro using the library in its implementation, we
split out function into `enrede-core` and then re-export both this and the macro in `enrede` itself.

fn main() {
  #[cfg(feature="build")]
  #[cfg(windows)]
  {
    static_vcruntime::metabuild();

    winresource::WindowsResource::new()
      .set_icon("./dist/windows/icon.ico")
      .set_manifest(include_str!("./dist/windows/cvte-mes.manifest"))
      .set_resource_file("./dist/windows/cvte-mes.rc")
      .set_version_info(winresource::VersionInfo::PRODUCTVERSION, 0x0001000000000000)
      .compile()
      .expect("embed Windows resources");
  }
  #[cfg(feature="build")]
  #[cfg(feature = "built")]
  built::write_built_file().expect("Failed to acquire build-time information");
}
